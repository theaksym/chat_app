use anyhow::Ok;
use chat_app::{ALPN, ClientLocalData};
use iroh::{Endpoint, EndpointAddr, endpoint::presets, protocol::Router};
use tokio::sync::mpsc;

use crate::protocol::ClientProtocol;

pub struct Client {
    app_sender: mpsc::Sender<ClientLocalData>,
    receiver: mpsc::Receiver<ClientLocalData>,

    server_addr: Option<EndpointAddr>,
    router: Router,
}
impl Client {
    pub async fn new(
        app_sender: mpsc::Sender<ClientLocalData>,
        protocol_sender: mpsc::Sender<ClientLocalData>,
        receiver: mpsc::Receiver<ClientLocalData>,
    ) -> anyhow::Result<Self> {
        let protocol = ClientProtocol::new(protocol_sender);

        let endpoint = Endpoint::builder(presets::N0).bind().await?;

        let router = Router::builder(endpoint).accept(ALPN, protocol).spawn();

        Ok(Self {
            app_sender,
            receiver,

            server_addr: None,
            router,
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
                    println!("Server address set!");
                }
            }
        }

        Ok(())
    }
}
