use log::info;
use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use warp::reply::Json;
use warp::{Filter, Rejection, Reply};
use warp_ships::db_models::{ListShipsFilter, NewShip, Ship, DB};
use warp_ships::services::{add_ship, get_all_ships, remove_ship, ServiceError};

#[macro_use]
extern crate log;

#[derive(Serialize, Debug, Clone)]
struct ErrorResponse {
    error_message: String,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("Dont know what happened : {0}")]
    UnknownError(String),
    #[error("sthg in the DB didnt work")]
    DBError,
    #[error("service crashed")]
    ServiceError(ServiceError),
    #[error("invalid body")]
    BodyParserError,
    #[error("Stuff not found")]
    NotFound,
}

use std::collections::HashMap;
use std::io::Bytes;
use std::ops::Deref;
use warp::http::Method;
use warp::reject::Reject;

impl Reject for AppError {}

impl ErrorResponse {
    fn new(error: &AppError) -> ErrorResponse {
        ErrorResponse {
            error_message: format!("{}", *error),
        }
    }
}

impl warp::reject::Reject for ErrorResponse {}

async fn to_response(rejection: Rejection) -> Result<Json, Infallible> {
    let response = rejection
        .find::<AppError>()
        .map(|err: &AppError| ErrorResponse::new(err))
        .unwrap_or(ErrorResponse::new(&AppError::UnknownError(format!(
            "{:?}",
            rejection
        ))));
    return Ok::<Json, Infallible>(warp::reply::json(&response));
}

#[macro_use]
extern crate warp;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let db: SharableDB = Arc::new(Mutex::new(DB::new(&db_url)));

    let get_db = Box::new(move || db.clone());

    let get_ships = warp::path::end()
        .and(warp::get())
        .map(get_db.clone())
        .and(warp::query())
        .map(|db: SharableDB, params: ListShipsFilter| get_all_ships(db, params))
        .and_then(|result: Result<Vec<Ship>, ServiceError>| async move {
            result
                .map(|ships| warp::reply::json(&ships))
                .map_err(|error: ServiceError| warp::reject::custom(AppError::ServiceError(error)))
        });

    let add_ship = warp::post()
        .and(warp::body::json())
        .and(warp::any().map(get_db.clone()))
        .and(warp::any().map(|| "Huhu was geht".to_owned()))
        .and_then(
            |new_ship: NewShip, db: SharableDB, text: String| async move {
                info!("-> text : {}", text);
                let result: Result<Ship, Rejection> = add_ship(db, new_ship)
                    .map_err(|e| warp::reject::custom(AppError::ServiceError(e)));
                result
            },
        )
        .map(|ships| warp::reply::json(&ships));

    let remove_ship = warp::delete()
        .and(warp::path::param::<i32>())
        .and(warp::any().map(get_db.clone()))
        .and_then(|id: i32, db: SharableDB| async move {
            info!("id: {}", id);
            remove_ship(db, id)
                .map(|ship| warp::reply::json(&ship))
                .map_err(|e| warp::reject::custom(e))
        });

    let ships = warp::path("ships").and(get_ships.or(remove_ship).or(add_ship));
    let test = warp::get()
        .map(|| "Hallo".to_owned())
        .map(|text: String| format!("{} da drüben", text))
        .map(|text: String| warp::reply::with_status(text, warp::http::StatusCode::OK));

    let root = warp::any().and(ships.or(test)).recover(to_response);
    warp::serve(root).run(([127, 0, 0, 1], 4000)).await;
}

type SharableDB = Arc<Mutex<DB>>;
