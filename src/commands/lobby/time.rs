use sea_orm::DbErr;
use uuid::Uuid;

use crate::{
    check::is_guild_init,
    commands::lobby::helper::LobbyEvent,
    database::{get_lobby, get_server},
    Context, Error,
};

#[poise::command(slash_command, guild_only, check = "is_guild_init")]
async fn change_lobby_time(
    ctx: Context<'_>,
    #[description = "ID of the lobby"] lobby_id: String,
    #[description = "Time you want to set for the lobby."] time: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;

    let lobby_id = Uuid::parse_str(&lobby_id);
    if let Err(_) = lobby_id {
        ctx.say("Invalid lobby ID is given").await?;
        return Ok(());
    }

    let offset = chrono::offset::FixedOffset::east(
        get_server(guild_id, ctx.data().db).await?.timezone as i32 * 3600,
    );
    let time = dateparser::parse_with_timezone(&time, &offset).ok();

    match time {
        None => {
            ctx.send(|m| {
                m.embed(|e| {
                    e.title("Invalid time")
                    .description("Couldn't set lobby time. Either you did not specify a lobby time or the time format is false")
                    .field("Example usage", "`/create_lobby <lobby time>`\n`/create_lobby 6:00pm`\n`/create_lobby May 02, 2021 15:51 UTC+2`", false)
                    .field("\0", "If no time zone is specified the guild time zone is used.", false)
                    .field("\0", format!("Your guild time zone is UTC{offset}"), false)
                })
            }).await?;
            return Ok(());
        }
        Some(lobby_time) => {
            if lobby_time <= (chrono::Utc::now() + chrono::Duration::minutes(15)) {
                ctx.send(|m| {
                    m.embed(|e| {
                        e.title("Lobby time cannot be within 15 minutes")
                        .description(format!("Couldn't set lobby time. Got time: {}",lobby_time))
                        .field("Example usage", "`/create_lobby <lobby time>`\n`/create_lobby 6:00pm`\n`/create_lobby May 02, 2021 15:51 UTC+2`", false)
                        .field("\0", "If no time zone is specified the guild time zone is used.", false)
                        .field("\0", format!("Your guild time zone is UTC{offset}"), false)
                    })
                }).await?;
                return Ok(());
            } else if lobby_time >= (chrono::Utc::now() + chrono::Duration::weeks(2)) {
                ctx.send(|m| {
                    m.embed(|e| {
                        e.title("Lobby time must be within 2 weeks.")
                        .description(format!("Couldn't set lobby time. Got time: {}",lobby_time))
                        .field("Example usage", "`/create_lobby <lobby time>`\n`/create_lobby 6:00pm`\n`/create_lobby May 02, 2021 15:51 UTC+2`", false)
                        .field("\0", "If no time zone is specified the guild time zone is used.", false)
                        .field("\0", format!("Your guild time zone is UTC{offset}"), false)
                    })
                }).await?;
                return Ok(());
            }
        }
    }

    let lobby = get_lobby(lobby_id.unwrap(), ctx.data().db).await;

    match lobby {
        Ok(ref lobby) => {
            if lobby.guild_id.parse::<u64>().unwrap() != guild_id {
                ctx.say("Lobby doesn't belong to this guild.").await?;
                return Ok(());
            } else if !lobby.active {
                ctx.say("Lobby is not active").await?;
                return Ok(());
            }
        }
        Err(DbErr::RecordNotFound(_)) => {
            ctx.say("There is no lobby with the given ID").await?;
            return Ok(());
        }
        Err(_) => {
            ctx.say("A database error has occured. Try again.").await?;
            return Ok(());
        }
    }

    let lobby = lobby.unwrap();
    let time = time.unwrap();
    let event = LobbyEvent::ChangeTime(time, ctx.data().active_lobbies.clone());
    let active_lobbies = ctx.data().active_lobbies.read();

    let channel = match active_lobbies.get(&lobby.lobby_id.to_hyphenated().to_string()) {
        Some(channel) => channel,
        None => {
            drop(active_lobbies);
            ctx.say("Channel is not tracked.").await?;
            return Ok(());
        }
    };

    if let Err(_) = channel.send(event) {
        ctx.say("An error occured while setting the time.").await?;
        return Ok(());
    }

    Ok(())
}
