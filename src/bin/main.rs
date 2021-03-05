use std::convert::Infallible;
use std::sync::{Arc, Mutex, MutexGuard};
use warp::Filter;
use warp_ships::db_models::{Ship, DB};

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
                .map_err(|_e| warp::reject::not_found())
                .and_then(|db: MutexGuard<DB>| {
                    db.list_ships(None).map_err(|_e| warp::reject::not_found())
                })
                .map(|all_ships: Vec<Ship>| warp::reply::json::<Vec<Ship>>(&all_ships))
        })
        .recover(|_e: warp::Rejection| async move {
            Ok::<_, Infallible>(warp::reply::with_status(
                "Error",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        });
    let ships = warp::path("ships").and(all_ships);
    warp::serve(ships).run(([127, 0, 0, 1], 4000)).await;
}

type SharableDB = Arc<Mutex<DB>>;
