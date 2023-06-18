use std::time::Duration;

use byte_unit::Byte;
use tokio::{
    select,
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};

#[derive(Debug)]
pub struct Event {
    size: u64,
    latency: Duration,
}

impl Event {
    pub fn new(size: u64, latency: Duration) -> Self {
        Self { size, latency }
    }
}

#[derive(Clone)]
pub struct Reporter {
    tx: UnboundedSender<anyhow::Result<Event>>,
}

impl Reporter {
    pub fn report(&self, event: anyhow::Result<Event>) {
        self.tx.send(event).unwrap()
    }
}

pub struct Statistic {
    rx: UnboundedReceiver<anyhow::Result<Event>>,
    close_rx: oneshot::Receiver<()>,
    status: Status,
}

struct Status {
    size: u64,
    latency: Duration,
    err: u64,
    count: u64,
}

impl Status {
    fn update(&mut self, event: anyhow::Result<Event>) {
        match event {
            Ok(event) => {
                self.size += event.size;
                self.latency += event.latency;
            }
            Err(_) => {
                self.err += 1;
            }
        }
    }

    fn flash(&mut self, interval: u64) {
        let throughput = Byte::from_bytes(self.size / interval).get_appropriate_unit(true);
        let latency = (self.latency / interval as u32).as_secs_f32();
        let qps = self.count / interval;
        let err = self.err / interval;

        println!("throughput {throughput}, qps: {qps}/sec, latency: {latency} s, err: {err}/sec",);

        self.size = 0;
        self.latency = Duration::ZERO;
        self.err = 0;
        self.count = 0;
    }
}

impl Statistic {
    pub async fn run(mut self) {
        const INTERVAL_SEC: u64 = 2;

        let mut interval = tokio::time::interval(Duration::from_secs(INTERVAL_SEC));
        select! {
            _ = self.close_rx => {
                return ;
            },
            Some(event) = self.rx.recv() => {
                self.status.update(event);
            },
            _ = interval.tick() => {
                self.status.flash(INTERVAL_SEC);
            },
            else => return ,
        }
    }
}
