use std::rc::Rc;

use anyhow::Ok;
use chat_app::ClientLocalData;
use slint::{VecModel, include_modules};
use tokio::sync::mpsc;

include_modules!();

pub struct Interface {
    window: MainWindow,
    sender: mpsc::Sender<ClientLocalData>,
    receiver: mpsc::Receiver<ClientLocalData>,
}
impl Interface {
    pub async fn new(
        sender: mpsc::Sender<ClientLocalData>,
        receiver: mpsc::Receiver<ClientLocalData>,
    ) -> anyhow::Result<Self> {
        let window = MainWindow::new()?;

        Ok(Self {
            window,
            sender,
            receiver,
        })
    }
    pub async fn start(&mut self) -> anyhow::Result<()> {
        // test

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

        self.window.run()?;

        Ok(())
    }
}
