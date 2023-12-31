pub mod broadcaster;
pub mod viewer;
use self::{broadcaster::process_broadcaster, viewer::process_viewer};
use crate::{
    models::{BroadCastPayload, ViewerPayload},
    types::{ClientsMap, Tx},
};
use serde_json::Value;
use std::net::SocketAddr;

trait DeserializeOwned: Sized {
    fn variant_key() -> &'static str;
    fn event_type() -> &'static str;
}

impl DeserializeOwned for BroadCastPayload {
    fn variant_key() -> &'static str {
        "BROADCASTER"
    }

    fn event_type() -> &'static str {
        "START_BROADCAST"
    }
}

impl DeserializeOwned for ViewerPayload {
    fn variant_key() -> &'static str {
        "VIEWER"
    }

    fn event_type() -> &'static str {
        "ENTER_ROOM"
    }
}

async fn process_payload(
    payload: serde_json::Map<std::string::String, Value>,
    channel_peer_map: ClientsMap,
    addr: SocketAddr,
    tx: Tx,
) {
    if let Some(Value::Object(raw_payload)) = payload.get(BroadCastPayload::variant_key()) {
        if let Ok(parsed_payload) =
            serde_json::from_value::<BroadCastPayload>(Value::Object(raw_payload.clone()))
        {
            process_broadcaster(
                parsed_payload,
                tx.clone(),
                addr.clone(),
                channel_peer_map.clone(),
            )
            .await;
        }
    }
    if let Some(Value::Object(raw_payload)) = payload.get(ViewerPayload::variant_key()) {
        if let Ok(parsed_payload) =
            serde_json::from_value::<ViewerPayload>(Value::Object(raw_payload.clone()))
        {
            process_viewer(
                parsed_payload,
                channel_peer_map.clone(),
                tx.clone(),
                addr.clone(),
            )
            .await;
        }
    }
}

pub async fn define_socket_user_role(
    payload: serde_json::Map<std::string::String, Value>,
    channel_peer_map: ClientsMap,
    addr: SocketAddr,
    tx: Tx,
) {
    process_payload(payload.clone(), channel_peer_map.clone(), addr, tx).await;
}
