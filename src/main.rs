use ark_guild_bot::{
    commands::{
        characters::*,
        lobby::{command::*, context::LobbyContext, helper::process_lobby_event},
        register::*,
        Data,
    },
    database::{disable_lobby, get_active_characters_joined, get_active_lobbies},
    info::ContentInfo,
    listener::listener,
    Error, EventComponent,
};
use chrono::Utc;
use dotenv::dotenv;
use hashbrown::HashMap;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use poise::serenity_prelude::{self as serenity, GatewayIntents};
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

pub static DB: OnceCell<DatabaseConnection> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    DB.set(
        Database::connect(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")).await?,
    )
    .unwrap();
    poise::Framework::build()
        .token(std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"))
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    db: DB.get().unwrap(),
                    // active_lobbies: todo!("Init this with database query"),
                    active_lobbies: init_active_lobbies(DB.get().unwrap()).await?,
                })
            })
        })
        .options(poise::FrameworkOptions {
            commands: vec![
                register_guild(),
                register_commands(),
                character(),
                list_characters(),
                delete_character(),
                edit_character_ilvl(),
                create_lobby(),
            ],
            listener: |ctx, event, framework, user_data| {
                Box::pin(listener(ctx, event, framework, user_data))
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .client_settings(move |client_builder| {
            client_builder.intents(GatewayIntents::privileged() | GatewayIntents::non_privileged())
        })
        .intents(serenity::GatewayIntents::all())
        .run()
        .await
        .unwrap();
    Ok(())
}

async fn init_active_lobbies(
    db: &'static DatabaseConnection,
) -> Result<RwLock<HashMap<String, UnboundedSender<EventComponent>>>, DbErr> {
    let mut lobby_map = HashMap::new();
    let active_lobbies = get_active_lobbies(db).await?;
    for lobby in active_lobbies {
        if let Some(time) = lobby.scheduled {
            if time <= Utc::now() {
                disable_lobby(&lobby, db).await?;
                continue;
            } else if time <= Utc::now() + chrono::Duration::minutes(10) {
                todo!("Notify users")
            }
        }

        let (sender, mut reciever) = unbounded_channel();
        lobby_map.insert(lobby.lobby_id.to_hyphenated().to_string(), sender);

        let active_players = get_active_characters_joined(lobby.lobby_id, db).await?;
        tokio::spawn({
            let content_info: &ContentInfo = lobby.content.into();
            let lobby_context_locked = Arc::new(RwLock::new(LobbyContext {
                id: lobby.lobby_id,
                id_as_string: lobby.lobby_id.to_hyphenated().to_string(),
                guild_id: lobby.guild_id.parse().unwrap(),
                channel_id: lobby.channel_id.parse().unwrap(),
                message_id: lobby.message_id.parse().unwrap(),
                lobby_master: lobby.lobby_master.parse().unwrap(),
                state: State::Generated,
                content: Some(content_info.content_type.as_str().into()),
                content_info: Some(content_info),
                lobby_time: lobby.scheduled,
                players: vec![],
                active_players: vec![],
                player_list: vec![],
            }));

            {
                let lobby_context = lobby_context_locked.clone();
                let mut lobby_context = lobby_context.write();
                lobby_context.player_list = vec![
                    "\n*This slot is empty*".to_string();
                    lobby_context.content_info().content_size as usize
                ];
                for char_model in active_players {
                    lobby_context.add_active_player_by_model(char_model);
                }
            }
            println!("Started listening lobby: ({})", lobby.lobby_id);

            async move {
                while let Some(event_c) = reciever.recv().await {
                    match process_lobby_event(event_c, lobby_context_locked.clone(), db).await {
                        Ok(_) => {}
                        Err(err) => {
                            println!("Error processing event: {err}")
                        }
                    }
                }
            }
        });
    }

    Ok(RwLock::from(lobby_map))
}
