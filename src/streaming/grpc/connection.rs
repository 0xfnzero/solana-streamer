use std::time::Duration;
use tonic::transport::channel::ClientTlsConfig;
use yellowstone_grpc_client::{GeyserGrpcClient, Interceptor};
use crate::common::AnyResult;
use crate::streaming::common::constants::{
    DEFAULT_CONNECT_TIMEOUT, DEFAULT_REQUEST_TIMEOUT, DEFAULT_MAX_DECODING_MESSAGE_SIZE
};

/// gRPC connection pool - simplified version
pub struct GrpcConnectionPool {
    endpoint: String,
    x_token: Option<String>,
}

impl GrpcConnectionPool {
    pub fn new(endpoint: String, x_token: Option<String>) -> Self {
        Self {
            endpoint,
            x_token,
        }
    }

    pub async fn create_connection(&self) -> AnyResult<GeyserGrpcClient<impl Interceptor>> {
        let builder = GeyserGrpcClient::build_from_shared(self.endpoint.clone())?
            .x_token(self.x_token.clone())?
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .max_decoding_message_size(DEFAULT_MAX_DECODING_MESSAGE_SIZE)
            .connect_timeout(Duration::from_secs(DEFAULT_CONNECT_TIMEOUT))
            .timeout(Duration::from_secs(DEFAULT_REQUEST_TIMEOUT));

        Ok(builder.connect().await?)
    }
}
