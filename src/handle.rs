use std::str::FromStr;

use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::store::Store;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Handle {
    pub id: uuid::Uuid,
}

impl Handle {
    // Creates a new, empty handle
    pub fn new() -> Handle {
        Handle {
            id: uuid::Uuid::new_v4(),
        }
    }

    pub fn new_from(id: &String) -> Handle{
        let mut h = Handle::new();
        h.id = Uuid::from_str(id).unwrap();
        h
    }

    // convience getter, doesn't both checking cache
    pub fn get<T>(&self, store: impl Store<T>) -> Option<T> {
        store.get(&self)
    }
}