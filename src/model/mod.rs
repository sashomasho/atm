use rust_decimal::Decimal;
pub mod account;
pub mod input;
pub mod output;

pub type TransactionId = u32;
pub type ClientId = u16;
pub type Amount = Decimal;

/// A dispute may be in one of the tree states - Initiated, Resolved and ChargeBack
/// Valid transitions are:
/// Initiated -> Resolved
/// Initiated -> ChargeBack
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisputeState {
    Initiated,
    Resolved,
    ChargeBack,
}

/// A valid transaction can be one of the following: Deposit, Withdraw, Dispute{Initiated,
/// Resolved, ChargeBack}
#[derive(Debug, PartialEq, Eq)]
pub enum TxOperation {
    Deposit(Amount),
    Withdraw(Amount),
    Dispute(DisputeState),
}

/// A singe transaction than needs to be processed, contains transaction_id that is globally unique
#[derive(Debug, PartialEq, Eq)]
pub struct Tx {
    pub transaction_id: TransactionId,
    pub client_id: ClientId,
    pub operation: TxOperation,
}

/// Each TxRecord is constructed with one of the following: Deposit or Withdraw
pub enum TxRecordType {
    Deposit(Amount),
    Withdraw(Amount),
}

/// TxRecord is the main entity responsible for the lifecycle of the transaction,
/// once created with TxRecordType it can be further modified by setting `dispute` field
pub struct TxRecord {
    pub origin: TxRecordType,
    pub client_id: ClientId,
    pub dispute: Option<DisputeState>,
}

impl TxRecord {
    pub fn amount(&self) -> Amount {
        match self.origin {
            TxRecordType::Deposit(amount) => amount,
            TxRecordType::Withdraw(amount) => amount,
        }
    }
}
