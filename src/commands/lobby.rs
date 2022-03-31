use std::fmt::Display;

use parse_display::Display;
use poise::serenity_prelude::{CollectComponentInteraction, CreateButton};
use sea_orm::DbErr;

use super::*;
use crate::*;

#[derive(Display)]
enum Content {
    #[display("Guardian Raid")]
    GuardianRaid,
    #[display("Abyss Dungeon")]
    AbyssDungeon,
    #[display("Abyss Raid")]
    AbyssRaid,
}

impl From<&str> for Content {
    fn from(val: &str) -> Self {
        match val {
            "guardian-raid" => Self::GuardianRaid,
            "abyss-dungeon" => Self::AbyssDungeon,
            "abyss-raid" => Self::AbyssRaid,
            _ => unreachable!()
        }
    }
}


#[poise::command(slash_command, category = "Lobby")]
pub async fn create_lobby(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("You must be in a guild to use this command")
            .await?;
        return Ok(());
    };

    let db = &ctx.data().db;

    let lobby_master = match get_guildmate(ctx.author().id.0, db).await {
        Ok(guildmate) => {
            if guildmate.role == Role::Guildmate {
                ctx.say("Only guild administration can create lobbies.")
                    .await?;
                return Ok(());
            }
            ctx.author()
        }
        Err(DbErr::RecordNotFound(_)) => {
            ctx.say("You are not registered in guild.").await?;
            return Ok(());
        }
        Err(_) => {
            ctx.say("Error getting guildmate record from database.")
                .await?;
            return Ok(());
        }
    };

    let custom_id = ctx.id();

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Please select the content.")
            //TODO!: Add fields for each content
        })
        .components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.style(serenity::ButtonStyle::Danger)
                        .custom_id(custom_id.to_string() + "-guardian-raid")
                        .label("Guardian Raid")
                });
                r.create_button(|b| {
                    b.style(serenity::ButtonStyle::Primary)
                        .custom_id(custom_id.to_string() + "-abyss-dungeon")
                        .label("Abyss Dungeon")
                });
                r.create_button(|b| {
                    b.style(serenity::ButtonStyle::Primary)
                        .custom_id(custom_id.to_string() + "-abyss-raid")
                        .label("Abyss Raid")
                })
            })
        })
    })
    .await?;

    if let Some(mci) = CollectComponentInteraction::new(ctx.discord())
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(600))
        .filter(move |mci| {
            mci.data
                .custom_id
                .starts_with(custom_id.to_string().as_str())
        })
        .await
    {
        let content = Content::from(mci.data.custom_id.get(custom_id.to_string().len()..mci.data.custom_id.len()).unwrap());
        mci.message.clone().edit(ctx.discord(), |m| {
            m.embed(|e|{
                e.title(format!("Select a {}", content))
            })
        }).await?;

    }

    while let Some(mci) = CollectComponentInteraction::new(ctx.discord())
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(600))
        .filter(move |mci| mci.data.custom_id == custom_id.to_string())
        .await
    {
    }

    Ok(())
}
