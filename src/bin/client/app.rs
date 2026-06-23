use anyhow::Ok;
use chat_app::ClientLocalData;
use tokio::sync::mpsc;

pub struct App {
    interface_sender: mpsc::Sender<ClientLocalData>,
    client_sender: mpsc::Sender<ClientLocalData>,
    receiver: mpsc::Receiver<ClientLocalData>,
}
impl App {
    pub fn new(
        interface_sender: mpsc::Sender<ClientLocalData>,
        client_sender: mpsc::Sender<ClientLocalData>,
        receiver: mpsc::Receiver<ClientLocalData>,
    ) -> Self {
        Self {
            interface_sender,
            client_sender,
            receiver,
        }
    }
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.recv_loop().await
    }
    async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.client_sender.send(ClientLocalData::Shutdown).await?;
        self.client_sender.closed().await; // client closed at this point
        self.receiver.close();

        Ok(())
    }
    async fn recv_loop(&mut self) -> anyhow::Result<()> {
        while let Some(data) = self.receiver.recv().await {
            match data {
                ClientLocalData::Shutdown => self.shutdown().await?,
                ClientLocalData::ServerAddr(addr) => {
                    self.client_sender
                        .send(ClientLocalData::ServerAddr(addr))
                        .await?
                }
            }
        }

        Ok(())
    }
}
