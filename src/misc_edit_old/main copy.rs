mod broadcaster_role;
mod event_handlers;
mod handlers;
mod helpers;
mod models;
mod types;
mod utils;
mod viewer_role;

use futures_util::StreamExt;
// use futures_util::TryStreamExt;
// use futures_util::pin_mut;
// use handlers::handle_connection;
// use futures_util::future;
// use serde_json::Value;
// use tokio::sync::mpsc::unbounded_channel;
use tokio_tungstenite::tungstenite::Message;
// use std::collections::HashMap;
use std::net::SocketAddr;
// use std::sync::Mutex;
use crate::helpers::accept_and_split;
use futures_channel::mpsc::unbounded;
// use futures_util::{future, pin_mut};
use std::{env, io::Error as IoError};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
// use crate::types::BroadcastRoomMap;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // let state = BroadcastRoomMap::new(Mutex::new(HashMap::new()));
    let (local_track_chan_tx, mut local_track_chan_rx) = tokio::sync::mpsc::channel::<String>(1);

    while let Ok((stream, addr)) = listener.accept().await {
        let local_track_chan_tx = local_track_chan_tx.clone();
        tokio::spawn(async move {
            handle_connection(stream, addr, local_track_chan_tx).await;
        });
        if let Some(message) = local_track_chan_rx.recv().await {
            // Handle the received message here
            println!("Received message: {:?}", message);
            // You can perform actions based on the received message
        }
    }

    Ok(())
}

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    local_track_chan_tx: Sender<String>,
) {
    println!("Incoming TCP connection from: {}", addr);
    let (tx, rx) = unbounded();
    let (outgoing, mut incoming) = accept_and_split(stream).await;
    println!("WebSocket connection established: {}", addr);

    tokio::spawn(rx.forward(outgoing));

    while let Some(msg) = incoming.next().await {
        let msg = msg
            .unwrap_or(Message::Text("noooo".to_string()))
            .to_string()
            .clone();

        local_track_chan_tx.send(msg.clone()).await.unwrap();
        tx.unbounded_send(Ok(Message::Text(msg.to_string())))
            .unwrap();
    }
   
}
