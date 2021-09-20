use std::convert::TryInto;

use csv::{ReaderBuilder, Trim, WriterBuilder};

use crate::{
    db::{AccountStore, TransactionDB, TransactionStore},
    model::{account::Account, input::TxRow, output::Record, Amount, Tx},
};

pub fn read_csv_data<'a, R, T, A>(reader: R, db: &mut TransactionDB<'a, T, A>)
where
    R: std::io::Read,
    T: TransactionStore,
    A: AccountStore<'a>,
{
    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_reader(reader);
    for result in reader.deserialize() {
        let row: TxRow = match result {
            Ok(row) => row,
            Err(e) => {
                warn!("can't read row: {:?}", e);
                continue;
            }
        };

        debug!("{:?}", row);
        let tx: Tx = match row.try_into() {
            Ok(tx) => tx,
            Err(e) => {
                warn!("can't create valid transaction: {:?}", e);
                continue;
            }
        };

        if let Err(e) = db.add(tx) {
            warn!("can't process transaction, reason({:?})", e);
            continue;
        }
    }
}

pub fn print_results<'a>(
    writer: impl std::io::Write,
    account_iter: impl Iterator<Item = &'a Account>,
) {
    let mut writer = WriterBuilder::new().from_writer(writer);
    for acc in account_iter {
        let scale = |mut amount: Amount| -> Amount {
            amount.rescale(4);
            amount
        };
        let record = Record {
            client_id: acc.client(),
            balance: scale(acc.balance()),
            held: scale(acc.held()),
            total: scale(acc.total()),
            locked: acc.is_locked(),
        };
        if let Err(e) = writer.serialize(record) {
            warn!("can't serialize element: {:?}", e);
        }
    }
}
