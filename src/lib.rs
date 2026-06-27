// client and server shared stuff goes here

use std::{env, path::PathBuf, sync::LazyLock, time::Duration};

use anyhow::Ok;
use iroh::{
    EndpointAddr,
    endpoint::{RecvStream, SendStream},
};
use iroh_tickets::endpoint::EndpointTicket;
use postcard::{from_bytes, to_stdvec};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const ALPN: &[u8] = b"chat_alpn";
pub const MAX_PING: Duration = Duration::from_millis(500);
pub const CLIENT_DIR_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut buf = PathBuf::new();

    buf.push(env::current_dir().expect("Bad perms"));
    buf.push("client_data");

    buf
});
pub const CLIENT_DATA_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut buf = CLIENT_DIR_PATH.clone();

    buf.push("data.sqlite");

    buf
});
pub const SERVER_DIR_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut buf = PathBuf::new();

    buf.push(env::current_dir().expect("Bad perms"));
    buf.push("server_data");

    buf
});
pub const SERVER_DATA_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut buf = SERVER_DIR_PATH.clone();

    buf.push("data.sqlite");

    buf
});

/// Data meant to be read by the client
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientNetworkData {
    ReceiveMessages(Vec<UserMessageData>),
    JoinAccepted(String),
    RoomAdded(RoomData),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientLocalData {
    Shutdown,
    UserName(String),
    ServerAddr(EndpointAddr),
    AddRoomsUI(Vec<RoomData>),
    RemoveRoomUI(String),
    ChatView,
    ReceiveMessages(Vec<UserMessageData>),
    AddRoomRequest(String),
    AddRoomAccepted(RoomData),
    JoinRequest(String),
    LeaveRequest(String),
    JoinAccepted(String),
    SendMessage(String),
    SendMessageInit(UserMessageData),
}

/// Data meant to be read by the server
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerNetworkData {
    Test,
    Joined(EndpointTicket, String),
    Left(EndpointTicket, String),
    AddRoomRequest(EndpointTicket, String),
    MessageSent(UserMessageData),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerLocalData {
    Shutdown,
    ServerTicket(EndpointTicket),
    CreateRoomRequest(String),
    DeleteRoomRequest(String),
    AddRoomUI(RoomData),
    RemoveRoomUI(String),
    Joined(EndpointAddr, String),
    Left(EndpointAddr, String),
    SendMessages(EndpointAddr, Vec<UserMessageData>),
    ChatView(EndpointAddr),
    JoinAccepted(EndpointAddr, String),
    HandleAddRoomRequest(EndpointAddr, String),
    MessageReceived(UserMessageData),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserMessageData {
    pub room_id: String,
    pub user_name: String,
    pub content: String,
}
impl UserMessageData {
    pub fn new(room_id: &str, user_name: &str, content: &str) -> Self {
        Self {
            room_id: room_id.to_string(),
            user_name: user_name.to_string(),
            content: content.to_string(),
        }
    }
}

pub async fn send_data<T>(stream: &mut SendStream, data: T) -> anyhow::Result<()>
where
    T: Serialize + DeserializeOwned,
{
    let bytes = to_stdvec(&data)?;

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

    let deserialized: T = from_bytes(&data)?;

    Ok(Some(deserialized))
}
