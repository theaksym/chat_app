use anyhow::Ok;
use chat_app::{ClientLocalData, UserMessageData};
use tokio::{spawn, sync::mpsc};

use crate::database::{insert_room, set_up_local_data};

pub struct App {
    interface_sender: mpsc::Sender<ClientLocalData>,
    client_sender: mpsc::Sender<ClientLocalData>,
    receiver: mpsc::Receiver<ClientLocalData>,
    user_name: String,
    current_room_id: Option<String>,
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
            user_name: String::new(),
            current_room_id: None,
        }
    }
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.setup()?;
        self.recv_loop().await?;

        Ok(())
    }
    fn setup(&mut self) -> anyhow::Result<()> {
        set_up_local_data()?;

        Ok(())
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
                ClientLocalData::UserName(name) => {
                    self.user_name = name;
                    println!("Username set!!");
                }
                ClientLocalData::ServerAddr(addr) => {
                    self.client_sender
                        .send(ClientLocalData::ServerAddr(addr))
                        .await?
                }
                ClientLocalData::JoinRequest(id) => {
                    if !self.user_name.is_empty() {
                        self.client_sender
                            .send(ClientLocalData::JoinRequest(id))
                            .await?;
                    }
                }
                ClientLocalData::LeaveRequest(_) => {
                    if let Some(id) = self.current_room_id.clone() {
                        self.client_sender
                            .send(ClientLocalData::LeaveRequest(id))
                            .await?;
                    }
                }
                ClientLocalData::ReceiveMessages(messages) => {
                    if messages
                        .iter()
                        .all(|x| Some(x.room_id.clone()) == self.current_room_id)
                    {
                        self.interface_sender
                            .send(ClientLocalData::ReceiveMessages(messages))
                            .await?;
                    }
                }
                ClientLocalData::JoinAccepted(room_id) => {
                    self.interface_sender
                        .send(ClientLocalData::ChatView)
                        .await?;

                    self.current_room_id = Some(room_id);
                }
                ClientLocalData::AddRoomRequest(room_id) => {
                    self.client_sender
                        .send(ClientLocalData::AddRoomRequest(room_id))
                        .await?;
                }
                ClientLocalData::AddRoomAccepted(data) => {
                    if insert_room(data.clone()).is_ok() {
                        self.interface_sender
                            .send(ClientLocalData::AddRoomsUI(vec![data.clone()]))
                            .await?;
                    }
                }
                ClientLocalData::SendMessage(message) => {
                    if !self.user_name.is_empty() {
                        if let Some(room_id) = self.current_room_id.clone() {
                            let data = UserMessageData::new(&room_id, &self.user_name, &message);

                            self.client_sender
                                .send(ClientLocalData::SendMessageInit(data))
                                .await?;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
