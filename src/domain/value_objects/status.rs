use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Processing,
    InProgress,
    Success,
    Partial,
    Canceled,
    Error,
}

impl OrderStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Canceled | Self::Error)
    }

    pub fn is_eligible_for_refill(&self) -> bool {
        matches!(self, Self::Success)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DepositStatus {
    Pending,
    WaitingVerification,
    Success,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Member,
    Reseller,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    Default,
    CustomComments,
    Package,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositMethod {
    ManualQris,
    ManualBankBca,
    ManualBankMandiri,
    ManualBankBri,
    AutomaticGateway,
}
