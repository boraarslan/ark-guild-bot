use dotenv::dotenv;
use ark_guild_bot::commands::Data;
use ark_guild_bot::commands::characters::*;
use ark_guild_bot::commands::register::*;
use sea_orm::Database;

#[tokio::main]
async fn main() {
    dotenv().ok();
    poise::Framework::build()
        .token(std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"))
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {
            db: Database::connect(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")).await?
        }) }))
        .options(poise::FrameworkOptions {
            commands: vec![
                register_guild(),
                character(),
                list_characters(),
                delete_character(),
                edit_character_ilvl(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
        .await
        .unwrap();
}
