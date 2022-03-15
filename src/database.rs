use entity::prelude::*;
use entity::{characters, guildmates, servers};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter,
    QueryOrder, Set,
};

use super::*;

pub async fn insert_server(
    server_id: u64,
    server_name: &str,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let new_server = servers::ActiveModel {
        id: Set(server_id.to_string()),
        guild_name: Set(server_name.to_string()),
    };

    new_server.insert(db).await?;

    Ok(())
}

pub async fn insert_guildmate(
    server_id: u64,
    guildmate_id: u64,
    guildmate_role: Role,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let new_guildmate = guildmates::ActiveModel {
        id: Set(guildmate_id.to_string()),
        server_id: Set(server_id.to_string()),
        role: Set(guildmate_role),
    };

    new_guildmate.insert(db).await?;

    Ok(())
}

pub async fn insert_character(
    character_id: u64,
    character_name: &str,
    character_class: Class,
    character_item_level: i32,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let new_character = characters::ActiveModel {
        id: Set(character_id.to_string()),
        name: Set(character_name.to_string()),
        class: Set(character_class),
        item_level: Set(character_item_level),
        last_updated: Set(chrono::Utc::now()),
    };

    new_character.insert(db).await?;

    Ok(())
}

/// Gets the server from database.
/// 
/// Returns [`DbErr::RecordNotFound`] if record doesn't exists.
pub async fn get_server(server_id: u64, db: &DatabaseConnection) -> Result<servers::Model, DbErr> {
    Servers::find_by_id(server_id.to_string())
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Couldn't find server.".to_string()))
}

/// Gets the guildmate from database.
/// 
/// Returns [`DbErr::RecordNotFound`] if record doesn't exists.
pub async fn get_guildmate(
    guildmate_id: u64,
    db: &DatabaseConnection,
) -> Result<guildmates::Model, DbErr> {
    Guildmates::find_by_id(guildmate_id.to_string())
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound(
            "Couldn't find guildmate.".to_string(),
        ))
}

/// Gets all the characters from database.
/// 
/// Returns [`DbErr::RecordNotFound`] if record doesn't exists.
pub async fn get_all_characters(
    guildmate_id: u64,
    db: &DatabaseConnection,
) -> Result<Vec<characters::Model>, DbErr> {
    let characters = Characters::find()
        .filter(characters::Column::Id.eq(guildmate_id.to_string()))
        .order_by_desc(characters::Column::ItemLevel)
        .all(db)
        .await?;

    return if characters.len() == 0 {
        Err(DbErr::RecordNotFound(
            "Couldn't find any character.".to_string(),
        ))
    } else {
        Ok(characters)
    };
}

/// Gets a single character from database.
/// 
/// Returns [`DbErr::RecordNotFound`] if record doesn't exists.
pub async fn get_single_character(
    character_name: &str,
    db: &DatabaseConnection,
) -> Result<characters::Model, DbErr> {
    Characters::find_by_id(character_name.to_string())
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound(format!(
            "Couldn't find character named {character_name}."
        )))
}

pub async fn update_character(
    character_name: &str,
    character_class: Class,
    character_item_level: i32,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let existing_character = get_single_character(character_name, db).await;
    match existing_character {
        Ok(existing_character) => {
            let mut existing_character: characters::ActiveModel = existing_character.into();
            existing_character.item_level = Set(character_item_level);
            existing_character.class = Set(character_class);
            existing_character.last_updated = Set(chrono::Utc::now());

            existing_character.update(db).await?;

            Ok(())
        }
        Err(err) => Err(err),
    }
}

pub async fn update_ilvl(
    character_name: &str,
    character_item_level: i32,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let existing_character = get_single_character(character_name, db).await;
    match existing_character {
        Ok(existing_character) => {
            let mut existing_character: characters::ActiveModel = existing_character.into();
            existing_character.item_level = Set(character_item_level);
            existing_character.last_updated = Set(chrono::Utc::now());

            existing_character.update(db).await?;

            Ok(())
        }
        Err(err) => Err(err),
    }
}

// This also deletes all the guildmates and characters associated with the server
pub async fn remove_server(server_id: u64, db: &DatabaseConnection) -> Result<(), DbErr> {
    let _ = get_server(server_id, db).await?.delete(db).await?;

    Ok(())
}

// This also deletes all characters associated with the guildmate
pub async fn remove_guildmate(guildmate_id: u64, db: &DatabaseConnection) -> Result<(), DbErr> {
    let _ = get_guildmate(guildmate_id, db).await?.delete(db).await?;

    Ok(())
}

pub async fn remove_character(character_name: &str, db: &DatabaseConnection) -> Result<(), DbErr> {
    let _ = get_single_character(character_name, db)
        .await?
        .delete(db)
        .await?;

    Ok(())
}
