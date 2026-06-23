use std::{rc::Rc, str::FromStr};

use anyhow::Ok;
use chat_app::{ClientLocalData, MAX_PING};
use iroh::{Endpoint, endpoint::presets, protocol::ProtocolHandler};
use iroh_ping::Ping;
use iroh_tickets::endpoint::EndpointTicket;
use slint::{VecModel, Weak, include_modules};
use tokio::{spawn, sync::mpsc, time::timeout};

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
        self.set_callbacks();
        self.test();

        self.window.run()?;

        self.shutdown().await
    }
    async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.sender.send(ClientLocalData::Shutdown).await?;
        self.sender.closed().await;

        Ok(())
    }
    fn set_callbacks(&mut self) {
        let sender_clone = self.sender.clone();

        self.window.on_set_server_ticket(move |input| {
            let clone_again = sender_clone.clone();
            Self::on_set_server_ticket(&input, clone_again)
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
    fn test(&mut self) {
        let messages = Rc::new(VecModel::from(vec![
            UiUserMessageData {
                name: "1.".into(),
                content: "Then it just becomes what it is.".into(),
            },
            UiUserMessageData {
                name: "2.".into(),
                content: "I don't know man, that's like one of the dumbest things I've ever heard."
                    .into(),
            },
            UiUserMessageData {
                name: "1.".into(),
                content: "But hey, they all had a choice.".into(),
            },
            UiUserMessageData {
                name: "2.".into(),
                content: "Well, I don't even know what the fuck you're talking about.".into(),
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
        window: Weak<ClientWindow>,
    ) -> anyhow::Result<()> {
        while let Some(data) = receiver.recv().await {}

        Ok(())
    }
}
