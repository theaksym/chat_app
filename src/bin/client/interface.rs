use std::{rc::Rc, str::FromStr};

use anyhow::Ok;
use chat_app::{ClientLocalData, MAX_PING, RoomData, UserMessageData};
use iroh::{Endpoint, endpoint::presets, protocol::ProtocolHandler};
use iroh_ping::Ping;
use iroh_tickets::endpoint::EndpointTicket;
use slint::{Model, ToSharedString, VecModel, Weak, include_modules, invoke_from_event_loop};
use tokio::{spawn, sync::mpsc, time::timeout};

use crate::database::{get_all_rooms, remove_room};

include_modules!();

pub struct Interface {
    window: ClientWindow,
    sender: mpsc::Sender<ClientLocalData>,
}
impl Interface {
    pub async fn new(
        sender: mpsc::Sender<ClientLocalData>,
        receiver: mpsc::Receiver<ClientLocalData>,
    ) -> anyhow::Result<Self> {
        let window = ClientWindow::new()?;

        let sender_clone = sender.clone();

        let weak_window = window.as_weak();

        spawn(async move { Self::recv_loop(sender_clone, receiver, weak_window).await });

        Ok(Self { window, sender })
    }
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.setup().await?;
        // self.test();

        self.window.run()?;

        self.shutdown().await
    }
    async fn setup(&mut self) -> anyhow::Result<()> {
        self.setup_interface_vars();
        self.set_callbacks();
        self.load_rooms_from_database().await?;

        Ok(())
    }
    async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.sender.send(ClientLocalData::Shutdown).await?;
        self.sender.closed().await;

        Ok(())
    }
    fn setup_interface_vars(&mut self) {
        self.window
            .set_messages(Rc::new(VecModel::from(vec![])).into());

        self.window
            .set_rooms(Rc::new(VecModel::from(vec![])).into());

        self.window.set_room_name("".into());
    }
    fn set_callbacks(&mut self) {
        let sender_clone = self.sender.clone();

        self.window.on_set_user_name(move |input| {
            let clone_again = sender_clone.clone();

            spawn(async move {
                let _ = clone_again
                    .send(ClientLocalData::UserName(input.to_string()))
                    .await;
            });
        });

        let sender_clone = self.sender.clone();

        self.window.on_set_server_ticket(move |input| {
            let clone_again = sender_clone.clone();
            Self::on_set_server_ticket(&input, clone_again)
        });

        let sender_clone = self.sender.clone();

        self.window.on_add_room(move |input| {
            let clone_again = sender_clone.clone();

            spawn(async move {
                clone_again
                    .send(ClientLocalData::AddRoomRequest(input.to_string()))
                    .await
            });
        });

        let sender_clone = self.sender.clone();

        self.window.on_join_room(move |input| {
            println!("Got request! Interface");

            let clone_again = sender_clone.clone();

            spawn(async move {
                let _ = clone_again
                    .send(ClientLocalData::JoinRequest(input.to_string()))
                    .await;
            });
        });

        let weak = self.window.as_weak();

        self.window.on_remove_room(move |input| {
            let clone_again = weak.clone();

            let _ = Self::remove_room(input.to_string(), clone_again);
        });

        let sender_clone = self.sender.clone();

        self.window.on_send_message(move |input| {
            if !input.is_empty() {
                let clone_again = sender_clone.clone();

                spawn(async move {
                    clone_again
                        .send(ClientLocalData::SendMessage(input.to_string()))
                        .await
                });
            }
        });

        let sender_clone = self.sender.clone();

        self.window.on_chat_view_back(move || {
            let clone_again = sender_clone.clone();

            spawn(async move {
                clone_again
                    .send(ClientLocalData::LeaveRequest("".to_string()))
                    .await
            });
        });
    }
    fn on_set_server_ticket(input: &str, sender_clone: mpsc::Sender<ClientLocalData>) {
        if let std::result::Result::Ok(ticket) = EndpointTicket::from_str(&input) {
            let _ = spawn(async move {
                if let std::result::Result::Ok(endpoint) =
                    Endpoint::builder(presets::N0).bind().await
                {
                    let addr = ticket.endpoint_addr();
                    let ping = Ping::new();

                    let _ = match timeout(MAX_PING, ping.ping(&endpoint, addr.clone())).await {
                        std::result::Result::Ok(_) => {
                            sender_clone
                                .send(ClientLocalData::ServerAddr(ticket.endpoint_addr().clone()))
                                .await
                        }
                        std::result::Result::Err(_) => {
                            slint::invoke_from_event_loop(|| {
                                let _ = OfflineServerAddrDialog::new().unwrap().run();
                            })
                            .unwrap();
                            std::result::Result::Ok(())
                        }
                    };

                    ping.shutdown().await;
                    endpoint.close().await;
                }
            });
        }
    }
    fn remove_room(room_id: String, weak: Weak<ClientWindow>) -> anyhow::Result<()> {
        let _ = remove_room(&room_id);

        slint::invoke_from_event_loop(move || {
            let _ = Self::remove_room_ui(room_id.to_string(), weak);
        })?;

        Ok(())
    }
    fn test(&mut self) {
        let messages = Rc::new(VecModel::from(vec![
            UiUserMessageData {
                name: "Turing".into(),
                content: "After all I've done for you".into(),
            },
            UiUserMessageData {
                name: "Turing".into(),
                content: "The countless secrets I laid bare".into(),
            },
            UiUserMessageData {
                name: "Turing".into(),
                content: "This is how you repay me?".into(),
            },
        ]));
        self.window.set_messages(messages.into());

        let rooms = Rc::new(VecModel::from(vec![
            UiRoomData {
                name: "hut of lies".into(),
                id: "1234".into(),
            },
            UiRoomData {
                name: "red room".into(),
                id: "1234".into(),
            },
            UiRoomData {
                name: "whitehouse".into(),
                id: "1234".into(),
            },
            UiRoomData {
                name: "flesh chamber".into(),
                id: "1234".into(),
            },
            UiRoomData {
                name: "the lean years".into(),
                id: "1234".into(),
            },
            UiRoomData {
                name: "excellent servant".into(),
                id: "1234".into(),
            },
            UiRoomData {
                name: "terrible master".into(),
                id: "1234".into(),
            },
        ]));
        self.window.set_rooms(rooms.into());
    }
    async fn recv_loop(
        sender: mpsc::Sender<ClientLocalData>,
        mut receiver: mpsc::Receiver<ClientLocalData>,
        weak: Weak<ClientWindow>,
    ) -> anyhow::Result<()> {
        while let Some(data) = receiver.recv().await {
            match data {
                ClientLocalData::ChatView => {
                    let weak_clone = weak.clone();

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(window) = weak_clone.upgrade() {
                            window.invoke_chat_view();
                        }
                    });
                }
                ClientLocalData::ReceiveMessages(messages) => {
                    let weak_clone = weak.clone();

                    let _ = slint::invoke_from_event_loop(move || {
                        Self::load_messages(messages, weak_clone)
                            .expect("Couldn't load messages to UI !!!!!!!!!");
                    });
                }
                ClientLocalData::AddRoomsUI(data) => {
                    let weak_clone = weak.clone();
                    let _ = invoke_from_event_loop(move || {
                        Self::add_rooms_ui(data, weak_clone).unwrap();
                    });
                }
                ClientLocalData::RemoveRoomUI(data) => {
                    let weak_clone = weak.clone();
                    let _ = invoke_from_event_loop(move || {
                        Self::remove_room_ui(data, weak_clone).unwrap();
                    });
                }
                _ => {}
            }
        }

        Ok(())
    }
    fn load_messages(
        messages: Vec<UserMessageData>,
        weak: Weak<ClientWindow>,
    ) -> anyhow::Result<()> {
        println!("Received messages!!!!!");

        if let Some(window) = weak.upgrade() {
            let temp = window.get_messages();

            let new_messages: Vec<UiUserMessageData> = messages
                .iter()
                .map(|message| UiUserMessageData {
                    content: message.content.to_shared_string(),
                    name: message.user_name.to_shared_string(),
                })
                .collect::<Vec<_>>();

            let temp = window.get_messages();
            let Some(messages) = temp.as_any().downcast_ref::<VecModel<UiUserMessageData>>() else {
                return Ok(());
            };

            for message in new_messages {
                messages.push(message);
            }
        }

        Ok(())
    }
    async fn load_rooms_from_database(&mut self) -> anyhow::Result<()> {
        let rooms = get_all_rooms()?;

        let mut ui_rooms = Vec::with_capacity(rooms.len());

        for room in rooms {
            let ui_room = UiRoomData {
                id: room.id.to_shared_string(),
                name: room.name.to_shared_string(),
            };

            ui_rooms.push(ui_room);
        }

        self.window
            .set_rooms(Rc::new(VecModel::from(ui_rooms)).into());

        Ok(())
    }
    fn add_rooms_ui(data: Vec<RoomData>, weak: Weak<ClientWindow>) -> anyhow::Result<()> {
        let Some(window) = weak.upgrade() else {
            return Ok(());
        };

        let temp_rooms = window.get_rooms();
        let Some(rooms) = temp_rooms.as_any().downcast_ref::<VecModel<UiRoomData>>() else {
            return Ok(());
        };

        for room in data {
            let new_room = UiRoomData {
                id: room.id.to_shared_string(),
                name: room.name.to_shared_string(),
            };

            rooms.push(new_room);
        }

        Ok(())
    }
    fn remove_room_ui(id: String, weak: Weak<ClientWindow>) -> anyhow::Result<()> {
        let Some(window) = weak.upgrade() else {
            return Ok(());
        };

        let temp_rooms = window.get_rooms();
        let Some(rooms) = temp_rooms.as_any().downcast_ref::<VecModel<UiRoomData>>() else {
            return Ok(());
        };

        let index_to_remove = {
            let mut idx = None;
            let mut count = 0;

            for room in rooms.iter() {
                if room.id == id {
                    idx = Some(count);
                }

                count += 1;
            }

            idx
        };

        if let Some(idx) = index_to_remove {
            rooms.remove(idx);
        }

        Ok(())
    }
}
