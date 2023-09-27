use crate::{event_handlers::handle_candidate_event, types::Tx};
use std::sync::Arc;
use webrtc::peer_connection::{
    peer_connection_state::RTCPeerConnectionState, sdp::session_description::RTCSessionDescription,
    RTCPeerConnection,
};

use super::send_payload;

pub async fn peer_processor(
    peer_connection: Arc<RTCPeerConnection>,
    desc: RTCSessionDescription,
    tx: Tx,
) -> Arc<RTCPeerConnection> {
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");
        Box::pin(async {})
    }));

    let mut gathering_complete_rx = handle_candidate_event(peer_connection.clone()).await;

    peer_connection.set_remote_description(desc).await.unwrap();
    let answer = peer_connection.create_answer(None).await.unwrap();

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection
        .set_local_description(answer.clone())
        .await
        .unwrap();

    let cand_recv = gathering_complete_rx.recv().await.unwrap();
    peer_connection.add_ice_candidate(cand_recv).await.unwrap();

    let local_desc = peer_connection.local_description().await.unwrap();

    send_payload(local_desc, tx).await;

    peer_connection
}
