use std::{net::SocketAddr, time::Instant};

use arrow_flight::{flight_service_client::FlightServiceClient, Ticket};
use futures::stream::TryStreamExt;
use tokio::task::JoinHandle;
use tonic::{transport::Channel, Request};

use crate::{
    command::request_provider::RequestProvider,
    statistic::{Event, Reporter},
};

pub struct Connection {
    client: FlightServiceClient<Channel>,
    request_provider: Vec<RequestProvider>,
    reporter: Reporter,
}

impl Connection {
    pub fn start(self) -> impl Iterator<Item = JoinHandle<()>> {
        self.request_provider.into_iter().map(move |provider| {
            let mut client = self.client.clone();
            let reporter = self.reporter.clone();
            tokio::spawn(async move {
                for req in provider {
                    let event = request(&mut client, req).await;
                    reporter.report(event);
                }
            })
        })
    }
}

async fn request(
    client: &mut FlightServiceClient<Channel>,
    req: Request<Ticket>,
) -> anyhow::Result<Event> {
    let now = Instant::now();

    let resp = client.do_get(req).await?;

    resp.into_inner()
        .map_ok(|bytes| bytes.data_body.len() + bytes.data_header.len())
        .try_fold(0, |acc, n| async move { Ok(acc + n) })
        .await
        .map_err(anyhow::Error::from)
        .map(|len| Event::new(len as u64, now.elapsed()))
}
