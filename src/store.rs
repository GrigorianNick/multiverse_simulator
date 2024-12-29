use crate::{handle::Handle, simulation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use rusqlite::{self, Connection};

pub trait Store<T>
{
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
            let v: String = row.get(0).unwrap();
            Ok(v)
        }) {
            Ok(id) => ret_vec.push(Handle::new_from(&id)),
            Err(_) => ()
        };
        ret_vec
    }

    fn save(&self, val: T) -> Handle {
        let handle = Handle::new();
        let json = serde_json::to_string(&val).expect("Failed to serialize json");
        self.conn.execute("INSERT OR REPLACE INTO data (id, json) VALUES (?1, ?2)", (&handle.id.to_string(), json));
        handle
    }

    fn save_handle(&self, val: &T, handle: Handle) {
        let json = serde_json::to_string(&val).expect("Failed to serialize json");
        self.conn.execute("REPLACE INTO data (id, json) VALUES (?1, ?2)", (&handle.id.to_string(), json));
    }
}

/*pub struct UniverseStoreSQL
{
    conn: rusqlite::Connection,
}

impl UniverseStoreSQL {
    pub fn new() -> UniverseStoreSQL {
        let u = UniverseStoreSQL{
            conn: Connection::open("./universe.sqlite").expect("Failed to open universe.sqlite"),
        };
        u.conn.execute(
        "CREATE TABLE universes (
            id   TEXT PRIMARY KEY,
            json TEXT
        )",
        ());
        u
    }
}

impl<T> Store<T> for UniverseStoreSQL {
    fn get_universe(&self, id: &Uuid) -> Option<simulation::Universe> {
        let mut stmt = self.conn.prepare("SELECT id, json FROM universes WHERE id = ?1").expect("Failed to prep statement");
        let universe = stmt.query_row([id.to_string()], |row| {
            let json: String = row.get(1).expect("Failed to get json");
            let universe: simulation::Universe = serde_json::from_str(json.as_str()).expect("failed to parse json");
            Ok(universe)
        });
        match universe
        {
            Ok(u) => Some(u),
            Err(_) => None
        }
    }

    fn save_universe(&self, universe: &simulation::Universe) {
        let universe_json = serde_json::to_string(&universe).expect("Failed to serialize json");
        self.conn.execute(
            "INSERT OR REPLACE INTO universes (id, json) VALUES (?1, ?2)", (universe.id.to_string(), universe_json));
    }
}*/