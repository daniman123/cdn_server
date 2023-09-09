use anyhow::Result;
use serde_json::{json, Map, Value};
use std::{net::SocketAddr, sync::Arc};
// use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use webrtc::{
    // ice_transport::ice_candidate::RTCIceCandidateInit,
    peer_connection::{
        peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription, RTCPeerConnection,
    },
    track::track_local::TrackLocal,
};

use crate::{
    event_handlers::handle_candidate_event,
    models::{Room, ViewerMetaData, ViewerPayload},
    types::{ClientsMap, Tx},
    utils::create_peer_connection,
};

pub async fn process_viewer(
    raw_payload: &Map<String, Value>,
    channel_peer_map: ClientsMap,
    tx: Tx,
    addr: SocketAddr,
) {
    // Handle Viewer payload
    // TODO - Handle role function
    let mut channels = channel_peer_map.lock().await;
    if let Ok(parsed_payload) =
        serde_json::from_value::<ViewerPayload>(Value::Object(raw_payload.clone()))
    {
        let room_name = &parsed_payload.roomName;
        let desc = &parsed_payload.localDesc.unwrap();

        if let Some(room) = channels.get_mut(room_name) {
            let room_trans = &room.broadcaster.transmiter;
            if !room_trans.is_closed() {
                let peer_connection = viewer_peer(room, desc.clone(), tx.clone()).await;

                let viewer_meta_data = ViewerMetaData {
                    transmiter: tx.clone(),
                    viewer_peer: peer_connection,
                };

                room.room_users.push(viewer_meta_data);
                room.room_users
                    .retain(|user_socket| !user_socket.transmiter.is_closed());
                println!("Viewer: {:?} joined room {:?}", addr, room_name);
            // println!("Rooms: {:?}", channels);
            } else {
                channels.remove(room_name);
                println!("Broadcast @ {:?} is OFFLINE", room_name);
                // println!("Rooms: {:?}", channels);
            }
        } else {
            println!("Viewer: {addr}, Could not join room: {room_name}");
            // println!("Rooms: {:?}", channels);
        };
    }
}

pub async fn viewer_peer(
    room: &mut Room,
    desc: RTCSessionDescription,
    tx: Tx,
) -> Arc<RTCPeerConnection> {
    let local_track = &room.broadcaster.track_channel_rx;

    let peer_connection = create_peer_connection().await;
    let rtp_sender = peer_connection
        .add_track(Arc::clone(&local_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await
        .unwrap();

    // Read incoming RTCP packets
    // Before these packets are returned they are processed by interceptors. For things
    // like NACK this needs to be called.
    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
        Result::<()>::Ok(())
    });

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");
        Box::pin(async {})
    }));

    let mut gathering_complete_rx = handle_candidate_event(peer_connection.clone()).await;

    // Set the remote SessionDescription
    peer_connection
        .set_remote_description(desc.clone())
        .await
        .unwrap();

    // Create an answer
    let answer = peer_connection.create_answer(None).await.unwrap();

    // Sets the Localdescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await.unwrap();

    let cand_recv = gathering_complete_rx.recv().await.unwrap();

    peer_connection.add_ice_candidate(cand_recv).await.unwrap();

    let local_desc = peer_connection.local_description().await.unwrap();

    let payload = json!(local_desc);

    let answer = Message::Text(payload.to_string());

    if let Err(err) = tx.unbounded_send(answer.clone()) {
        eprintln!("Failed to send message to recipient: {} {:?}", err, tx);
    }

    peer_connection
}
