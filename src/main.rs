mod handlers;
mod models;
mod types;
mod utils;

use handlers::handle_connection;
use std::{collections::HashMap, sync::Mutex};
use std::{env, io::Error as IoError};
use tokio::net::TcpListener;

use crate::types::PeerMap;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // let channel_state = BroadcastRoomMap::new();

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);
    let state = PeerMap::new(Mutex::new(HashMap::new()));
    while let Ok((stream, addr)) = listener.accept().await {
        let channel_state = state.clone();
        tokio::spawn(handle_connection(channel_state, stream, addr));
    }

    Ok(())
}
