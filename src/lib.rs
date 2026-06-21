// client and server shared stuff goes here

use anyhow::Ok;
use iroh::{
    EndpointAddr,
    endpoint::{RecvStream, SendStream},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const ALPN: &[u8] = b"chat_alpn";

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
pub enum ServerLocalData {}

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
