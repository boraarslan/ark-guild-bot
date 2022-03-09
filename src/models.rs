use crate::schema::*;
use diesel::{Insertable, Queryable};
#[derive(Queryable, Insertable)]
#[table_name = "servers"]
pub struct Server {
    pub id: String,
    pub guild_name: String,
}

#[derive(Queryable, Insertable)]
#[table_name = "guildmates"]
pub struct Guildmate {
    pub id: String,
    pub server_id: String,
}

#[derive(Queryable, Insertable)]
#[table_name = "characters"]
pub struct Character {
    pub id: String,
    pub name: String,
    pub class: String,
    pub item_level: i32,
}
