use thiserror::Error;
mod accounts;
mod transactions;

use super::model::{
    account::{Account, TxError},
    ClientId, TransactionId, Tx, TxRecord,
};

pub struct TransactionDB<T: TransactionStore, A: AccountStore> {
    pub accounts: A,
    pub transactions: T,
}

impl<T: TransactionStore, A: AccountStore> TransactionDB<T, A> {
    pub fn new(transaction_store: T, account_store: A) -> Self {
        TransactionDB {
            accounts: account_store,
            transactions: transaction_store,
        }
    }
}

pub trait AccountStore {
    type IteratorType;
    fn get_account_mut(&mut self, client_id: &ClientId) -> Option<&mut Account>;
    fn add_account(&mut self, client_id: ClientId, account: Account) -> &mut Account;
    fn accounts(&self) -> Self::IteratorType;
}

pub trait TransactionStore {
    fn add(&mut self, id: TransactionId, record: TxRecord) -> Result<(), TransactionStoreError>;

    fn get_tx_mut(
        &mut self,
        client_id: &ClientId,
        id: &TransactionId,
    ) -> Result<Option<&mut TxRecord>, TransactionStoreError>;
}

impl<T: TransactionStore, A: AccountStore> TransactionDB<T, A> {
    pub fn add(&mut self, tx: Tx) -> Result<(), TxError> {
        let account = match self.accounts.get_account_mut(&tx.client_id) {
            Some(acc) => acc,
            None => self
                .accounts
                .add_account(tx.client_id, Account::new(tx.client_id)),
        };

        account.process(tx, &mut self.transactions)?;
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransactionStoreError {
    #[error("integrity error")]
    IntegrityError,
    #[error("transaction already exists")]
    TransactionAlreadyExists,
}
