#[macro_use]
extern crate log;
extern crate env_logger;

mod db;
mod model;
use std::convert::TryInto;
use std::time::Instant;
use std::{collections::HashMap, fs::File, path::PathBuf};

use csv::{ReaderBuilder, Trim, WriterBuilder};
use db::{AccountStore, TransactionStore};
use env_logger::Env;
use model::account::Account;
use model::output::Record;
use model::Amount;
use structopt::StructOpt;

use crate::db::TransactionDB;
use crate::model::input::TxRow;
use crate::model::Tx;

fn main() {
    let opt = Opt::from_args();
    let log_level = if opt.debug { "debug" } else { "error" };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    let input_file = match File::open(opt.input) {
        Ok(f) => f,
        Err(e) => {
            error!("can't open file: {:?}", e);
            return;
        }
    };

    let start = Instant::now();
    let mut db = TransactionDB::new(HashMap::default(), HashMap::default());
    read_csv_data(input_file, &mut db);
    print_results(std::io::stdout(), db.accounts.accounts().into_iter());
    debug!("processed in {:?}", start.elapsed());
}

fn read_csv_data<R, T, A>(reader: R, db: &mut TransactionDB<T, A>)
where
    R: std::io::Read,
    T: TransactionStore,
    A: AccountStore,
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
                info!("can't create valid transaction: {:?}", e);
                continue;
            }
        };

        if let Err(e) = db.add(tx) {
            info!("can't process transaction, readson({:?})", e);
            continue;
        }
    }
}

fn print_results(writer: impl std::io::Write, account_iter: impl Iterator<Item = Account>) {
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

#[derive(StructOpt, Debug)]
#[structopt(name = "csvatm")]
struct Opt {
    #[structopt(short, long)]
    pub debug: bool,
    #[structopt(name = "FILE", parse(from_os_str))]
    input: PathBuf,
}
