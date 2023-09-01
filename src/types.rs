use futures_channel::mpsc::UnboundedSender;
use futures_util::stream::{SplitSink, SplitStream};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::models::Room;

pub type Tx = UnboundedSender<Message>;

// pub type Tx = tokio::sync::broadcast::Sender<Message>;

pub type BroadcastRoomMap = Arc<Mutex<RoomMap>>;
pub type RoomMap = HashMap<RoomName, Room>;

pub type WsSplit = (
    SplitSink<WebSocketStream<TcpStream>, Message>,
    SplitStream<WebSocketStream<TcpStream>>,
);

pub type RoomName = String;
