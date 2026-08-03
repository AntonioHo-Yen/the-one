use crate::egress::invoice::LicensingInvoice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateStatus {
    Locked,
    Unlocked,
}

#[derive(Debug)]
pub struct EgressGate {
    pub status: GateStatus,
}

impl EgressGate {
    pub fn new() -> Self {
        Self {
            status: GateStatus::Locked,
        }
    }

    pub fn evaluate_settlement(&mut self, invoice: &LicensingInvoice) -> bool {
        if invoice.is_paid {
            self.status = GateStatus::Unlocked;
            true
        } else {
            self.status = GateStatus::Locked;
            false
        }
    }
}

impl Default for EgressGate {
    fn default() -> Self {
        Self::new()
    }
}