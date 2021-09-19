use std::collections::HashMap;

use crate::model::{account::Account, ClientId};

use super::AccountStore;

pub struct AccountsIter {
    //TODO borrow accounts
    accounts: Vec<Account>,
}

impl IntoIterator for AccountsIter {
    type Item = Account;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.into_iter()
    }
}

impl AccountStore for HashMap<ClientId, Account> {
    fn get_account_mut(&mut self, client_id: &ClientId) -> Option<&mut Account> {
        self.get_mut(client_id)
    }

    fn add_account(&mut self, client_id: ClientId, account: Account) -> &mut Account {
        self.entry(client_id).or_insert(account)
    }

    fn accounts(&self) -> AccountsIter {
        AccountsIter {
            accounts: self.values().into_iter().cloned().collect(),
        }
    }

    type IteratorType = AccountsIter;
}
