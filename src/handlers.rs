use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use serde_json::Value;
use std::net::SocketAddr;

use tokio::net::TcpStream;

use crate::{
    types::PeerMap,
    utils::{accept_and_split, payload_extractor},
};

pub async fn handle_connection(channel_peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);
    let (tx, rx) = unbounded();
    channel_peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = accept_and_split(raw_stream).await;
    println!("WebSocket connection established: {}", addr);

    let broadcast_incoming = incoming.try_for_each(|msg| {
        if let Ok(json) = msg.to_text().map_err(|_| ()) {
            if let Ok(payload) = serde_json::from_str::<Value>(&json) {
                tokio::spawn(payload_extractor(payload, channel_peer_map.clone(), addr));
            }
        }
        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
}
