pub mod peer_helpers;

use futures_util::StreamExt;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
        API,
    },
    interceptor::registry::Registry,
    peer_connection::sdp::session_description::RTCSessionDescription,
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

pub async fn send_payload(local_desc: RTCSessionDescription, tx: Tx) {
    let payload = json!(local_desc);
    let answer = Message::Text(payload.to_string());

    if let Err(err) = tx.unbounded_send(answer.clone()) {
        eprintln!("Failed to send message to recipient: {} {:?}", err, tx);
    }
}
