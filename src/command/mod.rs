use chrono::{Local, NaiveDate, NaiveTime};
use clap::{Args, Parser};

pub mod date_range_parser;
pub mod request_provider;

#[derive(Parser)]
pub enum Command {
    /// run benchmark using the same do_get request
    Bench {
        #[command(flatten)]
        a1: CommonArgs,
        #[command(flatten)]
        a2: BenchArgs,
    },
    /// pull down factors in a ranges of days
    Pull {
        #[command(flatten)]
        a1: CommonArgs,
    },
}

#[derive(Args, Clone)]
pub struct CommonArgs {
    /// host of sailor flight server
    pub host: String,
    /// factor name list
    pub factor: Vec<String>,
    #[arg(short, long, default_value_t = 1)]
    /// number of connection using in bench
    pub connection: usize,
    #[arg(short, long, default_value_t = 1)]
    /// number of stream pre connection
    pub stream: usize,
}

#[derive(Args, Clone)]
pub struct BenchArgs {
    #[arg(short, long, default_value_t = current_date())]
    /// specify date
    pub date: NaiveDate,
    #[arg(short, long, default_value_t = default_time())]
    /// specify time
    pub time: NaiveTime,
}

fn current_date() -> NaiveDate {
    Local::now().naive_local().date()
}

fn default_time() -> NaiveTime {
    NaiveTime::from_hms_opt(10, 0, 0).unwrap()
}
