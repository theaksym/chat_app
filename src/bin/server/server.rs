use anyhow::Ok;
use chat_app::{ALPN, ClientNetworkData, ServerLocalData, ServerNetworkData, send_data};
use iroh::{
    Endpoint, EndpointAddr,
    endpoint::{VarInt, presets},
    protocol::Router,
};
use iroh_ping::Ping;
use iroh_tickets::endpoint::EndpointTicket;
use tokio::sync::mpsc;

use crate::{database::get_room, protocol::ServerProtocol};

pub struct Server {
    sender: mpsc::Sender<ServerLocalData>,
    receiver: mpsc::Receiver<ServerLocalData>,

    router: Router,
    ticket: EndpointTicket,
}
impl Server {
    pub async fn new(
        app_sender: mpsc::Sender<ServerLocalData>,
        protocol_sender: mpsc::Sender<ServerLocalData>,
        receiver: mpsc::Receiver<ServerLocalData>,
    ) -> anyhow::Result<Self> {
        let protocol = ServerProtocol::new(protocol_sender);

        let endpoint = Endpoint::builder(presets::N0).bind().await?;

        let ticket = EndpointTicket::new(endpoint.addr());

        let ping = Ping::new();

        let router = Router::builder(endpoint)
            .accept(ALPN, protocol)
            .accept(iroh_ping::ALPN, ping)
            .spawn();

        Ok(Self {
            sender: app_sender,
            receiver,

            router,
            ticket,
        })
    }
    pub async fn start(&mut self) -> anyhow::Result<()> {
        println!("Ticket: {}", self.ticket);

        self.sender
            .send(ServerLocalData::ServerTicket(self.ticket.clone()))
            .await?;

        self.recv_loop().await
    }
    async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.router.shutdown().await?;
        self.receiver.close();

        Ok(())
    }
    async fn recv_loop(&mut self) -> anyhow::Result<()> {
        while let Some(data) = self.receiver.recv().await {
            match data {
                ServerLocalData::Shutdown => self.shutdown().await?,
                ServerLocalData::Joined(addr, room_id) => {
                    self.sender
                        .send(ServerLocalData::Joined(addr.clone(), room_id.clone()))
                        .await?;

                    self.send_data(addr, ClientNetworkData::JoinAccepted(room_id.clone()))
                        .await?;
                }
                ServerLocalData::Left(addr, room_id) => {
                    self.sender
                        .send(ServerLocalData::Left(addr, room_id))
                        .await?;
                }
                ServerLocalData::SendMessages(addr, messages) => {
                    self.send_data(addr, ClientNetworkData::ReceiveMessages(messages))
                        .await?
                }
                ServerLocalData::HandleAddRoomRequest(addr, room_id) => {
                    if let Some(room_data) = get_room(&room_id)? {
                        self.send_data(addr, ClientNetworkData::RoomAdded(room_data))
                            .await?;
                    }
                }
                ServerLocalData::MessageReceived(data) => {
                    self.sender
                        .send(ServerLocalData::MessageReceived(data))
                        .await?
                }
                _ => {}
            }
        }

        Ok(())
    }
    async fn send_data(
        &mut self,
        addr: EndpointAddr,
        data: ClientNetworkData,
    ) -> anyhow::Result<()> {
        let conn = self.router.endpoint().connect(addr, ALPN).await?;
        let (mut send_stream, _) = conn.open_bi().await?;

        send_data(&mut send_stream, data).await?;

        conn.closed().await;
        conn.close(VarInt::from_u32(0), b"bleh");

        Ok(())
    }
}
