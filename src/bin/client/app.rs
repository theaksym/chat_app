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
    async fn recv_loop(&mut self) -> anyhow::Result<()> {
        while let Some(data) = self.receiver.recv().await {}

        Ok(())
    }
}
