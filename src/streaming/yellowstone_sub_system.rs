use crate::{
    common::AnyResult,
    streaming::{
        grpc::{pool::factory, EventPretty},
        yellowstone_grpc::{TransactionFilter, YellowstoneGrpc},
    },
};
use futures::{SinkExt, StreamExt};
use log::error;
use solana_program::pubkey;
use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestPing,
};

const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");

#[derive(Debug)]
pub enum SystemEvent {
    NewTransfer(TransferInfo),
    Error(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransferInfo {
    pub slot: u64,
    pub signature: String,
    pub tx: Option<SubscribeUpdateTransactionInfo>,
}

impl YellowstoneGrpc {
    pub async fn subscribe_system<F>(
        &self,
        callback: F,
        account_include: Option<Vec<String>>,
        account_exclude: Option<Vec<String>>,
    ) -> AnyResult<()>
    where
        F: Fn(SystemEvent) + Send + Sync + Clone + 'static,
    {
        let addrs = vec![SYSTEM_PROGRAM_ID.to_string()];
        let account_include = account_include.unwrap_or_default();
        let account_exclude = account_exclude.unwrap_or_default();
        let tx_filter =
            vec![TransactionFilter { account_include, account_exclude, account_required: addrs }];
        let transactions = self.subscription_manager.get_subscribe_request_filter(tx_filter, None);
        let (mut subscribe_tx, mut stream, _) = self
            .subscription_manager
            .subscribe_with_request(transactions, None, None, None)
            .await?;

        let callback = Box::new(callback);

        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        let created_at = msg.created_at;
                        match msg.update_oneof {
                            Some(UpdateOneof::Transaction(sut)) => {
                                let transaction_pretty =
                                    factory::create_transaction_pretty_pooled(sut, created_at);
                                let event_pretty = EventPretty::Transaction(transaction_pretty);
                                if let Err(e) =
                                    Self::process_system_transaction(event_pretty, &*callback).await
                                {
                                    error!("Error processing transaction: {e:?}");
                                }
                            }
                            Some(UpdateOneof::Ping(_)) => {
                                let _ = subscribe_tx
                                    .send(SubscribeRequest {
                                        ping: Some(SubscribeRequestPing { id: 1 }),
                                        ..Default::default()
                                    })
                                    .await;
                            }
                            Some(UpdateOneof::Pong(_)) => {
                                // Pong response, no action needed
                            }
                            _ => {
                                // Other message types, ignore for system subscription
                            }
                        }
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });
        Ok(())
    }

    async fn process_system_transaction<F>(event_pretty: EventPretty, callback: &F) -> AnyResult<()>
    where
        F: Fn(SystemEvent) + Send + Sync,
    {
        match event_pretty {
            EventPretty::Transaction(transaction_pretty) => {
                callback(SystemEvent::NewTransfer(TransferInfo {
                    slot: transaction_pretty.slot,
                    signature: transaction_pretty.signature.to_string(),
                    tx: Some(transaction_pretty.grpc_tx),
                }));
            }
            _ => {}
        }
        Ok(())
    }
}
