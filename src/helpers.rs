use futures_util::StreamExt;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
    },
    interceptor::registry::Registry,
    peer_connection::RTCPeerConnection,
};

use crate::{
    types::{Tx, WsSplit},
    utils::rtc_config,
};

pub async fn accept_and_split(raw_stream: TcpStream) -> WsSplit {
    let tokio_stream = TcpStream::from(raw_stream);
    let ws_stream = match tokio_tungstenite::accept_async(tokio_stream).await {
        Ok(ws_stream) => Ok(ws_stream),
        Err(err) => Err(err),
    };

    ws_stream.unwrap().split()
}

pub async fn create_peer_connection() -> Arc<RTCPeerConnection> {
    let mut media_engine = MediaEngine::default();

    media_engine.register_default_codecs().unwrap();
    // media_engine
    //     .register_codec(
    //         RTCRtpCodecParameters {
    //             capability: RTCRtpCodecCapability    {
    //                 mime_type: MIME_TYPE_H264.to_owned(),
    //                 clock_rate: 90000,
    //                 channels: 0,
    //                 sdp_fmtp_line: "".to_owned(),
    //                 rtcp_feedback: vec![],
    //             },
    //             payload_type: 102,
    //             ..Default::default()
    //         },
    //         RTPCodecType::Video,
    //     )
    //     .expect("Failed to add h264 to media engine");

    let registry = Registry::new();
    let registry = register_default_interceptors(registry, &mut media_engine).unwrap();
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    // let api = APIBuilder::new().build();
    let config = rtc_config();

    Arc::new(api.new_peer_connection(config).await.unwrap())
}

pub async fn send_payload(room_users: (SocketAddr, Tx), answer: Message) {

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
