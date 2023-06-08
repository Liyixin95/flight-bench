use std::time::Duration;

use tokio::{
    select,
    sync::{mpsc::Receiver, oneshot},
    time::Interval,
};

pub struct Statistic {
    rx: Receiver<anyhow::Result<u64>>,
    close_rx: oneshot::Receiver<()>,
}

impl Statistic {
    pub async fn run(mut self) {
        const INTERVAL_SEC: u64 = 2;

        let interval = tokio::time::interval(Duration::from_secs(INTERVAL_SEC));
        select! {
            _ = self.close_rx => {

            },
            res = self.rx.recv() => {

            },
            _ = interval.tick() => {

            },
        }
    }
}
