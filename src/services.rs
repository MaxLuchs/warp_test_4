use crate::db_models::{NewShip, Ship, DB};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum ServiceError {
    Unknown,
    DBError,
}

impl warp::reject::Reject for ServiceError {}

pub type Service<S>
where
    S: Serialize,
= fn(db: DB) -> Result<S, ServiceError>;

type SharableDB = Arc<Mutex<DB>>;

pub fn get_all_ships(db: SharableDB) -> Result<Vec<Ship>, ServiceError> {
    db.lock()
        .map_err(|_| ServiceError::Unknown)
        .and_then(|db| db.list_ships(None).map_err(|_| ServiceError::DBError))
}

pub fn add_ship(db: SharableDB, new_ship: NewShip) -> Result<Ship, ServiceError> {
    db.lock().map_err(|_| ServiceError::Unknown).and_then(|db| {
        db.insert_new_ship(new_ship)
            .map_err(|_| ServiceError::DBError)
    })
}
