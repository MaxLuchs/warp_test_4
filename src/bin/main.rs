use serde::Serialize;
use std::convert::Infallible;
use std::sync::{Arc, Mutex, MutexGuard};
use warp::reply::Json;
use warp::{Filter, Rejection};
use warp_ships::db_models::{Ship, DB};

#[derive(Serialize, Debug, Clone)]
struct ServiceError {
    service_name: String,
    error_message: String,
}

impl ServiceError {
    fn new(service_name: String, error: String) -> ServiceError {
        ServiceError {
            service_name,
            error_message: error.to_string(),
        }
    }
}

impl warp::reject::Reject for ServiceError {}

async fn handle_rejection(rejection: Rejection) -> Result<Json, Infallible> {
    let response = rejection
        .find::<ServiceError>()
        .map(|err| err.to_owned())
        .unwrap_or(ServiceError {
            service_name: "unknown".to_owned(),
            error_message: "unknown error".to_owned(),
        });
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
                .map_err(|e| {
                    warp::reject::custom(ServiceError::new("db".to_owned(), format!("{}", e)))
                })
                .and_then(|db: MutexGuard<DB>| {
                    db.list_ships(None).map_err(|e| {
                        warp::reject::custom(ServiceError::new(
                            "list_ships".to_owned(),
                            format!("{:?}", e),
                        ))
                    })
                })
                .map(|all_ships: Vec<Ship>| warp::reply::json::<Vec<Ship>>(&all_ships))
        });
    let ships = warp::path("ships").and(all_ships);
    let root = warp::any().and(ships).recover(handle_rejection);
    warp::serve(root).run(([127, 0, 0, 1], 4000)).await;
}

type SharableDB = Arc<Mutex<DB>>;
