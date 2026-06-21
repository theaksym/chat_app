use std::rc::Rc;

use anyhow::Ok;
use chat_app::ClientLocalData;
use slint::{VecModel, Weak, include_modules};
use tokio::{spawn, sync::mpsc};

include_modules!();

pub struct Interface {
    window: MainWindow,
    sender: mpsc::Sender<ClientLocalData>,
}
impl Interface {
    pub async fn new(
        sender: mpsc::Sender<ClientLocalData>,
        receiver: mpsc::Receiver<ClientLocalData>,
    ) -> anyhow::Result<Self> {
        let window = MainWindow::new()?;

        let sender_clone = sender.clone();

        let weak_window = window.as_weak();

        spawn(async move { Self::recv_loop(sender_clone, receiver, weak_window) });

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
    fn set_callbacks(&mut self) {}
    fn test(&mut self) {
        let messages = Rc::new(VecModel::from(vec![
            UserMessageData {
                name: "1.".into(),
                content: "Then it just becomes what it is.".into(),
            },
            UserMessageData {
                name: "2.".into(),
                content: "I don't know man, that's like one of the dumbest things I've ever heard."
                    .into(),
            },
            UserMessageData {
                name: "1.".into(),
                content: "But hey, they all had a choice.".into(),
            },
            UserMessageData {
                name: "2.".into(),
                content: "Well, I don't even know what the fuck you're talking about.".into(),
            },
        ]));
        self.window.set_messages(messages.into());

        let rooms = Rc::new(VecModel::from(vec![
            RoomData {
                name: "hut of lies".into(),
                id: "1234".into(),
            },
            RoomData {
                name: "red room".into(),
                id: "1234".into(),
            },
            RoomData {
                name: "pig den".into(),
                id: "1234".into(),
            },
            RoomData {
                name: "flesh chamber".into(),
                id: "1234".into(),
            },
            RoomData {
                name: "the lean years".into(),
                id: "1234".into(),
            },
            RoomData {
                name: "excellent servant".into(),
                id: "1234".into(),
            },
            RoomData {
                name: "terrible master".into(),
                id: "1234".into(),
            },
        ]));
        self.window.set_rooms(rooms.into());
    }
    async fn recv_loop(
        sender: mpsc::Sender<ClientLocalData>,
        mut receiver: mpsc::Receiver<ClientLocalData>,
        window: Weak<MainWindow>,
    ) -> anyhow::Result<()> {
        while let Some(data) = receiver.recv().await {}

        Ok(())
    }
}
