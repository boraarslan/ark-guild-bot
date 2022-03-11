use dotenv::dotenv;
use ark_guild_bot::commands::Data;
use ark_guild_bot::commands::characters::*;
use ark_guild_bot::commands::register::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    poise::Framework::build()
        .token(std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"))
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }))
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
