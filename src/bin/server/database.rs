use chat_app::{RoomData, SERVER_DATA_PATH, SERVER_DIR_PATH, UserMessageData};
use rand::random_range;
use sqlite::State;
use std::fs::{File, create_dir_all};

pub fn set_up_local_data() -> anyhow::Result<()> {
    let dir_path = SERVER_DIR_PATH.clone();

    if !dir_path.exists() {
        create_dir_all(dir_path)?;
    }

    let data_path = SERVER_DATA_PATH.clone();

    if !data_path.exists() {
        File::create(data_path)?;
    }

    create_database()?;

    Ok(())
}
pub fn generate_room_id() -> anyhow::Result<String> {
    // this is a very naive way of doing that, but it will do for now.

    let mut id = String::with_capacity(10);

    for _ in 0..id.capacity() {
        let rand_num = random_range(0..=9).to_string();
        id.push_str(&rand_num);
    }

    Ok(id)
}
pub fn create_database() -> anyhow::Result<()> {
    let conn = sqlite::open(SERVER_DATA_PATH.clone())?;

    let query = "
    CREATE TABLE IF NOT EXISTS rooms (id TEXT NOT NULL PRIMARY KEY, name TEXT);
    CREATE TABLE IF NOT EXISTS messages (room_id TEXT, user_name TEXT, content TEXT)
    ";

    conn.execute(query)?;

    Ok(())
}
pub fn insert_room(data: RoomData) -> anyhow::Result<()> {
    let conn = sqlite::open(SERVER_DATA_PATH.clone())?;

    let query = "INSERT INTO rooms VALUES (:id, :name);";

    let mut statement = conn.prepare(query)?;
    statement.bind((":id", data.id.as_str()))?;
    statement.bind((":name", data.name.as_str()))?;

    while let State::Row = statement.next()? {}

    Ok(())
}
pub fn delete_room(id: &str) -> anyhow::Result<()> {
    let conn = sqlite::open(SERVER_DATA_PATH.clone())?;

    let query = "
    DELETE FROM rooms WHERE id = :id;
    DELETE FROM messages WHERE room_id := :id;
    ";

    let mut statement = conn.prepare(query)?;
    statement.bind((":id", id))?;

    while let State::Row = statement.next()? {}

    Ok(())
}
pub fn get_room(id: &str) -> anyhow::Result<Option<RoomData>> {
    let conn = sqlite::open(SERVER_DATA_PATH.clone())?;

    let query = "SELECT * FROM rooms WHERE id = :id";

    let mut statement = conn.prepare(query)?;

    statement.bind((":id", id))?;

    while let State::Row = statement.next()? {
        let id = statement.read::<String, _>("id")?;
        let name = statement.read::<String, _>("name")?;

        let data = RoomData::new(&id, &name);
        return Ok(Some(data));
    }

    Ok(None)
}
pub fn save_message(data: UserMessageData) -> anyhow::Result<()> {
    let conn = sqlite::open(SERVER_DATA_PATH.clone())?;

    let query = "INSERT INTO messages VALUES (:room_id, :user_name, :content);";

    let mut statement = conn.prepare(query)?;
    statement.bind((":room_id", data.room_id.as_str()))?;
    statement.bind((":user_name", data.user_name.as_str()))?;
    statement.bind((":content", data.content.as_str()))?;

    while let State::Row = statement.next()? {}

    Ok(())
}
pub fn get_messages(room_id: &str, amount: usize) -> anyhow::Result<Vec<UserMessageData>> {
    let conn = sqlite::open(SERVER_DATA_PATH.clone())?;

    let query = "SELECT * FROM messages WHERE room_id = :id";

    let mut statement = conn.prepare(query)?;

    statement.bind((":id", room_id))?;

    let mut messages = Vec::with_capacity(amount);

    let mut count = 0;

    while let State::Row = statement.next()?
        && count < amount
    {
        let room_id = statement.read::<String, _>("room_id")?;
        let user_name = statement.read::<String, _>("user_name")?;
        let content = statement.read::<String, _>("content")?;

        let data = UserMessageData::new(&room_id, &user_name, &content);
        messages.push(data);

        count += 1;
    }

    Ok(messages)
}
pub fn get_all_rooms() -> anyhow::Result<Vec<RoomData>> {
    let conn = sqlite::open(SERVER_DATA_PATH.clone())?;

    let query = "SELECT * FROM rooms";

    let mut statement = conn.prepare(query)?;

    let mut rooms = vec![];

    while let State::Row = statement.next()? {
        let id = statement.read::<String, _>("id")?;
        let name = statement.read::<String, _>("name")?;

        let data = RoomData::new(&id, &name);
        rooms.push(data);
    }

    Ok(rooms)
}
