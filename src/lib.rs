// client and server shared stuff goes here

use std::{env, path::PathBuf, sync::LazyLock, time::Duration};

use anyhow::Ok;
use iroh::{
    EndpointAddr,
    endpoint::{RecvStream, SendStream},
};
use iroh_tickets::endpoint::EndpointTicket;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const ALPN: &[u8] = b"chat_alpn";
pub const MAX_PING: Duration = Duration::from_millis(500);

pub const SERVER_DATA_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut buf = PathBuf::new();

    buf.push(env::current_dir().expect("Bad perms"));
    buf.push("data");

    buf
});
pub const SERVER_ROOMS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut buf = SERVER_DATA_PATH.clone();

    buf.push("rooms.sqlite");

    buf
});

/// data meant to be used by the client
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientNetworkData {}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientLocalData {
    Shutdown,
    ServerAddr(EndpointAddr),
}

/// data meant to be used by the server
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerNetworkData {}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerLocalData {
    Shutdown,
    ServerTicket(EndpointTicket),
}

#[derive(Serialize, Deserialize)]
pub struct RoomData {
    pub id: String,
    pub name: String,
}
impl RoomData {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            name: name.to_string(),
            id: id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserMessageData {
    pub room_id: String,
    pub user_id: String,
    pub content: String,
}
impl UserMessageData {
    pub fn new(room_id: &str, user_id: &str, content: &str) -> Self {
        Self {
            room_id: room_id.to_string(),
            user_id: user_id.to_string(),
            content: content.to_string(),
        }
    }
}

pub async fn send_data<T>(stream: &mut SendStream, data: T) -> anyhow::Result<()>
where
    T: Serialize + DeserializeOwned,
{
    let bytes = serde_json::to_vec(&data)?;

    stream.write_u8(bytes.len() as u8).await?;
    stream.write_all(&bytes).await?;

    Ok(())
}

pub async fn recv_data<T>(stream: &mut RecvStream) -> anyhow::Result<Option<T>>
where
    T: Serialize + DeserializeOwned,
{
    let std::result::Result::Ok(len) = stream.read_u8().await else {
        return Ok(None);
    };

    let mut data = vec![0; len as usize];

    stream.read_exact(&mut data).await?;

    let deserialized = serde_json::from_slice(&data)?;

    Ok(Some(deserialized))
}
