use crate::{
    helpers::peer_helpers::peer_processor,
    models::{Room, ViewerMetaData, ViewerPayload},
    types::{ClientsMap, Tx},
    utils::create_peer_connection,
};
use anyhow::Result;
use std::{net::SocketAddr, sync::Arc};
use webrtc::{
    peer_connection::{sdp::session_description::RTCSessionDescription, RTCPeerConnection},
    track::track_local::TrackLocal,
};

pub async fn process_viewer(
    parsed_payload: ViewerPayload,
    channel_peer_map: ClientsMap,
    tx: Tx,
    addr: SocketAddr,
) {
    let mut channels = channel_peer_map.lock().await;

    let room_name = &parsed_payload.roomName;
    let event_type = &parsed_payload.eventType;

    if event_type == "ENTER_ROOM" {
        if let Some(desc) = parsed_payload.localDesc {
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
                } else {
                    channels.remove(room_name);
                    println!("Broadcast @ {:?} is OFFLINE", room_name);
                }
            } else {
                println!("Viewer: {addr}, Could not join room: {room_name}");
            }
        }
    }
}

async fn viewer_peer(
    room: &mut Room,
    desc: RTCSessionDescription,
    tx: Tx,
) -> Arc<RTCPeerConnection> {
    let local_track = &room.broadcaster.track_channel_rx;

    let peer_connection = create_peer_connection().await;

    let track_1 = local_track.get(0).unwrap();
    let track_2 = local_track.get(1).unwrap();

    peer_connection
        .add_track(Arc::clone(track_1) as Arc<dyn TrackLocal + Send + Sync>)
        .await
        .unwrap();

    let rtp_sender = peer_connection
        .add_track(Arc::clone(track_2) as Arc<dyn TrackLocal + Send + Sync>)
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

    peer_processor(peer_connection.clone(), desc, tx.clone()).await
}
