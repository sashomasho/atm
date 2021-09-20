use std::collections::HashMap;

use crate::model::{account::Account, ClientId};

use super::AccountStore;

pub struct AccountsIter<'a> {
    accounts: &'a HashMap<ClientId, Account>,
}

impl<'a> IntoIterator for AccountsIter<'a> {
    type Item = &'a Account;
    type IntoIter = std::collections::hash_map::Values<'a, ClientId, Account>;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.values()
    }
}

impl<'a> AccountStore<'a> for HashMap<ClientId, Account> {
    type IteratorType = AccountsIter<'a>;

    fn get_account_mut(&mut self, client_id: &ClientId) -> Option<&mut Account> {
        self.get_mut(client_id)
    }

    fn add_account(&mut self, client_id: ClientId, account: Account) -> &mut Account {
        self.entry(client_id).or_insert(account)
    }

    fn accounts(&'a self) -> AccountsIter<'a> {
        AccountsIter { accounts: self }
    }
}
