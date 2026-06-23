use std::fs::{File, create_dir_all};

use anyhow::Ok;
use chat_app::{RoomData, SERVER_DATA_PATH, SERVER_ROOMS_PATH, ServerLocalData, UserMessageData};
use sqlite::State;
use tokio::sync::mpsc;

pub struct App {
    interface_sender: mpsc::Sender<ServerLocalData>,
    server_sender: mpsc::Sender<ServerLocalData>,
    receiver: mpsc::Receiver<ServerLocalData>,
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
        }
    }
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.set_up_local_data().await?;
        self.recv_loop().await
    }
    async fn set_up_local_data(&mut self) -> anyhow::Result<()> {
        let data_path = SERVER_DATA_PATH.clone();

        if !data_path.exists() {
            create_dir_all(data_path)?;
        }

        let rooms_path = SERVER_ROOMS_PATH.clone();

        if !rooms_path.exists() {
            File::create(rooms_path)?;
        }

        create_database()?;

        insert_room(RoomData::new("1234", "hut of lies"))?;
        save_message(UserMessageData::new(
            "1234",
            "2137",
            "we've all been through it in here",
        ))?;
        save_message(UserMessageData::new(
            "1234",
            "2137",
            "we're still human beings",
        ))?;
        save_message(UserMessageData::new("1234", "2137", "dignity!"))?;

        if get_room("1234")?.is_some() {
            println!("got room!!!");
        }

        let messages = get_messages("1234", 2)?;

        for message in messages {
            println!("{}", message.content);
        }

        delete_room("1234")?;

        if get_room("1234")?.is_none() {
            println!("removed room!!!");
        }

        Ok(())
    }
    async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.server_sender.send(ServerLocalData::Shutdown).await?;
        self.server_sender.closed().await; // client closed at this point
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
            }
        }

        Ok(())
    }
}
fn create_database() -> anyhow::Result<()> {
    let conn = sqlite::open(SERVER_ROOMS_PATH.clone())?;

    let query = "
    CREATE TABLE IF NOT EXISTS rooms (id TEXT, name TEXT);
    CREATE TABLE IF NOT EXISTS messages (room_id TEXT, user_id TEXT, content TEXT)
    ";

    conn.execute(query)?;

    Ok(())
}
fn insert_room(data: RoomData) -> anyhow::Result<()> {
    let conn = sqlite::open(SERVER_ROOMS_PATH.clone())?;

    let query = "INSERT INTO rooms VALUES (:id, :name);";

    let mut statement = conn.prepare(query)?;
    statement.bind((":id", data.id.as_str()))?;
    statement.bind((":name", data.name.as_str()))?;

    while let std::result::Result::Ok(State::Row) = statement.next() {}

    Ok(())
}
fn delete_room(id: &str) -> anyhow::Result<()> {
    let conn = sqlite::open(SERVER_ROOMS_PATH.clone())?;

    let query = "
    DELETE FROM rooms WHERE id = :id;
    DELETE FROM messages WHERE room_id := :id;
    ";

    let mut statement = conn.prepare(query)?;
    statement.bind((":id", id))?;

    while let std::result::Result::Ok(State::Row) = statement.next() {}

    Ok(())
}
fn get_room(id: &str) -> anyhow::Result<Option<RoomData>> {
    let conn = sqlite::open(SERVER_ROOMS_PATH.clone())?;

    let query = "SELECT * FROM rooms WHERE id = :id";

    let mut statement = conn.prepare(query)?;

    statement.bind((":id", id))?;

    while let std::result::Result::Ok(State::Row) = statement.next() {
        let id = statement.read::<String, _>("id")?;
        let name = statement.read::<String, _>("name")?;

        let data = RoomData::new(&id, &name);
        return Ok(Some(data));
    }

    Ok(None)
}
fn save_message(data: UserMessageData) -> anyhow::Result<()> {
    let conn = sqlite::open(SERVER_ROOMS_PATH.clone())?;

    let query = "INSERT INTO messages VALUES (:room_id, :user_id, :content);";

    let mut statement = conn.prepare(query)?;
    statement.bind((":room_id", data.room_id.as_str()))?;
    statement.bind((":user_id", data.user_id.as_str()))?;
    statement.bind((":content", data.content.as_str()))?;

    while let std::result::Result::Ok(State::Row) = statement.next() {}

    Ok(())
}
fn get_messages(room_id: &str, amount: usize) -> anyhow::Result<Vec<UserMessageData>> {
    let conn = sqlite::open(SERVER_ROOMS_PATH.clone())?;

    let query = "SELECT * FROM messages WHERE room_id = :id";

    let mut statement = conn.prepare(query)?;

    statement.bind((":id", room_id))?;

    let mut messages = Vec::with_capacity(amount);

    let mut count = 0;

    while let std::result::Result::Ok(State::Row) = statement.next()
        && count < amount
    {
        let room_id = statement.read::<String, _>("room_id")?;
        let user_id = statement.read::<String, _>("user_id")?;
        let content = statement.read::<String, _>("content")?;

        let data = UserMessageData::new(&room_id, &user_id, &content);
        messages.push(data);

        count += 1;
    }

    Ok(messages)
}
