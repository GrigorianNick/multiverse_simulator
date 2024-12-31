use crate::{handle::Handle, simulation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use rusqlite::{self, Connection};

pub trait Store<T>
{
    fn delete_handle(&self, handle: Handle);
    fn get(&self, handle: &Handle) -> Option<T>;
    fn get_handles(&self) -> Vec<Handle>;
    fn save(&self, val: T) -> Handle;
    fn save_handle(&self, val: &T, handle: Handle);
}

pub struct StoreSQL
{
    conn: rusqlite::Connection,
}

impl StoreSQL {
    pub fn new(path: String) -> Self {
        let c = Connection::open(path).expect("Failed to open sqlite file");
        let _ = c.execute("CREATE TABLE data (
            id STRING PRIMARY KEY,
            json TEXT NOT NULL)", ());
        StoreSQL{
            conn: c,
        }
    }
}

impl<T: Serialize + DeserializeOwned> Store<T> for StoreSQL {
    fn get(&self, handle: &Handle) -> Option<T> {
        let mut stmt = self.conn.prepare("SELECT id, json FROM data WHERE id = ?1").expect("Failed to prepare SQL statement");
        match stmt.query_row([handle.id.to_string()], |row| {
            let json: String = row.get(1).expect("Failed to get json");
            let v: T = serde_json::from_str(&json.as_str()).expect("Failed to parse json");
            Ok(v)
        }) {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    fn get_handles(&self) -> Vec<Handle> {
        let mut ret_vec = vec![];
        let mut stmt = self.conn.prepare("SELECT id FROM data").expect("Failed to prepare SQL statement");
        match stmt.query_row([], |row| {
            println!("Got a row from a database");
            let v: String = row.get(0).unwrap();
            Ok(v)
        }) {
            Ok(id) => ret_vec.push(Handle::new_from(&id)),
            Err(_) => ()
        };
        ret_vec
    }

    fn save(&self, val: T) -> Handle {
        println!("Saving handle");
        let handle = Handle::new();
        let json = serde_json::to_string(&val).expect("Failed to serialize json");
        println!("Inserted: {}", self.conn.execute("INSERT INTO data (id, json) VALUES (?1, ?2)", (&handle.id.to_string(), json)).expect("Failed to create row"));
        handle
    }

    fn save_handle(&self, val: &T, handle: Handle) {
        let json = serde_json::to_string(&val).expect("Failed to serialize json");
        self.conn.execute("REPLACE INTO data (id, json) VALUES (?1, ?2)", (&handle.id.to_string(), json));
    }

    fn delete_handle(&self, handle: Handle) {
        self.conn.execute("DELETE FROM data WHERE id == ?1", [&handle.id.to_string()]);
    }
}