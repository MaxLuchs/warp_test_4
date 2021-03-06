use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, MutexGuard};
use thiserror::Error;
use warp::reply::Json;
use warp::{Filter, Rejection};
use warp_ships::db_models::{Ship, DB};

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
    ServiceError,
}

use warp::reject::Reject;
impl Reject for AppError {}

impl ErrorResponse {
    fn new(error: &AppError) -> ErrorResponse {
        ErrorResponse {
            error_message: format!("{:?}", error),
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

    let root = warp::any();
    let all_ships = root
        .and(warp::get())
        .and(warp::any().map(move || db.clone()))
        .and_then(|db: SharableDB| async move {
            db.lock()
                .map_err(|_| warp::reject::custom(AppError::DBError))
                .and_then(|db: MutexGuard<DB>| {
                    db.list_ships(None)
                        .map_err(|_| warp::reject::custom(AppError::ServiceError))
                })
                .map(|all_ships: Vec<Ship>| warp::reply::json::<Vec<Ship>>(&all_ships))
        });
    let ships = warp::path("ships").and(all_ships);
    let root = warp::any().and(ships).recover(to_response);
    warp::serve(root).run(([127, 0, 0, 1], 4000)).await;
}

type SharableDB = Arc<Mutex<DB>>;
