use crate::egress::invoice::LicensingInvoice;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MoneroError {
    #[error("HTTP RPC request failed: {0}")]
    NetworkError(String),
    #[error("Payment verification failed for subaddress: {0}")]
    PaymentNotFound(String),
}

#[derive(Debug, Clone)]
pub struct MoneroRpcClient {
    pub rpc_url: String,
    pub account_index: u32,
}

impl MoneroRpcClient {
    pub fn new(rpc_url: impl Into<String>, account_index: u32) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            account_index,
        }
    }

    pub fn generate_subaddress_invoice(&self, amount_xmr: f64) -> LicensingInvoice {
        let fake_subaddress = format!("888tXMR_{}", self.account_index);
        let invoice_id = format!("INV-{}", self.account_index);

        // Convert f64 XMR to atomic units (piconero / u64)
        let amount_piconeros = (amount_xmr * 1e12) as u64;

        LicensingInvoice::new(invoice_id, amount_piconeros, fake_subaddress)
    }

    pub async fn check_payment_status(&self, _subaddress: &str) -> Result<bool, MoneroError> {
        let _client = reqwest::Client::new();
        
        // Live RPC check logic goes here
        Ok(true)
    }

    pub fn verify_payment(&self, _tx_hash: &str) -> Result<bool, String> {
        Ok(true)
    }
}