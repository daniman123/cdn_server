mod event_handlers;
mod helpers;
mod models;
mod roles;
mod types;
mod utils;

use anyhow::Result;
use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use serde_json::Value;
use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc};
use types::ClientsMap;

use crate::helpers::accept_and_split;
use crate::roles::define_socket_user_role;
use std::{env, io::Error as IoError};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    let state = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(async move {
            handle_connection(state, stream, addr).await;
        });
    }
    Ok(())
}

pub async fn handle_connection(
    channel_peer_map: ClientsMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
) {
    println!("Incoming TCP connection from: {}", addr);
    let (tx, rx) = unbounded();
    let (outgoing, incoming) = accept_and_split(raw_stream).await;
    println!("WebSocket connection established: {}", addr);

    let broadcast_incoming = incoming.try_for_each(|msg| {
        if let Ok(message_string) = msg.to_text().map_err(|_| ()) {
            if let Ok(json_data) =
                serde_json::from_str::<serde_json::Map<String, Value>>(&message_string)
            {
                tokio::spawn(define_socket_user_role(
                    json_data,
                    channel_peer_map.clone(),
                    addr,
                    tx.clone(),
                ));
            }
        }
        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
}
