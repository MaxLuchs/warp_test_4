use std::sync::{Arc, Mutex};
use warp::Filter;
use warp_ships::db_models::{Ship, DB};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let db: SharableDB = Arc::new(Mutex::new(DB::new(&db_url)));

    let root = warp::any();
    let get_ships = root
        .and(warp::get())
        .and(warp::any().map(move || {
            let db_clone = db.clone();
            db_clone
        }))
        .map(|db: SharableDB| {
            let db = db.lock().unwrap();
            let all_ships = db.list_ships(None);
            return all_ships
                .map(|all_ships| {
                    return warp::reply::json::<Vec<Ship>>(&all_ships);
                })
                .unwrap();
            //.map_err(|_| warp::reject::reject());
        });
    let ships = warp::path("ships").and(get_ships);
    warp::serve(ships).run(([127, 0, 0, 1], 4000)).await;
}

type SharableDB = Arc<Mutex<DB>>;
