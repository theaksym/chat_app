use anyhow::Ok;
use chat_app::ClientLocalData;
use iroh::{
    endpoint::{Connection, VarInt},
    protocol::{AcceptError, ProtocolHandler},
};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct ClientProtocol {
    sender: mpsc::Sender<ClientLocalData>,
}
impl ClientProtocol {
    pub fn new(sender: mpsc::Sender<ClientLocalData>) -> Self {
        Self { sender }
    }
}
impl ProtocolHandler for ClientProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let (mut send_stream, mut recv_stream) = connection.accept_bi().await?;

        connection.close(VarInt::from_u32(0), b"thanks guh");

        std::result::Result::Ok(())
    }
}
