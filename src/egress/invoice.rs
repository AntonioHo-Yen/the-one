#[derive(Debug, Clone)]
pub struct LicensingInvoice {
    pub invoice_id: String,
    pub amount_atomic_units: u64,
    pub destination_address: String,
    pub is_paid: bool,
}

impl LicensingInvoice {
    pub fn new(invoice_id: String, amount: u64, destination: String) -> Self {
        Self {
            invoice_id,
            amount_atomic_units: amount,
            destination_address: destination,
            is_paid: false,
        }
    }

    pub fn mark_paid(&mut self) {
        self.is_paid = true;
    }
}