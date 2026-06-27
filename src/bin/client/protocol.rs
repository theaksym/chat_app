use anyhow::Ok;
use chat_app::{ClientLocalData, ClientNetworkData, recv_data};
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

        if let std::result::Result::Ok(Some(data)) =
            recv_data::<ClientNetworkData>(&mut recv_stream).await
        {
            match data {
                ClientNetworkData::ReceiveMessages(messages) => {
                    let _ = self
                        .sender
                        .send(ClientLocalData::ReceiveMessages(messages))
                        .await;
                }
                ClientNetworkData::JoinAccepted(room_id) => {
                    let _ = self
                        .sender
                        .send(ClientLocalData::JoinAccepted(room_id))
                        .await;
                }
                ClientNetworkData::RoomAdded(data) => {
                    let _ = self
                        .sender
                        .send(ClientLocalData::AddRoomAccepted(data))
                        .await;
                }
            }
        }

        connection.close(VarInt::from_u32(0), b"thanks guh");

        std::result::Result::Ok(())
    }
}
