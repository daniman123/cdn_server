use crate::{
    helpers::{create_peer_connection, send_payload},
    models::Room,
    types::Tx,
};

use std::{net::SocketAddr, sync::Arc};

use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use webrtc::{
    self,
    ice_transport::ice_server::RTCIceServer,
    peer_connection::{
        configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
    rtp_transceiver::rtp_codec::{RTCRtpCodecParameters, RTPCodecType},
    track::track_remote::TrackRemote,
};

// "stun:stun2.1.google.com:19302".to_owned(),

pub fn rtc_config() -> RTCConfiguration {
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec![
                "stun:stun.l.google.com:19302".to_owned(),
            ],
            ..Default::default()
        }],
        ..Default::default()
    };
    config
}

pub async fn handle_broadcast_offer(peer_connection: Arc<RTCPeerConnection>, room: Room) {
    peer_connection.on_track(Box::new(move |track: Arc<TrackRemote>, _, _| {
        tokio::spawn(handle_track_event(track, room.clone()));
        Box::pin(async {})
    }));
}

pub async fn handle_track_event(track: Arc<TrackRemote>, mut room: Room) {
    println!("{:?}", track.kind());
    room.broadcast = track.codec();
}

pub async fn handle_broadcaster(
    peer_connection: Arc<RTCPeerConnection>,
    desc: RTCSessionDescription,
    mut room: Room,
) {
    peer_connection
        .add_transceiver_from_kind(RTPCodecType::Video, None)
        .await
        .unwrap();

    if let Err(err) = peer_connection.set_remote_description(desc).await {
        eprintln!("Error setting remote description: {:?}", err);
        return;
    }

    // for codec_type in &[RTPCodecType::Video, RTPCodecType::Audio] {
    //     if let Err(err) = peer_connection
    //         .add_transceiver_from_kind(*codec_type, None)
    //         .await
    //     {
    //         eprintln!("Error adding transceiver: {:?}", err);
    //         return;
    //     }
    // }
    let dees = Arc::clone(&peer_connection);
    handle_broadcast_offer(dees, room.clone()).await;

    let answer = match peer_connection.create_answer(None).await {
        Ok(answer) => answer,
        Err(err) => {
            eprintln!("Error creating answer: {:?}", err);
            return;
        }
    };

    if let Err(err) = peer_connection.set_local_description(answer).await {
        eprintln!("Error setting local description: {:?}", err);
        return;
    }

    if let Some(local_desc) = peer_connection.local_description().await {
        let payload = json!(local_desc);
        println!("WE ARE IN THE MAINFRAME");
        room.broadcast = RTCRtpCodecParameters::default();
        let answer = Message::Text(payload.to_string());
        send_payload(room.broadcaster, answer).await
    } else {
        eprintln!("Error getting local description");
    }
}

pub async fn _handle_viewer(desc: RTCSessionDescription, _room: Room, _addr: (SocketAddr, Tx)) {
    let peer_connection = create_peer_connection().await;

    peer_connection.set_remote_description(desc).await.unwrap();

    // TODO -- get media tracks as arg
    // let tracks = room.broadcast.capability.channels;

    // peer_connection.add_track(track);

    let answer = peer_connection.create_answer(None).await.unwrap();

    peer_connection.set_local_description(answer).await.unwrap();

    let local_desc = peer_connection.local_description().await.unwrap();

    let payload = json!(local_desc);

    println!("WE ARE IN THE ");

    let _answer = Message::Text(payload.to_string());

    // send_payload(channel_peer_map, addr, answer).await;
}
