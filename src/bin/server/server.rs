use anyhow::Ok;
use chat_app::{ALPN, ServerLocalData};
use iroh::{Endpoint, endpoint::presets, protocol::Router};
use iroh_ping::Ping;
use iroh_tickets::endpoint::EndpointTicket;
use tokio::sync::mpsc;

use crate::protocol::ServerProtocol;

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
                _ => {}
            }
        }

        Ok(())
    }
}
