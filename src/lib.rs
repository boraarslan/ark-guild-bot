#[macro_use]
extern crate diesel;
pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenv::dotenv;
use std::{env, fmt::{Display, Debug}};

use models::*;
use schema::*;


#[derive(Debug, poise::ChoiceParameter, Clone, Copy)]
pub enum Class {
    Berserker,
    Paladin,
    Gunlancer,
    Striker,
    Wardancer,
    Scrapper,
    Soulfist,
    Gunslinger,
    Artillerist,
    Deadeye,
    Sharpshooter,
    Bard,
    Sorceress,
    Shadowhunter,
    Deathblade,
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn insert_server(server_id: u64, server_name: &str) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    let new_server = Server {
        id: server_id.to_string(),
        guild_name: server_name.to_string(),
    };

    diesel::insert_into(servers::table)
        .values(&new_server)
        .execute(&connection)
        .expect("Error saving new server");

    Ok(())
}

pub fn insert_guildmate(server_id: u64, guildmate_id: u64) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    let new_guildmate = Guildmate {
        id: guildmate_id.to_string(),
        server_id: server_id.to_string(),
    };

    diesel::insert_into(guildmates::table)
        .values(&new_guildmate)
        .execute(&connection)
        .expect("Error saving new guildmate");

    Ok(())
}

pub fn insert_character(
    character_id: u64,
    character_name: &str,
    character_class: Class,
    character_item_level: i32,
) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    let new_character = Character {
        id: character_id.to_string(),
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

pub fn get_server(server_id: u64) -> Option<Server> {
    let connection = establish_connection();

    let server = servers::table
        .filter(servers::id.eq(server_id.to_string()))
        .get_result(&connection)
        .ok();

    server
}

pub fn get_guildmate(guildmate_id: u64) -> Option<Guildmate> {
    let connection = establish_connection();

    let guildmate = guildmates::table
        .filter(guildmates::id.eq(guildmate_id.to_string()))
        .get_result(&connection)
        .ok();

    guildmate
}

pub fn get_all_characters(guildmate_id: u64) -> Option<Vec<Character>> {
    let connection = establish_connection();

    let characters = characters::table
        .filter(characters::id.eq(guildmate_id.to_string()))
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

pub fn update_character(
    character_name: &str,
    character_class: Class,
    character_item_level: i32,
) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    let existing_character = get_single_character(character_name);
    if let None = existing_character {
        return Err(diesel::result::Error::NotFound);
    }
    let mut existing_character = existing_character.unwrap();
    existing_character.item_level = character_item_level;
    existing_character.class = character_class.to_string();

    diesel::replace_into(characters::table)
        .values(existing_character)
        .execute(&connection)
        .expect("Error updating character");

    Ok(())
}

// This also deletes all the guildmates and characters associated with the server
pub fn remove_server(server_id: u64) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    diesel::delete(servers::table.filter(servers::id.eq(server_id.to_string())))
        .execute(&connection)
        .expect("Error deleting server");

    Ok(())
}

// This also deletes all characters associated with the guildmate
pub fn remove_guildmate(guildmate_id: u64) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    diesel::delete(guildmates::table.filter(guildmates::id.eq(guildmate_id.to_string())))
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