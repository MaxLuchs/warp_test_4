use crate::schema;
use diesel::prelude::*;
use eyre::Result;
use serde::Serialize;

#[derive(Debug, Queryable, Insertable, Identifiable, Serialize)]
pub struct Ship {
    id: i32,
    name: String,
    warp_speed: i32,
    faction: Option<String>,
}

use schema::ships;
use std::ops::Deref;

#[derive(Debug, Insertable, Clone)]
#[table_name = "ships"]
pub struct NewShip {
    pub name: String,
    pub warp_speed: i32,
    pub faction: Option<String>,
}

pub struct DB {
    con: Box<diesel::SqliteConnection>,
}

impl DB {
    pub fn new(db_url: &str) -> DB {
        let con = diesel::sqlite::SqliteConnection::establish(db_url).expect("DB not connected");
        DB { con: Box::new(con) }
    }

    pub fn insert_new_ship(&self, new_ship: NewShip) -> Result<Ship> {
        let id = uuid::Uuid::new_v4().as_u128() as i32;
        let ship_n = new_ship.clone();
        let inserted = diesel::insert_into(schema::ships::table)
            .values(Ship {
                id,
                warp_speed: ship_n.warp_speed,
                faction: ship_n.faction,
                name: ship_n.name,
            })
            .execute(self.con.deref())?;
        info!("inserted {:?} : {:?}", inserted, &new_ship);
        return Ok(schema::ships::table
            .filter(schema::ships::name.eq(&new_ship.name))
            .order(schema::ships::id.desc())
            .first::<Ship>(self.con.deref())?);
    }

    pub fn delete(&self, ship: Ship) -> Result<Ship> {
        let ship_to_delete = schema::ships::table
            .find(&ship.id)
            .first::<Ship>(self.con.deref())?;
        let deleted = diesel::delete(schema::ships::table.filter(schema::ships::id.eq(ship.id)))
            .execute(self.con.deref())?;
        info!("deleted {:?} : {:?}", deleted, ship_to_delete);
        return Ok(ship_to_delete);
    }

    pub fn list_ships(&self, name: Option<String>) -> Result<Vec<Ship>> {
        if let Some(n) = name {
            return Ok(schema::ships::table
                .filter(schema::ships::name.like(format!("%{}%", n)))
                .get_results::<Ship>(self.con.deref())?);
        }
        return Ok(schema::ships::table.get_results::<Ship>(self.con.deref())?);
    }
}
