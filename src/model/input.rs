use std::convert::TryFrom;

use serde::{self, Deserialize};
use thiserror::Error;

use super::{Amount, ClientId, DisputeState, TransactionId, Tx, TxOperation};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    ChargeBack,
}

#[derive(Debug, Deserialize)]
pub struct TxRow {
    #[serde(rename = "tx")]
    transaction_id: TransactionId,
    #[serde(rename = "client")]
    client_id: ClientId,
    #[serde(rename = "type")]
    row_type: TransactionType,
    #[serde(rename = "amount")]
    amount: Option<Amount>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ConversionError {
    #[error("deposit without amount")]
    DepositWithoutAmount,
    #[error("withdraw without amount")]
    WithdrawalWithoutAmount,
    #[error("dispute action should not contain amount")]
    DisputeWithAmount,
}

impl TryFrom<TxRow> for Tx {
    type Error = ConversionError;

    fn try_from(value: TxRow) -> Result<Self, Self::Error> {
        let operation = match value.amount {
            Some(amount) => match value.row_type {
                TransactionType::Deposit => TxOperation::Deposit(amount),
                TransactionType::Withdrawal => TxOperation::Withdraw(amount),
                TransactionType::Dispute
                | TransactionType::Resolve
                | TransactionType::ChargeBack => {
                    return Err(ConversionError::DisputeWithAmount);
                }
            },
            None => match value.row_type {
                TransactionType::Deposit => return Err(ConversionError::DepositWithoutAmount),
                TransactionType::Withdrawal => {
                    return Err(ConversionError::WithdrawalWithoutAmount)
                }
                TransactionType::Dispute => TxOperation::Dispute(DisputeState::Initiated),
                TransactionType::Resolve => TxOperation::Dispute(DisputeState::Resolved),
                TransactionType::ChargeBack => TxOperation::Dispute(DisputeState::ChargeBack),
            },
        };
        Ok(Tx {
            transaction_id: value.transaction_id,
            client_id: value.client_id,
            operation,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::model::{input::ConversionError, Amount, Tx, TxOperation};

    use super::TxRow;

    #[test]
    fn test_conversion() {
        let row = TxRow {
            transaction_id: 1,
            client_id: 1,
            row_type: super::TransactionType::Deposit,
            amount: Some(Amount::from(10)),
        };

        assert_eq!(
            row.try_into(),
            Ok(Tx {
                transaction_id: 1,
                client_id: 1,
                operation: TxOperation::Deposit(Amount::from(10)),
            })
        );

        let row = TxRow {
            transaction_id: 2,
            client_id: 2,
            row_type: crate::model::input::TransactionType::Resolve,
            amount: Some(Amount::from(10)),
        };

        let res: Result<Tx, ConversionError> = row.try_into();
        assert_eq!(res, Err(ConversionError::DisputeWithAmount));
    }
}
