use std::collections::HashMap;

use crate::model::{ClientId, TransactionId, TxRecord};

use super::{TransactionStore, TransactionStoreError};

impl TransactionStore for HashMap<TransactionId, TxRecord> {
    fn get_tx_mut(
        &mut self,
        client_id: &ClientId,
        id: &TransactionId,
    ) -> Result<Option<&mut TxRecord>, TransactionStoreError> {
        match self.get_mut(id) {
            Some(trans) => {
                //ensure that existing transaction is owned by request client
                if &trans.client_id != client_id {
                    return Err(TransactionStoreError::IntegrityError);
                }
                Ok(Some(trans))
            }
            None => Ok(None),
        }
    }

    fn add(&mut self, id: TransactionId, record: TxRecord) -> Result<(), TransactionStoreError> {
        if self.get(&id).is_some() {
            return Err(TransactionStoreError::TransactionAlreadyExists);
        }
        self.insert(id, record);
        Ok(())
    }
}
