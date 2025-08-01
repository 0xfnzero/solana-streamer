use std::{collections::HashMap, fmt, time::Duration};

use chrono::Local;
use futures::{channel::mpsc, sink::Sink, SinkExt, Stream, StreamExt};
use log::{error, info};
use prost_types::Timestamp;
use rustls::crypto::{ring::default_provider, CryptoProvider};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionEncoding};
use tonic::{transport::channel::ClientTlsConfig, Status};
use yellowstone_grpc_client::{GeyserGrpcClient, Interceptor};
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
    SubscribeRequestFilterTransactions, SubscribeRequestPing, SubscribeUpdate,
    SubscribeUpdateTransaction,
};

use crate::common::AnyResult;
use crate::streaming::event_parser::{EventParserFactory, Protocol, UnifiedEvent};

type TransactionsFilterMap = HashMap<String, SubscribeRequestFilterTransactions>;

const CONNECT_TIMEOUT: u64 = 10;
const REQUEST_TIMEOUT: u64 = 60;
const CHANNEL_SIZE: usize = 1000;
const MAX_DECODING_MESSAGE_SIZE: usize = 1024 * 1024 * 10;

#[derive(Clone)]
pub struct TransactionPretty {
    pub slot: u64,
    pub block_time: Option<Timestamp>,
    pub signature: Signature,
    pub is_vote: bool,
    pub tx: EncodedTransactionWithStatusMeta,
}

impl fmt::Debug for TransactionPretty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct TxWrap<'a>(&'a EncodedTransactionWithStatusMeta);
        impl<'a> fmt::Debug for TxWrap<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let serialized = serde_json::to_string(self.0).expect("failed to serialize");
                fmt::Display::fmt(&serialized, f)
            }
        }

        f.debug_struct("TransactionPretty")
            .field("slot", &self.slot)
            .field("signature", &self.signature)
            .field("is_vote", &self.is_vote)
            .field("tx", &TxWrap(&self.tx))
            .finish()
    }
}

impl From<(SubscribeUpdateTransaction, Option<Timestamp>)> for TransactionPretty {
    fn from(
        (SubscribeUpdateTransaction { transaction, slot }, block_time): (
            SubscribeUpdateTransaction,
            Option<Timestamp>,
        ),
    ) -> Self {
        let tx = transaction.expect("should be defined");
        Self {
            slot,
            block_time: block_time,
            signature: Signature::try_from(tx.signature.as_slice()).expect("valid signature"),
            is_vote: tx.is_vote,
            tx: yellowstone_grpc_proto::convert_from::create_tx_with_meta(tx)
                .expect("valid tx with meta")
                .encode(UiTransactionEncoding::Base64, Some(u8::MAX), true)
                .expect("failed to encode"),
        }
    }
}

pub struct YellowstoneGrpc {
    endpoint: String,
    x_token: Option<String>,
}

impl YellowstoneGrpc {
    pub fn new(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        if CryptoProvider::get_default().is_none() {
            default_provider()
                .install_default()
                .map_err(|e| anyhow::anyhow!("Failed to install crypto provider: {:?}", e))?;
        }

        Ok(Self { endpoint, x_token })
    }

    pub async fn connect(&self) -> AnyResult<GeyserGrpcClient<impl Interceptor>> {
        let builder = GeyserGrpcClient::build_from_shared(self.endpoint.clone())?
            .x_token(self.x_token.clone())?
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .max_decoding_message_size(MAX_DECODING_MESSAGE_SIZE)
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT))
            .timeout(Duration::from_secs(REQUEST_TIMEOUT));

        Ok(builder.connect().await?)
    }

    pub async fn subscribe_with_request(
        &self,
        transactions: TransactionsFilterMap,
        commitment: Option<CommitmentLevel>,
    ) -> AnyResult<(
        impl Sink<SubscribeRequest, Error = mpsc::SendError>,
        impl Stream<Item = Result<SubscribeUpdate, Status>>,
    )> {
        let subscribe_request = SubscribeRequest {
            transactions,
            commitment: if let Some(commitment) = commitment {
                Some(commitment as i32)
            } else {
                Some(CommitmentLevel::Processed.into())
            },
            ..Default::default()
        };

        let mut client = self.connect().await?;
        let (sink, stream) = client
            .subscribe_with_request(Some(subscribe_request))
            .await?;
        Ok((sink, stream))
    }

    pub fn get_subscribe_request_filter(
        &self,
        account_include: Vec<String>,
        account_exclude: Vec<String>,
        account_required: Vec<String>,
    ) -> TransactionsFilterMap {
        let mut transactions = HashMap::new();
        transactions.insert(
            "client".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include,
                account_exclude,
                account_required,
            },
        );
        transactions
    }

    pub async fn handle_stream_message(
        msg: SubscribeUpdate,
        tx: &mut mpsc::Sender<TransactionPretty>,
        subscribe_tx: &mut (impl Sink<SubscribeRequest, Error = mpsc::SendError> + Unpin),
    ) -> AnyResult<()> {
        let created_at = msg.created_at;
        match msg.update_oneof {
            Some(UpdateOneof::Transaction(sut)) => {
                let transaction_pretty = TransactionPretty::from((sut, created_at));
                tx.try_send(transaction_pretty)?;
            }
            Some(UpdateOneof::Ping(_)) => {
                subscribe_tx
                    .send(SubscribeRequest {
                        ping: Some(SubscribeRequestPing { id: 1 }),
                        ..Default::default()
                    })
                    .await?;
                info!("service is ping: {}", Local::now());
            }
            Some(UpdateOneof::Pong(_)) => {
                info!("service is pong: {}", Local::now());
            }
            _ => {}
        }
        Ok(())
    }

    /// Subscribe to Yellowstone GRPC service events with advanced filtering options
    ///
    /// This method allows subscribing to specific protocol events with more granular account filtering.
    /// It processes transactions in real-time and calls the provided callback function when matching events are found.
    ///
    /// # Parameters
    ///
    /// * `protocols` - List of protocols to parse (e.g., PumpFun, PumpSwap, Bonk, RaydiumCpmm)
    /// * `bot_wallet` - Optional bot wallet address. If passed: in PumpFunTradeEvent if user is in the address, is_bot=true will be set. In BonkTradeEvent if payer is in the address, is_bot=true will be set. Default is false.
    /// * `account_include` - List of account addresses to include in the subscription
    /// * `account_exclude` - List of account addresses to exclude from the subscription
    /// * `account_required` - List of account addresses that must be present in transactions
    /// * `commitment` - Optional commitment level for the subscription
    /// * `callback` - Function to call when matching events are found
    pub async fn subscribe_events_v2<F>(
        &self,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        account_include: Vec<String>,
        account_exclude: Vec<String>,
        account_required: Vec<String>,
        commitment: Option<CommitmentLevel>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        if account_include.is_empty() && account_exclude.is_empty() && account_required.is_empty() {
            return Err(anyhow::anyhow!(
                "account_include or account_exclude or account_required cannot be empty"
            ));
        }

        let transactions =
            self.get_subscribe_request_filter(account_include, account_exclude, account_required);
        // Subscribe to events
        let (mut subscribe_tx, mut stream) = self
            .subscribe_with_request(transactions, commitment)
            .await?;

        // Create channel
        let (mut tx, mut rx) = mpsc::channel::<TransactionPretty>(CHANNEL_SIZE);

        // Create callback function, wrap with Arc to share across multiple tasks
        let callback = std::sync::Arc::new(Box::new(callback));

        // Start task to process the stream
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(e) =
                            Self::handle_stream_message(msg, &mut tx, &mut subscribe_tx).await
                        {
                            error!("Error handling message: {:?}", e);
                            break;
                        }
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        // Process transactions
        tokio::spawn(async move {
            while let Some(transaction_pretty) = rx.next().await {
                if let Err(e) = Self::process_event_transaction(
                    transaction_pretty,
                    &**callback,
                    bot_wallet,
                    protocols.clone(),
                )
                .await
                {
                    error!("Error processing transaction: {:?}", e);
                }
            }
        });

        tokio::signal::ctrl_c().await?;
        Ok(())
    }

    /// 订阅事件
    #[deprecated(
        since = "0.1.5",
        note = "This method will be removed, please use the new API: subscribe_events_v2"
    )]
    pub async fn subscribe_events<F>(
        &self,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        account_include: Option<Vec<String>>,
        account_exclude: Option<Vec<String>>,
        account_required: Option<Vec<String>>,
        commitment: Option<CommitmentLevel>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        // 创建过滤器
        let protocol_accounts = protocols
            .iter()
            .map(|p| p.get_program_id())
            .flatten()
            .map(|p| p.to_string())
            .collect::<Vec<String>>();
        let mut account_include = account_include.unwrap_or_default();
        let account_exclude = account_exclude.unwrap_or_default();
        let account_required = account_required.unwrap_or_default();

        account_include.extend(protocol_accounts.clone());

        let transactions =
            self.get_subscribe_request_filter(account_include, account_exclude, account_required);

        // 订阅事件
        let (mut subscribe_tx, mut stream) = self
            .subscribe_with_request(transactions, commitment)
            .await?;

        // 创建通道
        let (mut tx, mut rx) = mpsc::channel::<TransactionPretty>(CHANNEL_SIZE);

        // 创建回调函数，使用 Arc 包装以便在多个任务中共享
        let callback = std::sync::Arc::new(Box::new(callback));

        // 启动处理流的任务
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(e) =
                            Self::handle_stream_message(msg, &mut tx, &mut subscribe_tx).await
                        {
                            error!("Error handling message: {:?}", e);
                            break;
                        }
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        // 处理交易
        tokio::spawn(async move {
            while let Some(transaction_pretty) = rx.next().await {
                if let Err(e) = Self::process_event_transaction(
                    transaction_pretty,
                    &**callback,
                    bot_wallet,
                    protocols.clone(),
                )
                .await
                {
                    error!("Error processing transaction: {:?}", e);
                }
            }
        });

        tokio::signal::ctrl_c().await?;
        Ok(())
    }

    async fn process_event_transaction<F>(
        transaction_pretty: TransactionPretty,
        callback: &F,
        bot_wallet: Option<Pubkey>,
        protocols: Vec<Protocol>,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync,
    {
        let program_received_time_ms = chrono::Utc::now().timestamp_millis();
        let slot = transaction_pretty.slot;
        let signature = transaction_pretty.signature.to_string();
        let mut futures = Vec::new();
        for protocol in protocols {
            let parser = EventParserFactory::create_parser(protocol);
            let tx_clone = transaction_pretty.tx.clone();
            let signature_clone = signature.clone();
            let bot_wallet_clone = bot_wallet.clone();

            futures.push(tokio::spawn(async move {
                parser
                    .parse_transaction(
                        tx_clone,
                        &signature_clone,
                        Some(slot),
                        transaction_pretty.block_time,
                        program_received_time_ms,
                        bot_wallet_clone,
                    )
                    .await
                    .unwrap_or_else(|_e| vec![])
            }));
        }

        let results = futures::future::join_all(futures).await;
        for result in results {
            if let Ok(events) = result {
                for event in events {
                    callback(event);
                }
            }
        }
        Ok(())
    }
}
