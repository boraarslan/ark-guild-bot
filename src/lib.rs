#[macro_use]
extern crate diesel;
pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use models::*;
use schema::*;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn insert_server(server_id: i32, server_name: &str) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    let new_server = Server {
        id: server_id,
        guild_name: server_name.to_string(),
    };

    diesel::insert_into(servers::table)
        .values(&new_server)
        .execute(&connection)
        .expect("Error saving new server");

    Ok(())
}

pub fn insert_guildmate(server_id: i32, guildmate_id: i32) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    let new_guildmate = Guildmate {
        id: guildmate_id,
        server_id: server_id,
    };

    diesel::insert_into(guildmates::table)
        .values(&new_guildmate)
        .execute(&connection)
        .expect("Error saving new guildmate");

    Ok(())
}

pub fn insert_character(
    character_id: i32,
    character_name: &str,
    character_class: &str,
    character_item_level: i32,
) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    let new_character = Character {
        id: character_id,
        name: character_name.to_string(),
        class: character_class.to_string(),
        item_level: character_item_level,
    };

    diesel::insert_into(characters::table)
        .values(&new_character)
        .execute(&connection)
        .expect("Error saving new character");

    Ok(())
}

pub fn get_server(server_id: i32) -> Option<Server> {
    let connection = establish_connection();

    let server = servers::table
        .filter(servers::id.eq(server_id))
        .get_result(&connection)
        .ok();

    server
}

pub fn get_guildmate(guildmate_id: i32) -> Option<Guildmate> {
    let connection = establish_connection();

    let guildmate = guildmates::table
        .filter(guildmates::id.eq(guildmate_id))
        .get_result(&connection)
        .ok();

    guildmate
}

pub fn get_guildmate_characters(guildmate_id: i32) -> Option<Vec<Character>> {
    let connection = establish_connection();

    let characters = characters::table
        .filter(characters::id.eq(guildmate_id))
        .get_results(&connection)
        .ok();

    characters
}

pub fn get_single_character(character_name: &str) -> Option<Character> {
    let connection = establish_connection();

    let character = characters::table
        .filter(characters::name.eq(character_name))
        .get_result(&connection)
        .ok();

    character
}

// This also deletes all the guildmates and characters associated with the server
pub fn remove_server(server_id: i32) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    diesel::delete(servers::table.filter(servers::id.eq(server_id)))
        .execute(&connection)
        .expect("Error deleting server");

    Ok(())
}

// This also deletes all characters associated with the guildmate
pub fn remove_guildmate(guildmate_id: i32) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    diesel::delete(guildmates::table.filter(guildmates::id.eq(guildmate_id)))
        .execute(&connection)
        .expect("Error deleting guildmate");

    Ok(())
}

pub fn remove_character(character_name: &str) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    diesel::delete(characters::table.filter(characters::name.eq(character_name)))
        .execute(&connection)
        .expect("Error deleting character");

    Ok(())
}