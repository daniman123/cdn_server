use futures_util::{future, pin_mut};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::tungstenite::Message;

pub async fn handle_connection(
    channel_peer_map: ClientsMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
) {
    println!("Incoming TCP connection from: {}", addr);

    let (outgoing, incoming) = accept_and_split(raw_stream).await;
    println!("WebSocket connection established: {}", addr);

    let (tx, rx) = mpsc::channel(128);
    channel_peer_map
        .lock()
        .unwrap()
        .insert(addr.clone(), tx.clone());

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
}

async fn broadcast_message(msg: Message) {
    println!("{:?}", msg);
}

// pub async fn payload_extractor(
//     local_track_chan_tx: Sender<Arc<TrackLocalStaticRTP>>,
//     local_track_chan_rx: Receiver<Arc<TrackLocalStaticRTP>>,
//     payload: &Value,
//     channel_peer_map: &BroadcastRoomMap,
//     tx: &Tx,
//     addr: SocketAddr,
// ) {
//     // let (local_track_chan_tx, local_track_chan_rx) =
//     //     tokio::sync::mpsc::channel::<Arc<TrackLocalStaticRTP>>(2);

//     if let Some(Value::Object(obj)) = payload.get("BROADCASTER") {
//         if let Ok(payload) = serde_json::from_value::<BroadCastPayload>(Value::Object(obj.clone()))
//         {
//             let rm =
//                 handle_broadcaster_role(local_track_chan_tx, payload, &channel_peer_map, &tx, addr)
//                     .await
//                     .unwrap();
//             channel_peer_map
//                 .lock()
//                 .unwrap()
//                 .insert("/dashboard/dani".to_string(), rm);
//         }
//     }

//     if let Some(Value::Object(obj)) = payload.get("VIEWER") {
//         if let Ok(payload) = serde_json::from_value::<ViewerPayload>(Value::Object(obj.clone())) {
//             let dees = channel_peer_map.lock().unwrap().clone();
//             let tracks = &dees.get(&payload.roomName).unwrap().broadcast.clone();
//             handle_viewer_role(payload, tx, addr, tracks.to_vec(), local_track_chan_rx).await;
//         }
//     }
// }
