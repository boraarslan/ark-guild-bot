use sea_orm::DbErr;
use uuid::Uuid;

use crate::{
    check::is_guild_init, commands::lobby::helper::LobbyEvent, database::get_lobby, Context, Error,
};

#[poise::command(slash_command, guild_only, check = "is_guild_init")]
pub async fn close_lobby(
    ctx: Context<'_>,
    #[description = "ID of the lobby"] lobby_id: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;

    let lobby_id = Uuid::parse_str(&lobby_id);
    if let Err(_) = lobby_id {
        ctx.say("Invalid lobby ID is given").await?;
        return Ok(());
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

    let active_lobbies = ctx.data().active_lobbies.write();

    let channel = match active_lobbies.get(&lobby.lobby_id.to_hyphenated().to_string()) {
        Some(channel) => channel,
        None => {
            drop(active_lobbies);
            ctx.say("Lobby is not tracked.").await?;
            return Ok(());
        }
    };

    if let Err(_) = channel.send(LobbyEvent::CloseLobby(ctx.data().active_lobbies.clone())) {
        ctx.say("An error occured while setting the time.").await?;
        return Ok(());
    }

    Ok(())
}
