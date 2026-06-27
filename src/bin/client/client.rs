use anyhow::Ok;
use chat_app::{ALPN, ClientLocalData, ServerNetworkData, send_data};
use iroh::{
    Endpoint, EndpointAddr,
    endpoint::{VarInt, presets},
    protocol::Router,
};
use iroh_ping::Ping;
use iroh_tickets::endpoint::EndpointTicket;
use tokio::sync::mpsc;

use crate::protocol::ClientProtocol;

pub struct Client {
    sender: mpsc::Sender<ClientLocalData>,
    receiver: mpsc::Receiver<ClientLocalData>,

    server_addr: Option<EndpointAddr>,
    router: Router,
    ticket: EndpointTicket,
}
impl Client {
    pub async fn new(
        app_sender: mpsc::Sender<ClientLocalData>,
        protocol_sender: mpsc::Sender<ClientLocalData>,
        receiver: mpsc::Receiver<ClientLocalData>,
    ) -> anyhow::Result<Self> {
        let protocol = ClientProtocol::new(protocol_sender);

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

            server_addr: None,
            router,
            ticket,
        })
    }
    pub async fn start(&mut self) -> anyhow::Result<()> {
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
                ClientLocalData::Shutdown => self.shutdown().await?,
                ClientLocalData::ServerAddr(addr) => {
                    self.server_addr = Some(addr);
                }
                ClientLocalData::JoinRequest(id) => {
                    self.send_data(ServerNetworkData::Joined(self.ticket.clone(), id))
                        .await?
                }
                ClientLocalData::LeaveRequest(id) => {
                    self.send_data(ServerNetworkData::Left(self.ticket.clone(), id))
                        .await?
                }
                ClientLocalData::ReceiveMessages(messages) => {
                    self.sender
                        .send(ClientLocalData::ReceiveMessages(messages))
                        .await?
                }
                ClientLocalData::JoinAccepted(room_id) => {
                    self.sender
                        .send(ClientLocalData::JoinAccepted(room_id))
                        .await?;
                }
                ClientLocalData::AddRoomRequest(room_id) => {
                    self.send_data(ServerNetworkData::AddRoomRequest(
                        self.ticket.clone(),
                        room_id,
                    ))
                    .await?
                }
                ClientLocalData::AddRoomAccepted(data) => {
                    self.sender
                        .send(ClientLocalData::AddRoomAccepted(data))
                        .await?;
                }
                ClientLocalData::SendMessageInit(data) => {
                    self.send_data(ServerNetworkData::MessageSent(data)).await?
                }
                _ => {}
            }
        }

        Ok(())
    }
    async fn send_data(&mut self, data: ServerNetworkData) -> anyhow::Result<()> {
        let Some(addr) = self.server_addr.clone() else {
            return Ok(());
        };

        let conn = self.router.endpoint().connect(addr, ALPN).await?;
        let (mut send_stream, _) = conn.open_bi().await?;

        send_data(&mut send_stream, data).await?;

        conn.closed().await;
        conn.close(VarInt::from_u32(0), b"bleh");

        Ok(())
    }
}
