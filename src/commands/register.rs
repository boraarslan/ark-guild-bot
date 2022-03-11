use crate::*;
use super::*;


#[poise::command(prefix_command, hide_in_help, required_permissions = "ADMINISTRATOR")]
pub async fn register_guild(ctx: Context<'_>, name: String, #[flag] global: bool) -> Result<(), Error> {
    if let Err(err) = poise::builtins::register_application_commands(ctx, global).await {
        ctx.say(format!("Failed to register application commands, {err}"))
            .await?;
    }

    if let Some(_) = get_server(ctx.guild_id().expect("No guild id").0) {
        remove_server(ctx.guild_id().expect("No guild id").0).expect("Failed to remove server");
    }
    let connection = establish_connection();

    let new_server = Server {
        id: ctx.guild_id().expect("No guild id").0.to_string(),
        guild_name: name,
    };

    diesel::insert_into(servers::table)
        .values(&new_server)
        .execute(&connection)
        .expect("Error saving new server");
    Ok(())
}