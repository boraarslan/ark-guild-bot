use super::*;
use crate::{*, check::is_guild_init};
use entity::characters;
use sea_orm::DbErr;

fn construct_character_list(characters: &Vec<characters::Model>) -> String {
    let mut character_list = String::new();
    character_list.push_str("```");
    character_list.push_str(&format!(
        "{:<15} {:<15}    {}\n",
        "Name", "Class", "Item Level"
    ));
    character_list.push_str(&format!("{:-<15} {:-<15}    {:-<10}\n", "", "", ""));
    for character in characters {
        character_list.push_str(&format!(
            "{:<15} {:<15} -> {:<5} ilvl \n",
            character.name, character.class, character.item_level
        ));
    }
    character_list.push_str("```");
    character_list
}

#[poise::command(slash_command, category = "Character", check = "is_guild_init")]
pub async fn character(
    ctx: Context<'_>,
    #[description = "Name of the character"] character_name: String,
    #[description = "Class of the character"] class: Class,
    #[description = "Item level of the character"]
    #[min = 0]
    #[max = 1490]
    item_level: i32,
) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("You must be in a guild to use this command")
            .await?;
        return Ok(());
    };

    let db = &ctx.data().db;

    match get_guildmate(ctx.author().id.0, guild_id, db).await {
        Ok(_) => {}
        Err(DbErr::RecordNotFound(_)) => {
            ctx.say("Couldn't find guildmate record. Adding discord account as a guildmate first.")
                .await?;
            insert_guildmate(guild_id, ctx.author().id.0, Role::Guildmate, db)
                .await
                .expect("Failed to insert guildmate");
        }
        Err(err) => {
            ctx.say("Failed to acess guildmate.".to_string()).await?;
            println!("{}", err);
            return Ok(());
        }
    }

    match get_single_character(&character_name, guild_id, db).await {
        Ok(_) => {
            update_character(&character_name, guild_id, class, item_level, db)
                .await
                .expect("Error updating character");
            ctx.say(format!("Updated character named **{character_name}** as *{class}* with __{item_level}__ Item Level")).await?;
        }
        Err(DbErr::RecordNotFound(_)) => {
            insert_character(
                ctx.author().id.0,
                guild_id,
                &character_name,
                class,
                item_level,
                db,
            )
            .await
            .expect("Failed to insert character");
            ctx.say(format!("Added **{character_name}** as *{class}* to your characters (__{item_level}__ Item Level)")).await?;
        }
        Err(err) => {
            ctx.say("Failed to get character.".to_string()).await?;
            println!("{}", err);
            return Ok(());
        }
    }

    Ok(())
}

#[poise::command(slash_command, category = "Character", check = "is_guild_init")]
pub async fn list_characters(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("You must be in a guild to use this command")
            .await?;
        return Ok(());
    };

    let db = &ctx.data().db;

    match get_all_characters(ctx.author().id.0, guild_id, db).await {
        Ok(characters) => {
            let character_list = construct_character_list(&characters);
            ctx.send(|m| {
                m.embed(|e| {
                    e.title(format!("Characters of {}", ctx.author().name))
                        .field("Characters:", character_list, false)
                        .thumbnail(ctx.author().avatar_url().unwrap_or_default())
                })
            })
            .await?;
        }
        Err(DbErr::RecordNotFound(_)) => {
            ctx.say("You have no characters.").await?;
            return Ok(());
        }
        Err(err) => {
            ctx.say("Error getting characters from database.").await?;
            println!("{}", err);
        }
    }

    Ok(())
}

#[poise::command(slash_command, track_edits, category = "Character", check = "is_guild_init")]
pub async fn delete_character(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("You must be in a guild to use this command")
            .await?;
        return Ok(());
    };

    let db = &ctx.data().db;

    match get_all_characters(ctx.author().id.0, guild_id, db).await {
        Ok(characters) => {
            let custom_uuid = ctx.id();

            let character_list = construct_character_list(&characters);
            ctx.send(|m| {
                m.embed(|e| {
                    e.title(format!("Characters of {}", ctx.author().name))
                        .field("Characters:", &character_list, false)
                        .thumbnail(ctx.author().avatar_url().unwrap_or_default())
                })
                .components(|c| {
                    c.create_action_row(|r| {
                        r.create_select_menu(|m| {
                            m.placeholder("Select a character to delete".to_string())
                                .options(|o| {
                                    for character in characters {
                                        o.create_option(|option| {
                                            option
                                                .label(&character.name)
                                                .description(format!(
                                                    "{:<15} -> {:<5} ilvl",
                                                    character.class, character.item_level
                                                ))
                                                .value(&character.name)
                                        });
                                    }
                                    o
                                })
                                .custom_id(custom_uuid)
                        })
                    })
                })
            })
            .await?;

            if let Some(mci) = serenity::CollectComponentInteraction::new(ctx.discord())
                .author_id(ctx.author().id)
                .channel_id(ctx.channel_id())
                .timeout(std::time::Duration::from_secs(60))
                .filter(move |mci| mci.data.custom_id == custom_uuid.to_string())
                .await
            {
                remove_character(&mci.data.values[0], guild_id, db)
                    .await
                    .expect("Failed to remove character");
                mci.create_interaction_response(ctx.discord(), |ir| {
                    ir.kind(serenity::model::interactions::InteractionResponseType::UpdateMessage)
                })
                .await?;

                let mut msg = mci.message.clone();
                msg.edit(ctx.discord(), |m| {
                    m.embed(|e| {
                        e.title("Character deleted")
                            .description(format!(
                                "```Deleted {} from your character list.```",
                                mci.data.values[0]
                            ))
                            .thumbnail(ctx.author().avatar_url().unwrap_or_default())
                    })
                    .components(|c| c)
                })
                .await?;
            }
        }
        Err(DbErr::RecordNotFound(_)) => {
            ctx.say("You dont have any characters.").await?;
        }
        Err(err) => {
            ctx.say("Error accessing the database.").await?;
            println!("{}", err);
        }
    }

    Ok(())
}

#[poise::command(slash_command, category = "Character", check = "is_guild_init")]
pub async fn edit_character_ilvl(
    ctx: Context<'_>,
    #[description = "Name of your character"] character_name: String,
    #[description = "Item level of your character"]
    #[min = 0]
    #[max = 1490]
    item_level: i32,
) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("You must be in a guild to use this command")
            .await?;
        return Ok(());
    };
    let db = &ctx.data().db;
    match update_ilvl(&character_name, guild_id, item_level, db).await {
        Ok(()) => {
            ctx.say(format!(
                "Updated {}'s item level to {}",
                character_name, item_level
            ))
            .await?;
        }
        Err(DbErr::RecordNotFound(_)) => {
            ctx.say(format!("No character named {} found", character_name))
                .await?;
        }
        Err(e) => {
            ctx.say(format!(
                "Error updating {}'s item level: {}",
                character_name, e
            ))
            .await?;
        }
    }

    Ok(())
}
