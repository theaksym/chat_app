use std::{rc::Rc, time::Duration};

use anyhow::Ok;
use arboard::Clipboard;
use chat_app::ServerLocalData;
use iroh_tickets::endpoint::EndpointTicket;
use slint::{ToSharedString, VecModel, Weak, include_modules};
use tokio::{spawn, sync::mpsc};

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
        self.test();

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
                _ => {}
            }
        }

        Ok(())
    }
    fn set_server_ticket(ticket: EndpointTicket, weak: Weak<ServerWindow>) {
        println!("Ran!!!!");

        if let Some(window) = weak.upgrade() {
            println!("Upgraded!!!");
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
