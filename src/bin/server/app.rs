use std::collections::{HashMap, HashSet};

use anyhow::Ok;
use chat_app::{RoomData, ServerLocalData, UserMessageData};
use iroh::EndpointAddr;
use tokio::{spawn, sync::mpsc};

use crate::database::{
    delete_room, generate_room_id, get_messages, get_room, insert_room, save_message,
    set_up_local_data,
};

pub struct App {
    interface_sender: mpsc::Sender<ServerLocalData>,
    server_sender: mpsc::Sender<ServerLocalData>,
    receiver: mpsc::Receiver<ServerLocalData>,

    clients: HashMap<String, HashSet<EndpointAddr>>,
}
impl App {
    pub fn new(
        interface_sender: mpsc::Sender<ServerLocalData>,
        server_sender: mpsc::Sender<ServerLocalData>,
        receiver: mpsc::Receiver<ServerLocalData>,
    ) -> Self {
        Self {
            interface_sender,
            server_sender,
            receiver,
            clients: HashMap::new(),
        }
    }
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.setup()?;
        self.recv_loop().await
    }
    fn setup(&mut self) -> anyhow::Result<()> {
        set_up_local_data()?;

        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.server_sender.send(ServerLocalData::Shutdown).await?;
        self.server_sender.closed().await;
        self.receiver.close();

        Ok(())
    }
    async fn recv_loop(&mut self) -> anyhow::Result<()> {
        while let Some(data) = self.receiver.recv().await {
            match data {
                ServerLocalData::Shutdown => self.shutdown().await?,
                ServerLocalData::ServerTicket(addr) => {
                    self.interface_sender
                        .send(ServerLocalData::ServerTicket(addr))
                        .await?
                }
                ServerLocalData::CreateRoomRequest(name) => {
                    let sender_clone = self.interface_sender.clone();
                    spawn(
                        async move { Self::handle_create_room_request(name, sender_clone).await },
                    );
                }
                ServerLocalData::DeleteRoomRequest(id) => {
                    let sender_clone = self.interface_sender.clone();
                    spawn(async move { Self::handle_delete_room_request(id, sender_clone).await });
                }
                ServerLocalData::Joined(addr, room_id) => {
                    self.on_client_joined(addr, room_id).await?;
                }
                ServerLocalData::Left(addr, room_id) => {
                    if let Some(set) = self.clients.get_mut(&room_id) {
                        set.remove(&addr);
                        println!("Removed addr!!");

                        if set.len() == 0 {
                            self.clients.remove(&room_id);
                        }
                    }
                }
                ServerLocalData::MessageReceived(data) => {
                    self.on_message_received(data).await?;
                }
                _ => {}
            }
        }

        println!("Channel closed!");

        Ok(())
    }
    async fn on_client_joined(
        &mut self,
        addr: EndpointAddr,
        room_id: String,
    ) -> anyhow::Result<()> {
        if !self.clients.contains_key(&room_id) {
            println!("Added addr!!");
            self.clients.insert(room_id.clone(), HashSet::new());
        }

        let addr_clone = addr.clone();

        if let Some(set) = self.clients.get_mut(&room_id) {
            set.insert(addr.clone());
        }

        let sender_clone = self.server_sender.clone();

        spawn(async move { Self::send_messages(sender_clone, addr_clone, room_id).await });

        self.server_sender
            .send(ServerLocalData::ChatView(addr))
            .await?;

        Ok(())
    }
    async fn send_messages(
        sender: mpsc::Sender<ServerLocalData>,
        addr: EndpointAddr,
        room_id: String,
    ) -> anyhow::Result<()> {
        let messages = get_messages(&room_id, 10)?;

        sender
            .send(ServerLocalData::SendMessages(addr, messages))
            .await?;

        Ok(())
    }
    async fn on_message_received(&mut self, data: UserMessageData) -> anyhow::Result<()> {
        if get_room(&data.room_id)?.is_none() {
            return Ok(());
        }

        save_message(data.clone())?;

        let Some(addrs) = self.clients.get(&data.room_id) else {
            return Ok(());
        };

        for addr in addrs {
            self.server_sender
                .send(ServerLocalData::SendMessages(
                    addr.clone(),
                    vec![data.clone()],
                ))
                .await?;
        }

        Ok(())
    }
    async fn handle_create_room_request(
        name: String,
        sender: mpsc::Sender<ServerLocalData>,
    ) -> anyhow::Result<()> {
        let id = generate_room_id()?;

        let data = RoomData::new(&id, &name);

        insert_room(data.clone())?;

        sender.send(ServerLocalData::AddRoomUI(data)).await?;

        Ok(())
    }
    async fn handle_delete_room_request(
        id: String,
        sender: mpsc::Sender<ServerLocalData>,
    ) -> anyhow::Result<()> {
        delete_room(&id)?;

        sender.send(ServerLocalData::RemoveRoomUI(id)).await?;

        Ok(())
    }
}
