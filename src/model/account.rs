use rust_decimal::Decimal;
use thiserror::Error;

use crate::db::{TransactionStore, TransactionStoreError};

use super::{
    Amount, ClientId, DisputeState, TransactionId, Tx, TxOperation, TxRecord, TxRecordType,
};

/// Account is he main entity that is responsible for transaction processing,
/// keep the internals private, should be modified only by transaction
#[derive(Debug, Clone)]
pub struct Account {
    client_id: ClientId,
    total: Amount,
    held: Amount,
    locked: bool,
}

impl Account {
    /// construct new Account
    pub fn new(client_id: ClientId) -> Self {
        Account {
            client_id,
            total: Amount::default(),
            held: Amount::default(),
            locked: false,
        }
    }

    /// returns client id
    pub fn client(&self) -> ClientId {
        self.client_id
    }

    /// chek current balance, i.e. available funds
    pub fn balance(&self) -> Amount {
        self.total - self.held
    }

    /// check the total amount in the account
    pub fn total(&self) -> Amount {
        self.total
    }

    /// the held amount
    pub fn held(&self) -> Amount {
        self.held
    }

    /// if account is locked no transactions should be processed
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// process new transaction
    pub fn process<T>(&mut self, tx: Tx, store: &mut T) -> Result<(), TxError>
    where
        T: TransactionStore,
    {
        if self.locked {
            return Err(TxError::AccountLocked(self.client_id));
        }

        match tx.operation {
            //always allow
            TxOperation::Deposit(amount) => {
                self.total += amount;
                store.add(
                    tx.transaction_id,
                    TxRecord {
                        origin: TxRecordType::Deposit(amount),
                        dispute: None,
                        client_id: self.client_id,
                    },
                )?;
            }
            TxOperation::Withdraw(amount) => {
                //allow only if balance >= amount
                let balance = self.balance();
                if amount > balance {
                    return Err(TxError::InsufficientFunds(tx.transaction_id));
                }
                self.total -= amount;
                store.add(
                    tx.transaction_id,
                    TxRecord {
                        origin: TxRecordType::Withdraw(amount),
                        dispute: None,
                        client_id: self.client_id,
                    },
                )?;
            }
            TxOperation::Dispute(new_dispute) => {
                match store.get_tx_mut(&self.client_id, &tx.transaction_id)? {
                    Some(prev_tx) => match new_dispute {
                        DisputeState::Initiated => match prev_tx.dispute {
                            None => {
                                //XXX at this point the balance may become negative value
                                self.held += prev_tx.amount();
                                prev_tx.dispute = Some(new_dispute);
                            }
                            _ => {
                                return Err(TxError::InvalidState(new_dispute, prev_tx.dispute));
                            }
                        },
                        DisputeState::Resolved => {
                            match prev_tx.dispute {
                                //resolve only if initiated
                                Some(DisputeState::Initiated) => {
                                    prev_tx.dispute = Some(new_dispute);
                                    self.held -= prev_tx.amount();
                                    assert!(self.held >= Decimal::from(0));
                                }
                                _ => {
                                    return Err(TxError::InvalidState(
                                        new_dispute,
                                        prev_tx.dispute,
                                    ));
                                }
                            }
                        }
                        DisputeState::ChargeBack => {
                            //chargeback only if duspute is initiated, lock accout afterwards
                            //not sure if we want to perform chargeback if there is not sufficent
                            //amount, but my gut feeling is that we should perform it, even if the
                            //total becomes less than 0
                            match prev_tx.dispute {
                                Some(DisputeState::Initiated) => {
                                    prev_tx.dispute = Some(new_dispute);
                                    let amount = prev_tx.amount();
                                    self.held -= amount;
                                    self.total -= amount;
                                    self.locked = true;
                                }
                                _ => {
                                    return Err(TxError::InvalidState(
                                        new_dispute,
                                        prev_tx.dispute,
                                    ));
                                }
                            }
                        }
                    },
                    None => {
                        return Err(TxError::TransactionNotFound(tx.transaction_id));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TxError {
    #[error("account locked: {0:?}")]
    AccountLocked(ClientId),
    #[error("insufficent funds")]
    InsufficientFunds(TransactionId),
    #[error("transaction not found: {0:?}")]
    TransactionNotFound(TransactionId),
    #[error("invalid dispute state for: {0:?}, {1:?}")]
    InvalidState(DisputeState, Option<DisputeState>),
    #[error(transparent)]
    IntegrityError(#[from] TransactionStoreError),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::model::{
        account::TxError, Amount, DisputeState, TransactionId, Tx, TxOperation, TxRecord,
    };

    use super::Account;

    #[test]
    fn test_processing() {
        // perform multiple transactions on a single account, check the state after each one
        let mut acc = Account::new(12);
        let mut store: HashMap<TransactionId, TxRecord> = Default::default();

        assert_eq!(acc.balance(), 0.into());
        acc.process(
            Tx {
                transaction_id: 1,
                client_id: 12,
                operation: TxOperation::Deposit(Amount::from(10)),
            },
            &mut store,
        )
        .expect("should succeed");

        //check balance
        assert_eq!(acc.balance(), Amount::from(10));

        //try to withdraw more than the avaiable amount
        let res = acc.process(
            Tx {
                transaction_id: 2,
                client_id: 12,
                operation: TxOperation::Withdraw(Amount::from(20)),
            },
            &mut store,
        );
        assert_eq!(res, Err(TxError::InsufficientFunds(2)));

        //try to withdraw lower amount
        acc.process(
            Tx {
                transaction_id: 3,
                client_id: 12,
                operation: TxOperation::Withdraw(Amount::from(5)),
            },
            &mut store,
        )
        .expect("witdhraw should succeed");
        assert_eq!(acc.balance(), Amount::from(5));
        assert_eq!(acc.total(), Amount::from(5));
        assert_eq!(acc.held(), Amount::from(0));

        //try to dispute transaction 2, which was not successful
        let res = acc.process(
            Tx {
                transaction_id: 2,
                client_id: 12,
                operation: TxOperation::Dispute(DisputeState::Initiated),
            },
            &mut store,
        );
        assert_eq!(res, Err(TxError::TransactionNotFound(2)));

        //try to dispute transaction 3
        acc.process(
            Tx {
                transaction_id: 3,
                client_id: 12,
                operation: TxOperation::Dispute(DisputeState::Initiated),
            },
            &mut store,
        )
        .expect("disput should be processed");

        assert_eq!(acc.balance(), Amount::from(0));
        assert_eq!(acc.held(), Amount::from(5));
        assert_eq!(acc.total(), Amount::from(5));

        //try to dispute transaction 3 again
        let res = acc.process(
            Tx {
                transaction_id: 3,
                client_id: 12,
                operation: TxOperation::Dispute(DisputeState::Initiated),
            },
            &mut store,
        );

        assert_eq!(
            res,
            Err(TxError::InvalidState(
                DisputeState::Initiated,
                Some(DisputeState::Initiated)
            ))
        );

        //try to resolve the dispute for transaction 3
        acc.process(
            Tx {
                transaction_id: 3,
                client_id: 12,
                operation: TxOperation::Dispute(DisputeState::Resolved),
            },
            &mut store,
        )
        .expect("resolve for transaction 3 should succeed");
        assert_eq!(acc.balance(), Amount::from(5));
        assert_eq!(acc.total(), Amount::from(5));
        assert_eq!(acc.held(), Amount::from(0));

        //try to resolve the dispute for transaction 3 again
        let res = acc.process(
            Tx {
                transaction_id: 3,
                client_id: 12,
                operation: TxOperation::Dispute(DisputeState::Resolved),
            },
            &mut store,
        );
        assert_eq!(
            res,
            Err(TxError::InvalidState(
                DisputeState::Resolved,
                Some(DisputeState::Resolved)
            ))
        );

        //try to perform dispure for transaction 1
        acc.process(
            Tx {
                transaction_id: 1,
                client_id: 12,
                operation: TxOperation::Dispute(DisputeState::Initiated),
            },
            &mut store,
        )
        .expect("dispute for the first transaction should be ok");
        assert_eq!(acc.balance(), Amount::from(-5));
        assert_eq!(acc.total(), Amount::from(5));
        assert_eq!(acc.held(), Amount::from(10));

        //try to charge back the dispute for transaction 1
        acc.process(
            Tx {
                transaction_id: 1,
                client_id: 12,
                operation: TxOperation::Dispute(DisputeState::ChargeBack),
            },
            &mut store,
        )
        .expect("chargeback for transaction 1 should succeed");
        assert_eq!(acc.balance(), Amount::from(-5));
        assert_eq!(acc.total(), Amount::from(-5)); //is this OK?!?
        assert_eq!(acc.held(), Amount::from(0));
        assert!(acc.is_locked());

        //check if account when locked is really locked
        let res = acc.process(
            Tx {
                transaction_id: 1,
                client_id: 12,
                operation: TxOperation::Deposit(Amount::from(100)),
            },
            &mut store,
        );
        assert_eq!(res, Err(TxError::AccountLocked(12)));
        //accout amounts should stay the same
        assert_eq!(acc.balance(), Amount::from(-5));
        assert_eq!(acc.total(), Amount::from(-5));
        assert_eq!(acc.held(), Amount::from(0));

        //check the number of transactions, should be 2
        assert_eq!(store.len(), 2);
    }
}
