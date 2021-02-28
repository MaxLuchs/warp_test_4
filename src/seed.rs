use eyre::Result;
use warp_ships::db_models::{DB, NewShip};

#[macro_use]
extern crate eyre;

#[macro_use]
extern crate log;

fn main() -> Result<()> {
    pretty_env_logger::init();
    dotenv::dotenv().ok().ok_or_else(|| eyre!("Fck..."));

    let db_url = std::env::var("DATABASE_URL")?;
    let db = DB::new(&db_url);

    let ships = vec![
        NewShip {
            name: "USS Enterprise".to_owned(),
            faction: Some("Starfleet".to_owned()),
            warp_speed: 12,
        },
        NewShip {
            name: "USS Calister".to_owned(),
            faction: Some("Netflix".to_owned()),
            warp_speed: 1,
        },
        NewShip {
            name: "Black Pearl".to_owned(),
            faction: Some("Caribeans".to_owned()),
            warp_speed: 0,
        },
        NewShip {
            name: "USS Voyager".to_owned(),
            faction: Some("Starfleet".to_owned()),
            warp_speed: 6,
        },
    ];

    for ship in ships {
        db.insert_new_ship(ship)?;
    }

    // Check:
    let ships = db.list_ships(None)?;
    for ship in ships {
        info!("Ship inserted : {:?}", ship);
    }

    Ok(())
}