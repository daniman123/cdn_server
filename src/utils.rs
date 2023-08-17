use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use crate::types::{BroadcastRoom, PeerMap, RoomBroadcastMetaData, WsSplit};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use webrtc::{
    self,
    api::APIBuilder,
    ice_transport::ice_server::RTCIceServer,
    peer_connection::{
        configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
    rtp_transceiver::{
        rtp_codec::RTCRtpCodecParameters, rtp_receiver::RTCRtpReceiver, RTCRtpTransceiver,
    },
    track::track_remote::TrackRemote,
};

pub async fn accept_and_split(raw_stream: TcpStream) -> WsSplit {
    let tokio_stream = TcpStream::from(raw_stream);
    let ws_stream = match tokio_tungstenite::accept_async(tokio_stream).await {
        Ok(ws_stream) => Ok(ws_stream),
        Err(err) => Err(err),
    };

    ws_stream.unwrap().split()
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

pub async fn create_peer_connection() -> RTCPeerConnection {
    let api = APIBuilder::new().build();
    let config = rtc_config();

    api.new_peer_connection(config).await.unwrap()
}

pub async fn handle_broadcast_offer(peer_connection: &RTCPeerConnection) {
    peer_connection.on_track(Box::new(
        move |track: Arc<TrackRemote>,
              _receiver: Arc<RTCRtpReceiver>,
              _transceiver: Arc<RTCRtpTransceiver>| {
            tokio::spawn(handle_track_event(track));
            Box::pin(async {})
        },
    ));
}

pub async fn handle_track_event(track: Arc<TrackRemote>) -> RTCRtpCodecParameters {
    track.codec()
}

pub async fn handle_broadcaster(
    desc: RTCSessionDescription,
    channel_peer_map: PeerMap,
    addr: SocketAddr,
) {
    let peer_connection = create_peer_connection().await;

    handle_broadcast_offer(&peer_connection).await;

    peer_connection.set_remote_description(desc).await.unwrap();

    let answer = peer_connection.create_answer(None).await.unwrap();

    peer_connection.set_local_description(answer).await.unwrap();

    let local_desc = peer_connection.local_description().await.unwrap();

    let payload = json!(local_desc);

    println!("WE ARE IN THE MAINFRAME");

    let answer = Message::Text(payload.to_string());

    send_payload(channel_peer_map, addr, answer).await;
}

pub async fn send_payload(channel_peer_map: PeerMap, addr: SocketAddr, answer: Message) {
    let channels = channel_peer_map.lock().unwrap();

    let broadcast_recipients = channels
        .iter()
        .filter(|(peer_addr, _)| peer_addr == &&addr)
        .map(|(peer_addr, ws_sink)| (peer_addr, ws_sink));

    for (addr, recp) in broadcast_recipients {
        println!("{:?}", addr);
        if let Err(err) = recp.unbounded_send(answer.clone()) {
            eprintln!("Failed to send message to recipient: {} {:?}", err, recp);
        }
    }
}

pub async fn _handle_viewer(
    desc: RTCSessionDescription,
    channel_peer_map: PeerMap,
    addr: SocketAddr,
) {
    let peer_connection = create_peer_connection().await;

    peer_connection.set_remote_description(desc).await.unwrap();

    // TODO -- get media tracks as arg

    let answer = peer_connection.create_answer(None).await.unwrap();

    peer_connection.set_local_description(answer).await.unwrap();

    let local_desc = peer_connection.local_description().await.unwrap();

    let payload = json!(local_desc);

    println!("WE ARE IN THE ");

    let answer = Message::Text(payload.to_string());

    send_payload(channel_peer_map, addr, answer).await;
}

pub async fn payload_extractor(payload: Value, channel_peer_map: PeerMap, addr: SocketAddr) {
    if let Some(Value::Object(obj)) = payload.get("START BROADCAST") {
        if let Ok(payload) = serde_json::from_value::<StartBroadcast>(Value::Object(obj.clone())) {
            handle_broadcaster_role(payload, channel_peer_map, addr).await;
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct StartBroadcast {
    role: String,
    roomName: String,
    localDesc: RTCSessionDescription,
}

pub async fn handle_broadcaster_role(
    broadcast_payload: StartBroadcast,
    channel_peer_map: PeerMap,
    addr: SocketAddr,
) {
    let _room_name = broadcast_payload.roomName;
    let desc = broadcast_payload.localDesc;

    handle_broadcaster(desc, channel_peer_map, addr).await;
}

pub async fn set_broadcast_room(room_name: String, channel_peer_map: BroadcastRoom) {
    let channels = channel_peer_map.lock().unwrap();
    let room_container = HashMap::new();
    // room_container.insert(, v)
}
