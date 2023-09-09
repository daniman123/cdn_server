use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use serde_json::json;
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::Message;
use webrtc::{
    peer_connection::peer_connection_state::RTCPeerConnectionState,
    rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters},
    track::track_local::{track_local_static_rtp::TrackLocalStaticRTP, TrackLocal},
};

use crate::{
    models::ViewerPayload,
    types::{BroadcastRoomMap, RoomMap, Tx},
    utils::create_peer_connection,
};

pub async fn handle_viewer_role(
    viewer_payload: ViewerPayload,
    // channel_peer_map: &BroadcastRoomMap,
    tx: &Tx,
    addr: SocketAddr,
    local_track: Arc<TrackLocalStaticRTP>,
) {
    let event_type = viewer_payload.eventType;
    let room_name = viewer_payload.roomName;

    // if event_type == "ENTER_ROOM" {
    // if let Some(desc) = viewer_payload.localDesc {
    let desc = viewer_payload.localDesc.unwrap();
    println!("connection: {:?}, joined room: {:?}", addr, room_name);
    let loc_des = desc.clone();
    let peer_connection = create_peer_connection().await;

    // if let Some(local_track) = local_track_chan_rx.recv().await {
    // loop {
    // println!("Viewer {:?}", local_track);

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
    // }
    // }

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");
        Box::pin(async {})
    }));

    // Set the remote SessionDescription
    peer_connection
        .set_remote_description(loc_des.clone())
        .await
        .unwrap();

    // Create an answer
    let answer = peer_connection.create_answer(None).await.unwrap();

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the Localdescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await.unwrap();

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    let local_desc = peer_connection.local_description().await.unwrap();

    let payload = json!(local_desc);

    println!("WE ARE IN THE ");

    let answer = Message::Text(payload.to_string());

    if let Err(err) = tx.unbounded_send(answer.clone()) {
        eprintln!("Failed to send message to recipient: {} {:?}", err, tx);
    }
    // }
}
