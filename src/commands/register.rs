use entity::sea_orm_active_enums::Role;

use super::*;

#[poise::command(
    prefix_command,
    slash_command,
    hide_in_help,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn register_guild(
    ctx: Context<'_>,
    #[description = "Name of the guild"] name: String,
    #[description = "Timezone of the guild"]
    #[min = -23]
    #[max = 23]
    utc_timezone: i32,
) -> Result<(), Error> {
    let db = &ctx.data().db;

    if (get_server(ctx.guild_id().expect("No guild id").0, db).await).is_ok() {
        remove_server(ctx.guild_id().expect("No guild id").0, db)
            .await
            .expect("Failed to remove server");
    }

    insert_server(
        ctx.guild_id().expect("No guild id").0,
        &name,
        utc_timezone,
        db,
    )
    .await?;
    insert_guildmate(
        ctx.guild_id().expect("No guild id").0,
        ctx.author().id.0,
        Role::GuildMaster,
        db,
    )
    .await?;
    ctx.say("Added you as a guildmaster!").await?;

    Ok(())
}

#[poise::command(prefix_command, hide_in_help, required_permissions = "ADMINISTRATOR")]
pub async fn register_commands(
    ctx: Context<'_>,
    #[description = "Will be removed"]
    #[flag]
    global: bool,
) -> Result<(), Error> {
    if let Err(err) = poise::builtins::register_application_commands(ctx, global).await {
        ctx.say(format!("Failed to register application commands, {err}"))
            .await?;
    }

    Ok(())
}
