use std::sync::Arc;

use chrono::{DateTime, Utc};
use poise::serenity_prelude::{self as serenity, CreateSelectMenu, Http};
use poise::serenity_prelude::{CreateActionRow, CreateEmbed};
use sea_orm::DatabaseConnection;
use tokio::task::JoinHandle;

use super::command::State;
use super::helper::*;
use crate::database::get_guildmates_by_min_ilvl_filter_out;
use crate::info::*;

/// Represents the lobby.
///
/// Important thing about the component creations is discord only allows [String] values
/// to pass as custom_ids or [serenity::builder::create_components::CreateSelectMenuOptions] values.
/// So as a workaround to filter the component interactions we check if custom id starts with our command's
/// custom id and then extract the last part of the custom id and match it.
/// This requires some string operations every time bot recieves a component interaction thus an allocation.
/// I currently have no idea how much does it cost but I doubt there is a better way since we can only send strings to discord.
pub struct LobbyContext {
    pub id: uuid::Uuid,
    pub id_as_string: String,
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub lobby_master: u64,
    pub state: State,
    pub content: Option<LobbyContent>,
    pub content_info: Option<&'static ContentInfo>,
    pub lobby_time: ( Option<DateTime<Utc>> , Option<JoinHandle<()>>),
    pub players: Vec<entity::characters::Model>,
    pub active_players: Vec<entity::characters::Model>,
    pub player_list: Vec<String>,
    // This field is added when I was writing lobby time change command.
    // I don't know why I did not thought about doing this earlier 
    // (probably because I thought it was not necessary)
    // but since this is added there is no need to send http client through channels
    // and such. I will fix those things later.
    pub http_client: Arc<Http>,
}

impl LobbyContext {
    /// Get a reference to the lobby context's lobby content.
    #[must_use]
    pub fn lobby_content(&self) -> LobbyContent {
        self.content.unwrap()
    }

    /// Get a reference to the lobby context's content info.
    #[must_use]
    pub fn content_info(&self) -> &ContentInfo {
        self.content_info.as_ref().unwrap()
    }

    /// Set the lobby context's content.
    pub fn set_content(&mut self, content: Option<LobbyContent>) {
        self.content = content;
    }

    /// Set the lobby context's content info.
    pub fn set_content_info(&mut self, content_info: Option<&'static ContentInfo>) {
        self.content_info = content_info;
    }

    pub fn create_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed
            .title(format!(
                "{}: {}",
                self.lobby_content(),
                self.content_info().name
            ))
            .description(&self.content_info().introduction)
            .image(&self.content_info().banner)
            .url(&self.content_info().guide)
            .field(
                format!(
                    "Tier {} {} (Minimum Item Level) => {}",
                    self.content_info().tier,
                    self.lobby_content(),
                    self.content_info().ilvl_req
                ),
                format!("Scheduled time: {}", {
                    match self.lobby_time.0 {
                        Some(time) => format!("<t:{0}:R>  (<t:{0}:F>)", time.timestamp()),
                        None => "Not Set".to_owned() + "",
                    }
                }), // This will be a discord timestamp
                true,
            )
            .field("Participating Players:", self.player_list.concat(), false)
            .footer(|foo| foo.text(format!("Lobby id: {}", self.id_as_string)));
        embed
    }

    pub fn create_lobby_buttons(&self) -> CreateActionRow {
        let mut buttons = CreateActionRow::default();

        buttons.create_button(|b| {
            b.label("Post Lobby")
                .style(serenity::ButtonStyle::Success)
                .custom_id(self.id_as_string.clone() + "post-lobby")
        });
        buttons.create_button(|b| {
            b.label("Close Lobby")
                .style(serenity::ButtonStyle::Danger)
                .custom_id(self.id_as_string.clone() + "close-lobby")
        });
        buttons.create_button(|b| {
            b.label("Open Lobby")
                .style(serenity::ButtonStyle::Primary)
                .custom_id(self.id_as_string.clone() + "open-lobby")
        });

        buttons
    }

    pub fn create_user_buttons(&self) -> CreateActionRow {
        let mut buttons = CreateActionRow::default();

        buttons.create_button(|b| {
            b.label("Join Lobby!")
                .style(serenity::ButtonStyle::Success)
                .custom_id(self.id_as_string.clone() + "lobby-join")
        });
        buttons.create_button(|b| {
            b.label("Leave Lobby!")
                .style(serenity::ButtonStyle::Danger)
                .custom_id(self.id_as_string.clone() + "lobby-leave")
        });

        buttons
    }

    pub fn players_as_add_options(&self) -> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();
        menu.custom_id(self.id_as_string.clone() + "add");

        if self.players.is_empty() {
            menu.disabled(true)
                .placeholder("No characters available!")
                .options(|o| o.create_option(|o| o.label("Empty").value("0")));
        } else if self.active_players.len() == self.content_info().content_size as usize {
            menu.disabled(true)
                .placeholder("Content size limit reached.")
                .options(|o| o.create_option(|o| o.label("empty").value("0")));
        } else {
            menu.options(|o| {
                for (index, player) in self.players.iter().enumerate() {
                    o.add_option(player.option(index));
                }
                o
            })
            .placeholder("Select a player to add.");
        }

        menu
    }

    pub fn active_players_as_remove_options(&self) -> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();

        menu.custom_id(self.id_as_string.clone() + "remove");

        if self.active_players.is_empty() {
            menu.disabled(true)
                .placeholder("No active characters available!")
                .options(|o| o.create_option(|o| o.label("Empty").value("0")));
        } else {
            menu.options(|o| {
                for (index, player) in self.active_players.iter().enumerate() {
                    o.add_option(player.option(index));
                }
                o
            })
            .placeholder("Select a player to remove.");
        }

        menu
    }

    // TODO!: Need to remove all affiliated guildmate characters
    pub async fn add_active_player(&mut self, idx: usize, db: &DatabaseConnection) {
        let player = self.players[idx].clone();

        self.player_list[self.active_players.len()] = format!(
            "\n**{}** ({}) => __**{}** Item Level__ | <@{}>",
            player.name, player.class, player.item_level, player.id
        );

        self.active_players.push(player);

        self.players = get_guildmates_by_min_ilvl_filter_out(
            self.guild_id,
            self.content_info().ilvl_req,
            &self.active_players,
            db,
        )
        .await
        .unwrap_or_default();
    }

    // This function is only called when lobby is posted and when lobby is posted we empty
    // the player list in lobby_context so we dont need to update player list
    pub fn add_active_player_by_model(&mut self, player: entity::characters::Model) {
        self.player_list[self.active_players.len()] = format!(
            "\n**{}** ({}) => __**{}** Item Level__ | <@{}>",
            player.name, player.class, player.item_level, player.id
        );

        self.active_players.push(player);
    }

    pub async fn remove_active_player(&mut self, idx: usize, db: &DatabaseConnection) {
        let _player = self.active_players.remove(idx);
        self.player_list.remove(idx);
        self.player_list.push("\n*This slot is empty*".to_string());

        self.players = get_guildmates_by_min_ilvl_filter_out(
            self.guild_id,
            self.content_info().ilvl_req,
            &self.active_players,
            db,
        )
        .await
        .unwrap_or_default();
    }

    pub fn remove_active_player_without_filter(&mut self, idx: usize) {
        let _player = self.active_players.remove(idx);
        self.player_list.remove(idx);
        self.player_list.push("\n*This slot is empty*".to_string());
    }

    pub fn is_active_player(&self, id: u64) -> bool {
        let id = id.to_string();
        for active_player in &self.active_players {
            if active_player.id == id {
                return true;
            }
        }
        false
    }
}
