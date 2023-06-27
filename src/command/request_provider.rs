use arrow_flight::Ticket;
use bytes::Bytes;
use chrono::{NaiveDate, NaiveTime};
use serde::Serialize;
use tonic::Request;

pub type RequestProvider = Box<dyn Iterator<Item = Request<Ticket>> + Send + Sync + 'static>;

#[derive(Serialize)]
struct GetCmd {
    date: String,
    timestamp: String,
    name: Vec<String>,
}

pub fn inifinate_request_provider(
    date: NaiveDate,
    time: NaiveTime,
    factors: Vec<String>,
) -> RequestProvider {
    let cmd = GetCmd {
        date: date.to_string(),
        timestamp: time.to_string(),
        name: factors,
    };

    let ticket = Ticket {
        ticket: Bytes::from(serde_json::to_vec(&cmd).unwrap()),
    };

    Box::new(std::iter::repeat_with(move || Request::new(ticket.clone())))
}
