#[derive(Debug)]
pub struct MoneroRpcClient {
    pub rpc_url: String,
}

impl MoneroRpcClient {
    pub fn from_env() -> Self {
        let rpc_url = std::env::var("MONERO_RPC_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:18081/json_rpc".to_string());

        Self { rpc_url }
    }

    pub fn verify_payment(&self, _tx_hash: &str) -> Result<bool, String> {
        // RPC network call logic goes here
        Ok(true)
    }
}