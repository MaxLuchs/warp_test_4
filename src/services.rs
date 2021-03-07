use crate::db_models::{NewShip, Ship, DB};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum ServiceError {
    Unknown,
    DBError,
    NotFound,
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

pub fn remove_ship(db: SharableDB, ship_id: i32) -> Result<Ship, ServiceError> {
    db.lock().map_err(|_| ServiceError::DBError).and_then(|db| {
        db.find_by_id(ship_id)
            .and_then(|result| result.ok_or(ServiceError::Unknown))
            .and_then(|ship| db.delete(ship).map_err(|_| ServiceError::DBError))
    })
}
