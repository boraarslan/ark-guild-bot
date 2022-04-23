use std::str::FromStr;

use entity::{characters, guildmates, lobby_player, servers};
use entity::{lobby, prelude::*};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, ModelTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::commands::lobby::context::LobbyContext;

use super::*;

pub async fn insert_server(
    server_id: u64,
    server_name: &str,
    server_timezone: i32,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let new_server = servers::ActiveModel {
        id: Set(server_id.to_string()),
        guild_name: Set(server_name.to_string()),
        timezone: Set(server_timezone),
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
    character_guild: u64,
    character_name: &str,
    character_class: Class,
    character_item_level: i32,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let new_character = characters::ActiveModel {
        id: Set(character_id.to_string()),
        guild_id: Set(character_guild.to_string()),
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
        .ok_or_else(|| DbErr::RecordNotFound("Couldn't find server.".to_string()))
}

/// Gets the guildmate from database.
///
/// Returns [`DbErr::RecordNotFound`] if record doesn't exists.
pub async fn get_guildmate(
    guildmate_id: u64,
    guild_id: u64,
    db: &DatabaseConnection,
) -> Result<guildmates::Model, DbErr> {
    Guildmates::find_by_id((guildmate_id.to_string(), guild_id.to_string()))
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("Couldn't find guildmate.".to_string()))
}

/// Gets all the characters from database.
///
/// Returns [`DbErr::RecordNotFound`] if record doesn't exists.
pub async fn get_all_characters(
    guildmate_id: u64,
    guild_id: u64,
    db: &DatabaseConnection,
) -> Result<Vec<characters::Model>, DbErr> {
    let characters = Characters::find()
        .filter(characters::Column::Id.eq(guildmate_id.to_string()))
        .filter(characters::Column::GuildId.eq(guild_id.to_string()))
        .order_by_desc(characters::Column::ItemLevel)
        .all(db)
        .await?;

    if characters.is_empty() {
        Err(DbErr::RecordNotFound(
            "Couldn't find any character.".to_string(),
        ))
    } else {
        Ok(characters)
    }
}

pub async fn get_all_character_by_ilvl(
    guildmate_id: u64,
    guild_id: u64,
    item_level: i32,
    db: &DatabaseConnection,
) -> Result<Vec<characters::Model>, DbErr> {
    let characters = Characters::find()
        .filter(characters::Column::Id.eq(guildmate_id.to_string()))
        .filter(characters::Column::GuildId.eq(guild_id.to_string()))
        .filter(characters::Column::ItemLevel.gte(item_level))
        .order_by_desc(characters::Column::ItemLevel)
        .all(db)
        .await?;

    if characters.is_empty() {
        Err(DbErr::RecordNotFound(
            "Couldn't find any character.".to_string(),
        ))
    } else {
        Ok(characters)
    }
}

/// Gets a single character from database.
///
/// Returns [`DbErr::RecordNotFound`] if record doesn't exists.
pub async fn get_single_character(
    character_name: &str,
    character_guild: u64,
    db: &DatabaseConnection,
) -> Result<characters::Model, DbErr> {
    Characters::find_by_id((character_guild.to_string(), character_name.to_string()))
        .one(db)
        .await?
        .ok_or_else(|| {
            DbErr::RecordNotFound(format!("Couldn't find character named {character_name}."))
        })
}

pub async fn get_guildmates_by_min_ilvl(
    guild_id: u64,
    item_level: i32,
    db: &DatabaseConnection,
) -> Result<Vec<characters::Model>, DbErr> {
    let characters = Characters::find()
        .filter(characters::Column::GuildId.eq(guild_id.to_string()))
        .filter(characters::Column::ItemLevel.gte(item_level))
        .order_by_desc(characters::Column::ItemLevel)
        .all(db)
        .await?;

    if characters.is_empty() {
        Err(DbErr::RecordNotFound(
            "Couldn't find any character.".to_string(),
        ))
    } else {
        Ok(characters)
    }
}

/// Adds filter for each player id in Vec
pub async fn get_guildmates_by_min_ilvl_filter_out(
    guild_id: u64,
    item_level: i32,
    filtered_out: &Vec<characters::Model>,
    db: &DatabaseConnection,
) -> Result<Vec<characters::Model>, DbErr> {
    let mut condition = Condition::all();

    for filtered_char in filtered_out {
        condition = condition.add(characters::Column::Id.ne(&*filtered_char.id));
    }

    let characters = Characters::find()
        .filter(characters::Column::GuildId.eq(guild_id.to_string()))
        .filter(condition)
        .filter(characters::Column::ItemLevel.gte(item_level))
        .order_by_desc(characters::Column::ItemLevel)
        .all(db)
        .await?;

    if characters.is_empty() {
        Err(DbErr::RecordNotFound(
            "Couldn't find any character.".to_string(),
        ))
    } else {
        Ok(characters)
    }
}

pub async fn update_character(
    character_name: &str,
    character_guild: u64,
    character_class: Class,
    character_item_level: i32,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let existing_character = get_single_character(character_name, character_guild, db).await;
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
    character_guild: u64,
    character_item_level: i32,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let existing_character = get_single_character(character_name, character_guild, db).await;
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
pub async fn remove_guildmate(
    guildmate_id: u64,
    guild_id: u64,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let _ = get_guildmate(guildmate_id, guild_id, db)
        .await?
        .delete(db)
        .await?;

    Ok(())
}

pub async fn remove_character(
    character_name: &str,
    guild_id: u64,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let _ = get_single_character(character_name, guild_id, db)
        .await?
        .delete(db)
        .await?;

    Ok(())
}

pub async fn insert_lobby(
    lobby_context: &LobbyContext,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let mut content_name_retained = lobby_context.content_info().name.clone();

    // EnumString trait defined as to accept only conflated lowercase &str because
    // Display names, .toml file names and database names are slightly different
    // Don't ask why its just bad design.
    // I did not think this project would grow this much
    content_name_retained.retain(|c| !c.is_whitespace());
    let content_name_retained = content_name_retained.to_lowercase();

    let lobby = entity::lobby::ActiveModel {
        lobby_id: Set(lobby_context.id),
        guild_id: Set(lobby_context.guild_id.to_string()),
        channel_id: Set(lobby_context.channel_id.to_string()),
        message_id: Set(lobby_context.message_id.to_string()),
        lobby_master: Set(lobby_context.lobby_master.to_string()),
        content: Set(Content::from_str(content_name_retained.as_str()).unwrap()),
        created: Set(chrono::Utc::now()),
        scheduled: Set(lobby_context.lobby_time),
        active: Set(true),
    };

    lobby.insert(db).await?;

    for player in &lobby_context.active_players {
        insert_lobby_player(lobby_context, player, db).await?;
    }

    Ok(())
}

pub async fn get_lobby(lobby_id: Uuid, db: &DatabaseConnection) -> Result<lobby::Model, DbErr> {
    Lobby::find_by_id(lobby_id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("Couldn't find lobby.".to_string()))
}

/// This might return an empty vec
pub async fn get_active_lobbies(db: &DatabaseConnection) -> Result<Vec<lobby::Model>, DbErr> {
    Lobby::find()
        .filter(lobby::Column::Active.eq(true))
        .all(db)
        .await
}

pub async fn disable_lobby(lobby: &lobby::Model, db: &DatabaseConnection) -> Result<(), DbErr> {
    let mut lobby_a_model: lobby::ActiveModel = lobby.clone().into();
    lobby_a_model.active = Set(false);
    lobby_a_model.update(db).await?;

    let players = get_lobby_players(lobby.lobby_id, db).await?;
    for player in players {
        let mut player: lobby_player::ActiveModel = player.into();
        player.active = Set(false);
        player.update(db).await?;
    }

    Ok(())
}

pub async fn insert_lobby_player(
    lobby_context: &LobbyContext,
    player: &characters::Model,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let lobby_player = lobby_player::ActiveModel {
        lobby_id: Set(lobby_context.id),
        guild_id: Set(lobby_context.guild_id.to_string()),
        player_id: Set(player.id.to_string()),
        character_name: Set(player.name.clone()),
        active: Set(true),
    };

    lobby_player.insert(db).await?;

    Ok(())
}

pub async fn insert_lobby_players(
    lobby_context: &LobbyContext,
    player_list: Vec<characters::Model>,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let models: Vec<lobby_player::ActiveModel> = player_list
        .iter()
        .map(|m| lobby_player::ActiveModel {
            lobby_id: Set(lobby_context.id),
            guild_id: Set(lobby_context.guild_id.to_string()),
            player_id: Set(m.id.to_string()),
            character_name: Set(m.name.clone()),
            active: Set(true),
        })
        .collect();

    LobbyPlayer::insert_many(models).exec(db).await?;

    Ok(())
}

pub async fn get_lobby_player(
    lobby_id: Uuid,
    player_name: &str,
    db: &DatabaseConnection,
) -> Result<lobby_player::Model, DbErr> {
    LobbyPlayer::find_by_id((lobby_id, player_name.to_string()))
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("Couldn't find player.".to_string()))
}

pub async fn get_lobby_players(
    lobby_id: Uuid,
    db: &DatabaseConnection,
) -> Result<Vec<lobby_player::Model>, DbErr> {
    LobbyPlayer::find()
        .filter(lobby_player::Column::LobbyId.eq(lobby_id))
        .all(db)
        .await
}

pub async fn remove_lobby_player(
    lobby_id: Uuid,
    player_name: &str,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    let _ = get_lobby_player(lobby_id, player_name, db)
        .await?
        .delete(db)
        .await?;
    Ok(())
}

// I AM FURIOUS
// HOW COME FIND RELATED CHAINS RETURN PERMUTATION OF ALL CHARACTERS HOW IS THIS POSSIBLE
// TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:TODO!:
// FIX THIS MONSTROSITY WHEN YOU LEARN HOW JOIN TABLE WORKS
pub async fn get_active_characters_joined(
    lobby_id: Uuid,
    db: &DatabaseConnection,
) -> Result<Vec<characters::Model>, DbErr> {
    let lobby_players = get_lobby_players(lobby_id, db).await?;
    let mut chars = vec![];

    for player in lobby_players {
        chars.push(
            get_single_character(&player.character_name, player.guild_id.parse().unwrap(), db)
                .await?,
        );
    }

    Ok(chars)
}
