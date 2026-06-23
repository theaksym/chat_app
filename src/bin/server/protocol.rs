use chat_app::ServerLocalData;
use iroh::{
    endpoint::{Connection, VarInt},
    protocol::{AcceptError, ProtocolHandler},
};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct ServerProtocol {
    sender: mpsc::Sender<ServerLocalData>,
}
impl ServerProtocol {
    pub fn new(sender: mpsc::Sender<ServerLocalData>) -> Self {
        Self { sender }
    }
}
impl ProtocolHandler for ServerProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let (mut send_stream, mut recv_stream) = connection.accept_bi().await?;

        connection.close(VarInt::from_u32(0), b"thanks guh");

        std::result::Result::Ok(())
    }
}
