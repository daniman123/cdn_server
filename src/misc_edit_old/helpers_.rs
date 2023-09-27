use futures_util::StreamExt;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
        API,
    },
    interceptor::registry::Registry,
};

use crate::types::{Tx, WsSplit};

pub async fn accept_and_split(raw_stream: TcpStream) -> WsSplit {
    let tokio_stream = TcpStream::from(raw_stream);
    let ws_stream = match tokio_tungstenite::accept_async(tokio_stream).await {
        Ok(ws_stream) => Ok(ws_stream),
        Err(err) => Err(err),
    };

    ws_stream.unwrap().split()
}

pub fn build_api() -> API {
    let mut media_engine = MediaEngine::default();

    media_engine.register_default_codecs().unwrap();

    let registry = Registry::new();
    let registry = register_default_interceptors(registry, &mut media_engine).unwrap();
    APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build()
}

pub async fn _send_payload(room_users: (SocketAddr, Tx), answer: Message) {
    let (addr, recp) = room_users;
    println!("{:?}", addr);
    if let Err(err) = recp.unbounded_send(answer.clone()) {
        eprintln!("Failed to send message to recipient: {} {:?}", err, recp);
    }

    // let channels = channel_peer_map.lock().unwrap();

    // let broadcast_recipients = channels
    //     .iter()
    //     .filter(|(peer_addr, _)| peer_addr == &&addr)
    //     .map(|(peer_addr, ws_sink)| (peer_addr, ws_sink));

    // for (addr, recp) in room_users {
    //     if let Err(err) = recp.unbounded_send(answer.clone()) {
    //         eprintln!("Failed to send message to recipient: {} {:?}", err, recp);
    //     }
    // }
}
