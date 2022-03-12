use super::*;


#[poise::command(prefix_command, hide_in_help, required_permissions = "ADMINISTRATOR")]
pub async fn register_guild(ctx: Context<'_>, name: String, #[flag] global: bool) -> Result<(), Error> {
    if let Err(err) = poise::builtins::register_application_commands(ctx, global).await {
        ctx.say(format!("Failed to register application commands, {err}"))
            .await?;
    }

    let db = &ctx.data().db;

    if let Ok(_) = get_server(ctx.guild_id().expect("No guild id").0, db).await {
        remove_server(ctx.guild_id().expect("No guild id").0, db).await.expect("Failed to remove server");
    }

    insert_server(ctx.guild_id().expect("No guild id").0, &name, db).await?;

    Ok(())
}