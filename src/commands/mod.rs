use poise::serenity_prelude as serenity;
use sea_orm::DatabaseConnection;
use crate::database::*;
pub mod characters;
pub mod register;
pub mod lobby;

pub struct Data {
    pub db: DatabaseConnection
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;