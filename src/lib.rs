#[macro_use]
extern crate diesel;
pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenv::dotenv;
use std::{
    env,
    fmt::{Debug, Display},
};

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

pub fn get_guildmate(guildmate_id: u64) -> Result<Option<Guildmate>, diesel::result::Error> {
    let connection = establish_connection();

    return match guildmates::table
        .filter(guildmates::id.eq(guildmate_id.to_string()))
        .get_result(&connection)
    {
        Ok(character) => Ok(Some(character)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(err) => Err(err),
    };
}

pub fn get_all_characters(
    guildmate_id: u64,
) -> Result<Option<Vec<Character>>, diesel::result::Error> {
    let connection = establish_connection();

    return match characters::table
        .filter(characters::id.eq(guildmate_id.to_string()))
        .order(characters::item_level.desc())
        .get_results(&connection)
    {
        Ok(characters) => Ok(Some(characters)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(err) => Err(err),
    };
}

pub fn get_single_character(
    character_name: &str,
) -> Result<Option<Character>, diesel::result::Error> {
    let connection = establish_connection();

    return match characters::table
        .filter(characters::name.eq(character_name))
        .get_result(&connection)
    {
        Ok(character) => Ok(Some(character)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(err) => Err(err),
    };
}

pub fn update_character(
    character_name: &str,
    character_class: Class,
    character_item_level: i32,
) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    let existing_character = get_single_character(character_name);
    match existing_character {
        Ok(existing_character) => {
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
        Err(err) => Err(err),
    }
}

pub fn update_ilvl(
    character_name: &str,
    character_item_level: i32,
) -> Result<(), diesel::result::Error> {
    let connection = establish_connection();

    let existing_character = get_single_character(character_name);
    match existing_character {
        Ok(existing_character) => {
            if let None = existing_character {
                return Err(diesel::result::Error::NotFound);
            }
            let mut existing_character = existing_character.unwrap();
            existing_character.item_level = character_item_level;

            diesel::replace_into(characters::table)
                .values(existing_character)
                .execute(&connection)
                .expect("Error updating character");

            Ok(())
        }
        Err(err) => Err(err),
    }
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
