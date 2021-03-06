use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use warp::reply::Json;
use warp::{Filter, Rejection};
use warp_ships::db_models::{Ship, DB};
use warp_ships::services::{get_all_ships, ServiceError};

#[derive(Serialize, Debug, Clone)]
struct ErrorResponse {
    error_message: String,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("Dont know what happened")]
    UnknownError,
    #[error("sthg in the DB didnt work")]
    DBError,
    #[error("service crashed")]
    ServiceError(ServiceError),
}

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
        .unwrap_or(ErrorResponse::new(&AppError::UnknownError));
    return Ok::<Json, Infallible>(warp::reply::json(&response));
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let db: SharableDB = Arc::new(Mutex::new(DB::new(&db_url)));

    let get_ships = warp::get()
        .map(move || db.clone())
        .map(get_all_ships)
        .and_then(|result: Result<Vec<Ship>, ServiceError>| async move {
            result
                .map(|ships| warp::reply::json(&ships))
                .map_err(|error: ServiceError| warp::reject::custom(AppError::ServiceError(error)))
        });
    let ships = warp::path("ships").and(get_ships);

    let root = warp::any().and(ships).recover(to_response);
    warp::serve(root).run(([127, 0, 0, 1], 4000)).await;
}

type SharableDB = Arc<Mutex<DB>>;
