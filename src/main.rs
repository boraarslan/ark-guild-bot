use ark_guild_bot::*;
use ark_guild_bot::{models::Server, schema::servers};
use diesel::prelude::*;
use dotenv::dotenv;
use poise::serenity_prelude as serenity;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(
    prefix_command,
    hide_in_help,
    required_permissions = "ADMINISTRATOR"
)]
async fn register_guild(
    ctx: Context<'_>,
    #[description = "Name of the guild"] name: String,
    #[flag] global: bool,
) -> Result<(), Error> {
    if let Err(_) = poise::builtins::register_application_commands(ctx, global).await {
        ctx.say("Failed to register application commands").await?;
    }

    if let Some(_) = get_server(ctx.guild_id().expect("No guild id").0 as i32) {
        remove_server(ctx.guild_id().expect("No guild id").0 as i32)
            .expect("Failed to remove server");
    }
    let connection = establish_connection();

    let new_server = Server {
        id: ctx.guild_id().expect("No guild id").0 as i32,
        guild_name: name,
    };

    diesel::insert_into(servers::table)
        .values(&new_server)
        .execute(&connection)
        .expect("Error saving new server");
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    poise::Framework::build()
        .token(std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"))
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move {
            Ok(Data {})
        })).options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            commands: vec![
                register_guild()
            ],
            ..Default::default()
        }).run().await.unwrap();
}
