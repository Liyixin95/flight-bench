use std::net::SocketAddr;

use arrow_flight::FlightClient;

pub struct Connection<B> {
    host: SocketAddr,
    client: FlightClient,
    request_provider: B,
}
