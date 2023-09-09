use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio_tungstenite::tungstenite::Message;

use webrtc::{
    self,
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_H264},
        APIBuilder,
    },
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
    },
    rtp_transceiver::{
        rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType},
        rtp_receiver::RTCRtpReceiver,
        RTCRtpTransceiver,
    },
    track::track_remote::TrackRemote,
};
// use serde_json::Value;
use tokio::net::TcpStream;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

pub async fn handle_connection(channel_peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = match tokio_tungstenite::accept_async(raw_stream).await {
        Ok(ws_stream) => ws_stream,
        Err(err) => {
            eprintln!("Error during the WebSocket handshake: {}", err);
            return;
        }
    };
    println!("WebSocket connection established: {}", addr);

    let (tx, rx) = unbounded();

    channel_peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        // println!("{:?}", msg.clone());
        if let Ok(json) = msg.to_text().map_err(|_| ()) {
            // println!("{:?}", json);
            if let Ok(offer) = serde_json::from_str::<RTCSessionDescription>(&json) {
                // println!("Offer   {:?}", offer);
                tokio::spawn(more_dater(offer, channel_peer_map.clone(), addr));
            }
        }
        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
}

pub async fn create_connection(offer: RTCSessionDescription) -> String {
    let mut media_engine = MediaEngine::default();
    media_engine
        .register_codec(
            RTCRtpCodecParameters {
                capability: RTCRtpCodecCapability {
                    mime_type: MIME_TYPE_H264.to_owned(),
                    clock_rate: 90000,
                    channels: 0,
                    sdp_fmtp_line: "".to_owned(),
                    rtcp_feedback: vec![],
                },
                payload_type: 102,
                ..Default::default()
            },
            RTPCodecType::Video,
        )
        .expect("Failed to add h264 to media engine");

    let registry = Registry::new();
    let registry = register_default_interceptors(registry, &mut media_engine).unwrap();
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    let config = rtc_config();
    let peer_connection = api.new_peer_connection(config).await.unwrap();

    peer_connection
        .add_transceiver_from_kind(RTPCodecType::Video, None)
        .await
        .unwrap();

    peer_connection.on_track(Box::new(
        move |track: Arc<TrackRemote>,
              _receiver: Arc<RTCRtpReceiver>,
              _transceiver: Arc<RTCRtpTransceiver>| {
            tokio::spawn(receive_rtp_track_media(track));
            Box::pin(async {})
        },
    ));

    peer_connection.set_remote_description(offer).await.unwrap();
    let answer = peer_connection.create_answer(None).await.unwrap();
    let mut channel = peer_connection.gathering_complete_promise().await;
    peer_connection.set_local_description(answer).await.unwrap();
    let _ = channel.recv().await;

    let answer = peer_connection.local_description().await.unwrap();
    let json = serde_json::to_string(&answer).unwrap();
    println!("jason {:?}", json);
    // let encoded_json = base64::encode(json);
    // peer_connection
    json
}

pub fn rtc_config() -> RTCConfiguration {
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec![
                "stun:stun1.1.google.com:19302".to_owned(),
                "stun:stun2.1.google.com:19302".to_owned(),
            ],
            ..Default::default()
        }],
        ..Default::default()
    };
    config
}

async fn receive_rtp_track_media(track: Arc<TrackRemote>) {
    let track_codec = track.codec();
    println!("tooo{:?}", track_codec);
}

pub async fn more_dater(offer: RTCSessionDescription, channel_peer_map: PeerMap, addr: SocketAddr) {
    let answer = create_connection(offer).await;

    let peers = channel_peer_map.lock().unwrap();
    let broadcast_recipients = peers
        .iter()
        .filter(|(peer_addr, _)| peer_addr != &&addr)
        .map(|(_, ws_sink)| ws_sink);

    let message = Message::Text(answer);
    println!("{:?}", message);
    for recp in broadcast_recipients {
        println!("{:?}", recp);
        recp.unbounded_send(message.clone()).unwrap();
    }
}

// pub fn send_message(channel_peer_map: BroadcastRoom, chat_user: &Broadcaster, addr: SocketAddr) {
//     let current_active_room = &chat_user.active_room.to_string();

//     // let mut channels = channel_peer_map.lock().unwrap();

//     // let active_channels = channels.get(current_active_room).unwrap();

//     let broadcast_recipients = active_channels
//         .iter()
//         .filter(|(peer_addr, _)| peer_addr != &&addr)
//         .map(|(peer_addr, ws_sink)| (peer_addr, ws_sink));

//     let mut recipients_to_remove = Vec::new();

//     for (addr, recp) in broadcast_recipients {
//         if let Err(_err) = recp.unbounded_send(message.clone()) {
//             // eprintln!("Failed to send message to recipient: {} {:?}", err, recp);
//             // recipients_to_remove.push(*addr);
//             // recp.close_channel();
//         }
//     }
// }
