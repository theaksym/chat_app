use chat_app::{CLIENT_DATA_PATH, CLIENT_DIR_PATH, RoomData};
use sqlite::State;
use std::fs::{File, create_dir_all};

pub fn set_up_local_data() -> anyhow::Result<()> {
    let dir_path = CLIENT_DIR_PATH.clone();

    if !dir_path.exists() {
        create_dir_all(dir_path)?;
    }

    let data_path = CLIENT_DATA_PATH.clone();

    if !data_path.exists() {
        File::create(data_path)?;
    }

    create_database()?;

    Ok(())
}
pub fn create_database() -> anyhow::Result<()> {
    let conn = sqlite::open(CLIENT_DATA_PATH.clone())?;

    let query = "
    CREATE TABLE IF NOT EXISTS rooms (id TEXT NOT NULL PRIMARY KEY, name TEXT);
    ";

    conn.execute(query)?;

    Ok(())
}
pub fn insert_room(data: RoomData) -> anyhow::Result<()> {
    let conn = sqlite::open(CLIENT_DATA_PATH.clone())?;

    let query = "INSERT INTO rooms VALUES (:id, :name);";

    let mut statement = conn.prepare(query)?;
    statement.bind((":id", data.id.as_str()))?;
    statement.bind((":name", data.name.as_str()))?;

    while let State::Row = statement.next()? {}

    Ok(())
}
pub fn remove_room(id: &str) -> anyhow::Result<()> {
    let conn = sqlite::open(CLIENT_DATA_PATH.clone())?;

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
    let conn = sqlite::open(CLIENT_DATA_PATH.clone())?;

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
pub fn get_all_rooms() -> anyhow::Result<Vec<RoomData>> {
    let conn = sqlite::open(CLIENT_DATA_PATH.clone())?;

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
