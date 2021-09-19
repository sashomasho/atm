#[macro_use]
extern crate log;
extern crate env_logger;

mod db;
mod io;
mod model;

use std::{collections::HashMap, fs::File, path::PathBuf, time::Instant};

use env_logger::Env;
use structopt::StructOpt;

use crate::{
    db::TransactionDB,
    io::{print_results, read_csv_data},
};

fn main() {
    //read cli arguments
    let opt = Opt::from_args();
    //init logger
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
    print_results(std::io::stdout(), db.accounts().into_iter());
    debug!("processed in {:?}", start.elapsed());
}

#[derive(StructOpt)]
#[structopt(name = "csvatm")]
struct Opt {
    #[structopt(short, long)]
    pub debug: bool,
    #[structopt(name = "FILE", parse(from_os_str))]
    input: PathBuf,
}
