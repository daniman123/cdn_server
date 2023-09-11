use crate::models::Room;
use futures_channel::mpsc::UnboundedSender;
use futures_util::stream::{SplitSink, SplitStream};
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use webrtc::peer_connection::RTCPeerConnection;

pub type Tx = UnboundedSender<Message>;
pub type PeerConn = Arc<RTCPeerConnection>;
pub type ClientsMap = Arc<Mutex<HashMap<String, Room>>>;
pub type WsSplit = (
    SplitSink<WebSocketStream<TcpStream>, Message>,
    SplitStream<WebSocketStream<TcpStream>>,
);
