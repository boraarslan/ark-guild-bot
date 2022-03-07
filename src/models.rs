use diesel::{Queryable, Insertable};
use crate::schema::*;
#[derive(Queryable, Insertable)]
#[table_name = "servers"]
pub struct Server {
    pub id: i32,
    pub guild_name: String,
}

#[derive(Queryable, Insertable)]
#[table_name="guildmates"]
pub struct Guildmate {
    pub id: i32,
    pub server_id: i32,
}

#[derive(Queryable, Insertable)]
#[table_name="characters"]
pub struct Character {
    pub id: i32,
    pub name: String,
    pub class: String,
    pub item_level: i32,
}