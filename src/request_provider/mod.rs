use arrow_flight::Ticket;
use tonic::Request;

pub type RequestIteraotr = Box<dyn Iterator<Item = Request<Ticket>> + Send + Sync>;
