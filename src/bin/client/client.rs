use chat_app::ClientLocalData;
use tokio::sync::mpsc;

pub struct Client {
    app_sender: mpsc::Sender<ClientLocalData>,
    protocol_sender: mpsc::Sender<ClientLocalData>,
    receiver: mpsc::Receiver<ClientLocalData>,
}
impl Client {
    pub fn new(
        app_sender: mpsc::Sender<ClientLocalData>,
        send_for_protocol: mpsc::Sender<ClientLocalData>,
        receiver: mpsc::Receiver<ClientLocalData>,
    ) -> Self {
        Self {
            // THIS IS PROBABLY NOT CORRECT
            // WILL FIX LATER
            // LOOK AT OLD CODE.
            // LOOK.
            // LOOK.
            // LOOK.
            // LOOK.
            // LOOK.
            app_sender,
            protocol_sender: send_for_protocol,
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
