use clap::Parser;

use crate::command::Command;

mod command;
mod statistic;
mod task;

#[tokio::main]
async fn main() {
    let command = Command::parse();
    match command {
        Command::Pull { a1 } => {
            println!("unsupport yet");
        }
        Command::Bench { a1, a2 } => {}
    }
}

async fn bench() {}
