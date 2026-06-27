use std::{rc::Rc, time::Duration};

use anyhow::Ok;
use arboard::Clipboard;
use chat_app::{RoomData, ServerLocalData};
use iroh_tickets::endpoint::EndpointTicket;
use slint::{Model, ToSharedString, VecModel, Weak, include_modules, invoke_from_event_loop};
use tokio::{spawn, sync::mpsc};

use crate::database::get_all_rooms;

include_modules!();

pub struct Interface {
    window: ServerWindow,
    sender: mpsc::Sender<ServerLocalData>,
}
impl Interface {
    pub async fn new(
        sender: mpsc::Sender<ServerLocalData>,
        receiver: mpsc::Receiver<ServerLocalData>,
    ) -> anyhow::Result<Self> {
        let window = ServerWindow::new()?;

        let sender_clone = sender.clone();

        let weak_window = window.as_weak();

        spawn(async move { Self::recv_loop(sender_clone, receiver, weak_window).await });

        Ok(Self { window, sender })
    }
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.set_callbacks();
        self.load_rooms_from_database().await?;

        // self.test();

        self.window.run()?;

        self.shutdown().await
    }
    async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.sender.send(ServerLocalData::Shutdown).await?;
        self.sender.closed().await;

        Ok(())
    }
    fn set_callbacks(&mut self) {
        self.window.on_copy_ticket(move |input| {
            spawn(async move {
                if Self::copy_ticket(&input).await.is_err() {
                    let _ = slint::invoke_from_event_loop(move || {
                        let _ = TicketCopyFailure::new().unwrap().run();
                    });
                }
            });
        });

        let sender_clone = self.sender.clone();

        self.window.on_create_room(move |input| {
            let clone_again = sender_clone.clone();

            spawn(async move {
                let _ = clone_again
                    .send(ServerLocalData::CreateRoomRequest(input.to_string()))
                    .await;
            });
        });

        let sender_clone = self.sender.clone();

        self.window.on_delete_room(move |input| {
            let zarazkurwaoszaleje = sender_clone.clone();

            spawn(async move {
                let _ = zarazkurwaoszaleje
                    .send(ServerLocalData::DeleteRoomRequest(input.to_string()))
                    .await;
            });
        });
    }
    fn test(&mut self) {
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
        sender: mpsc::Sender<ServerLocalData>,
        mut receiver: mpsc::Receiver<ServerLocalData>,
        weak: Weak<ServerWindow>,
    ) -> anyhow::Result<()> {
        while let Some(data) = receiver.recv().await {
            match data {
                ServerLocalData::ServerTicket(ticket) => {
                    let clone = weak.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        Self::set_server_ticket(ticket, clone)
                    });
                }
                ServerLocalData::AddRoomUI(data) => {
                    let weak_clone = weak.clone();
                    let _ = invoke_from_event_loop(move || {
                        Self::add_room_ui(data, weak_clone).unwrap();
                    });
                }
                ServerLocalData::RemoveRoomUI(data) => {
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
    fn add_room_ui(data: RoomData, weak: Weak<ServerWindow>) -> anyhow::Result<()> {
        let Some(window) = weak.upgrade() else {
            return Ok(());
        };
        let new_room = UiRoomData {
            id: data.id.to_shared_string(),
            name: data.name.to_shared_string(),
        };

        let temp_rooms = window.get_rooms();
        let Some(rooms) = temp_rooms.as_any().downcast_ref::<VecModel<UiRoomData>>() else {
            return Ok(());
        };

        rooms.push(new_room);

        Ok(())
    }
    fn remove_room_ui(id: String, weak: Weak<ServerWindow>) -> anyhow::Result<()> {
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
    fn set_server_ticket(ticket: EndpointTicket, weak: Weak<ServerWindow>) {
        if let Some(window) = weak.upgrade() {
            window.set_ticket(ticket.to_shared_string());
        }
    }
    async fn copy_ticket(ticket: &str) -> anyhow::Result<()> {
        let mut clipboard = Clipboard::new()?;

        clipboard.set_text(ticket)?;

        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }
}
