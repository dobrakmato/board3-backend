use ws::listen;
use crate::client::Client;
use log::info;

mod error;
mod ser;
mod de;
mod messages;
mod client;
mod server;
mod auth;

fn main() {
    env_logger::init();

    info!("Starting WebSocket server...");
    listen("0.0.0.0:3013", |out| {
        Client { board_context: None, authenticated_user: None, out }
    }).unwrap()
}
