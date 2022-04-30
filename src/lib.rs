pub mod check;
pub mod commands;
pub mod database;
pub mod info;
pub mod listener;
use commands::lobby::helper::LobbyEvent;
pub use entity::sea_orm_active_enums::*;
use hashbrown::HashMap;
use parking_lot::RwLock;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::MessageComponentInteraction;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub struct Data {
    pub db: &'static DatabaseConnection,
    // Hashmap to store lobby ids with their task's channel handle
    pub active_lobbies: RwLock<HashMap<String, UnboundedSender<LobbyEvent>>>,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct EventComponent {
    message_component_interaction: MessageComponentInteraction,
    http_client: Arc<serenity::http::client::Http>,
}

impl EventComponent {
    pub fn new(
        message_component_interaction: MessageComponentInteraction,
        http_client: Arc<serenity::http::client::Http>,
    ) -> Self {
        Self {
            message_component_interaction,
            http_client,
        }
    }
}
