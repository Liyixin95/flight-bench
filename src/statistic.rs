use std::time::Duration;

use byte_unit::Byte;
use hdrhistogram::Histogram;
use tokio::{
    select,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};

#[derive(Debug)]
pub struct Event {
    size: u64,
    latency: Duration,
    error: Option<anyhow::Error>,
}

impl Event {
    pub fn error(latency: Duration, err: anyhow::Error) -> Self {
        Self {
            size: 0,
            latency,
            error: Some(err),
        }
    }

    pub fn new(size: u64, latency: Duration) -> Self {
        Self { size, latency, error: None }
    }
}

#[derive(Clone)]
pub struct Reporter {
    tx: UnboundedSender<anyhow::Result<Event>>,
}

impl Reporter {
    pub fn err(&self, err: anyhow::Error) {
        self.tx.send(Err(err)).unwrap()
    }

    pub fn report(&self, event: anyhow::Result<Event>) {
        self.tx.send(event).unwrap()
    }
}

pub struct Statistic {
    rx: UnboundedReceiver<anyhow::Result<Event>>,
    close_rx: UnboundedReceiver<()>,
    status: Status,
    summary: Summary,
}

#[derive(Default)]
struct Status {
    size: u64,
    latency: Duration,
    err: u64,
    count: u64,
}

struct Summary {
    total_size: u64,
    total_err: Vec<anyhow::Error>,
    total_latency: Histogram<u64>,
}

impl Summary {
    fn update(&mut self, event: )
}

impl Summary {
    fn pretty_print(self) {
        struct Table {}
    }
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
    async fn run(mut self) {
        const INTERVAL_SEC: u64 = 2;

        let mut interval = tokio::time::interval(Duration::from_secs(INTERVAL_SEC));
        loop {
            select! {
                _ = self.close_rx.recv() => break ,
                Some(event) = self.rx.recv() => {
                    self.status.update(event);
                },
                _ = interval.tick() => {
                    self.status.flash(INTERVAL_SEC);
                },
                else => break ,
            }
        }
    }
}

pub fn start_statistic() -> Reporter {
    let (tx, rx) = unbounded_channel();

    let (close_tx, close_rx) = unbounded_channel();
    let statistic = Statistic {
        rx,
        close_rx,
        status: Default::default(),
        summary: todo!(),
    };

    ctrlc::set_handler(move || {
        close_tx.send(()).expect("Error sending shutdown message");
    })
    .expect("Error setting Ctrl-C handler");

    tokio::spawn(statistic.run());

    Reporter { tx }
}

#[cfg(test)]
mod tests {
    use hdrhistogram::Histogram;

    use super::*;

    #[test]
    fn hdrhistogram_test() {
        let mut histogram = Histogram::<u64>::new(3).unwrap();

        histogram.saturating_record(value, interval);

        eprintln!("histogram. = {:#?}", histogram.len());
    }
}
