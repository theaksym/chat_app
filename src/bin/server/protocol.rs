use chat_app::{ServerLocalData, ServerNetworkData, recv_data};
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

        if let std::result::Result::Ok(Some(data)) =
            recv_data::<ServerNetworkData>(&mut recv_stream).await
        {
            match data {
                ServerNetworkData::Joined(ticket, room_id) => {
                    println!("Client wants to join room: {}", room_id);
                    let addr = ticket.endpoint_addr().clone();
                    let _ = self
                        .sender
                        .send(ServerLocalData::Joined(addr, room_id))
                        .await;
                }
                ServerNetworkData::Left(ticket, room_id) => {
                    println!("Client left room: {}", room_id);
                    let addr = ticket.endpoint_addr().clone();
                    let _ = self.sender.send(ServerLocalData::Left(addr, room_id)).await;
                }
                ServerNetworkData::AddRoomRequest(ticket, room_id) => {
                    println!("Client wants to add room: {}", room_id);
                    let addr = ticket.endpoint_addr().clone();
                    let _ = self
                        .sender
                        .send(ServerLocalData::HandleAddRoomRequest(addr, room_id))
                        .await;
                }
                ServerNetworkData::MessageSent(data) => {
                    let _ = self
                        .sender
                        .send(ServerLocalData::MessageReceived(data))
                        .await;
                }
                _ => {}
            }
        }

        connection.close(VarInt::from_u32(0), b"thanks guh");

        std::result::Result::Ok(())
    }
}
