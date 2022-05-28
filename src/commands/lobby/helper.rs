use anyhow::anyhow;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use entity::sea_orm_active_enums::Content;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use parse_display::Display;
use poise::serenity_prelude::CreateSelectMenuOption;
use poise::serenity_prelude::GuildId;
use poise::serenity_prelude::User;
use poise::serenity_prelude::UserId;
use poise::serenity_prelude::{self as serenity, MessageComponentInteraction};
use sea_orm::{DatabaseConnection, DbErr};
use std::sync::Arc;

use crate::LobbyMap;
use crate::database::disable_lobby;
use crate::database::get_lobby;
use crate::{
    database::{
        get_all_character_by_ilvl, get_single_character, insert_lobby_player, remove_lobby_player,
    },
    info::*,
    Error, EventComponent,
};

use super::context::LobbyContext;

pub trait AddOption {
    fn option<V: ToString>(&self, val: V) -> CreateSelectMenuOption;
}

impl AddOption for entity::characters::Model {
    // TODO! Add user name to description
    fn option<V: ToString>(&self, val: V) -> CreateSelectMenuOption {
        let mut option = CreateSelectMenuOption::default();
        option
            .label(&self.name)
            .description(format!("{:<15} -> {:<5} ilvl", self.class, self.item_level))
            .value(val);
        option
    }
}

#[derive(Display, Clone, Copy)]
pub enum LobbyContent {
    #[display("Guardian Raid")]
    GuardianRaid,
    #[display("Abyss Dungeon")]
    AbyssDungeon,
    #[display("Abyss Raid")]
    AbyssRaid,
}

impl From<&str> for LobbyContent {
    fn from(val: &str) -> Self {
        match val {
            "guardian-raid" => Self::GuardianRaid,
            "abyss-dungeon" => Self::AbyssDungeon,
            "abyss-raid" => Self::AbyssRaid,
            _ => unreachable!(),
        }
    }
}

impl LobbyContent {
    pub fn get_content_list(&self) -> &Vec<Content> {
        match self {
            LobbyContent::GuardianRaid => &*GUARDIAN_RAIDS,
            LobbyContent::AbyssDungeon => &*ABYSS_DUNGEONS,
            LobbyContent::AbyssRaid => &*ABYSS_RAIDS,
        }
    }
}
/// List of lobby events tracked by event listener
// This list acts like a filter. Any event that is not in this
// list gets filtered.
pub static LOBBY_EVENTS: Lazy<Vec<&str>> =
    Lazy::new(|| vec!["lobby-join", "player-join", "lobby-leave"]);


// This event is constructed and sent to the lobby manager task.
pub enum LobbyEvent {
    LobbyJoin(EventComponent),
    PlayerJoin(EventComponent),
    LobbyLeave(EventComponent),
    ChangeTime(
        DateTime<Utc>,
        LobbyMap,
    ),
    LobbyIsDue,
    CloseLobby(LobbyMap),
}

impl LobbyEvent {
    pub fn new() -> EventBuilder {
        EventBuilder::default()
    }
}

/// A builder type for LobbyEvent
// I know this is unnecessary but I felt like this is cool.
#[derive(Default)]
pub struct EventBuilder {
    interaction: Option<MessageComponentInteraction>,
    http_client: Option<Arc<serenity::http::client::Http>>,
}

impl EventBuilder {
    pub fn component_interaction(mut self, interaction: MessageComponentInteraction) -> Self {
        self.interaction = Some(interaction);
        self
    }

    pub fn http_client(mut self, client: Arc<serenity::http::client::Http>) -> Self {
        self.http_client = Some(client);
        self
    }

    fn check_for_event_component(&self, event: &str) -> Result<()> {
        if let None = self.interaction {
            return Err(anyhow!("Interaction must be set for event {event}"));
        }
        if let None = self.http_client {
            return Err(anyhow!("Http client must be set for event {event}"));
        }
        Ok(())
    }

    pub fn build(self, event: &str) -> Result<LobbyEvent> {
        match event {
            "lobby-join" => {
                self.check_for_event_component(event)?;
                Ok(LobbyEvent::LobbyJoin(EventComponent {
                    message_component_interaction: self.interaction.unwrap(),
                    http_client: self.http_client.unwrap(),
                }))
            }
            "player-join" => {
                self.check_for_event_component(event)?;
                Ok(LobbyEvent::PlayerJoin(EventComponent {
                    message_component_interaction: self.interaction.unwrap(),
                    http_client: self.http_client.unwrap(),
                }))
            }
            "lobby-leave" => {
                self.check_for_event_component(event)?;
                Ok(LobbyEvent::LobbyLeave(EventComponent {
                    message_component_interaction: self.interaction.unwrap(),
                    http_client: self.http_client.unwrap(),
                }))
            }
            _ => Err(anyhow!("Got event: {event}. Which is not tracked")),
        }
    }
}

pub async fn process_lobby_event(
    event: LobbyEvent,
    lobby_context_locked: Arc<RwLock<LobbyContext>>,
    db: &DatabaseConnection,
) -> Result<(), Error> {
    match event {
        LobbyEvent::LobbyJoin(component) => {
            let lobby_context = lobby_context_locked.read();
            let mci = component.message_component_interaction;
            let http_client = component.http_client;

            // Check if lobby is full
            if lobby_context.content_info().content_size == lobby_context.active_players.len() {
                mci.create_interaction_response(&http_client, |m| {
                    m.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.embed(|e| {
                                e.description(
                                    "Lobby is already full. Wait for someone else to leave.",
                                )
                            })
                            .flags(
                                serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL,
                            )
                        })
                })
                .await
                .expect("Couldn't send message");
                return Ok(());
            }

            // If user is already is an active player
            if lobby_context.is_active_player(mci.user.id.0) {
                mci.create_interaction_response(&http_client, |m| {
                    m.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.embed(|e| e.description("You are already in the lobby dumbass."))
                                .flags(serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                        })
                })
                .await
                .expect("Couldn't send message");
                return Ok(());
            }

            // Get current user chars
            let user_chars = get_all_character_by_ilvl(
                mci.user.id.0,
                lobby_context.guild_id,
                lobby_context.content_info().ilvl_req,
                db,
            )
            .await;

            match user_chars {
                Err(DbErr::RecordNotFound(_)) => {
                    mci.create_interaction_response(&http_client, |m| {
                        m.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.embed(|e| {
                                e.description("You currently don't have any characters that can join this lobby.")
                            })
                            .flags(serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                        })
                    }).await.expect("Couldn't create a response.")

                }
                Ok(user_chars) => {
                    mci.create_interaction_response(&http_client, |m| {
                    m.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.flags(serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                            .embed(|e| {
                                e.description(
                                    "Please select a character to join the lobby.",
                                )
                                .title(&lobby_context.content_info().name)
                            })
                            .components(|c| {
                                c.create_action_row(|r| {
                                    r.create_select_menu(|m| {
                                        m.custom_id(
                                            lobby_context.id_as_string.clone() + "player-join",
                                        ).options(|o| {
                                            for char in user_chars {
                                                o.add_option(char.option(&char.name));
                                            }
                                            o
                                        })
                                    })
                                })
                            })
                        })
                    })
                    .await
                    .expect("Couldn't create a response.");
                }

                Err(_) => {
                    mci.create_interaction_response(&http_client, |r| {
                        r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.embed(|e| {
                                e.description("Database Error! :(")
                            })
                        })
                    }).await.expect("Couldn't create a response.")
                }
            };
            Ok(())
        }
        LobbyEvent::PlayerJoin(component) => {
            let mci = component.message_component_interaction;
            let http_client = component.http_client;
            //
            let (channel, message_id) = {
                let lobby_context = lobby_context_locked.read();
                (
                    serenity::ChannelId(lobby_context.channel_id),
                    lobby_context.message_id,
                )
            };

            let player =
                get_single_character(mci.data.values[0].as_str(), mci.guild_id.unwrap().0, db)
                    .await
                    .expect("Database Error");

            // Add character to lobby
            {
                insert_lobby_player(&lobby_context_locked.read(), &player, db).await?;
                let mut lobby_context = lobby_context_locked.write();

                if lobby_context.content_info().content_size == lobby_context.active_players.len() {
                    return Ok(());
                }

                lobby_context.add_active_player_by_model(player);

                let lobby_embed = lobby_context.create_embed();
                let lobby_buttons = lobby_context.create_user_buttons();

                channel
                    .edit_message(&http_client, message_id, |m| {
                        m.embed(|e| {
                            *e = lobby_embed;
                            e
                        })
                        .components(|c| c.set_action_row(lobby_buttons))
                    })
                    .await
                    .expect("Couldn't edit the message");
            }

            mci.create_interaction_response(&http_client, |r| {
                r.kind(serenity::InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.flags(serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                            .embed(|e| e.description("Added your character!"))
                            .components(|c| c)
                    })
            })
            .await
            .expect("Couldn't generate response");

            Ok(())
        }
        LobbyEvent::LobbyLeave(component) => {
            let mci = component.message_component_interaction;
            let http_client = component.http_client;

            let (channel, message_id) = {
                let lobby_context = lobby_context_locked.read();
                (
                    serenity::ChannelId(lobby_context.channel_id),
                    lobby_context.message_id,
                )
            };

            let mut deleted = false;

            let (embed, buttons) = {
                let mut lobby_context = lobby_context_locked.write();

                let user_id = mci.user.id.0.to_string();

                for (index, char) in lobby_context.active_players.iter().enumerate() {
                    if char.id == user_id {
                        remove_lobby_player(lobby_context.id, &char.name, db).await?;
                        lobby_context.remove_active_player_without_filter(index);
                        deleted = true;
                        break;
                    }
                }
                (
                    lobby_context.create_embed(),
                    lobby_context.create_user_buttons(),
                )
            };

            if deleted {
                channel
                    .edit_message(&http_client, message_id, |m| {
                        m.embed(|e| {
                            *e = embed;
                            e
                        })
                        .components(|c| c.set_action_row(buttons))
                    })
                    .await
                    .expect("Couldn't edit the message.");

                mci.create_interaction_response(&http_client, |r| {
                    r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.flags(
                                serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL,
                            )
                            .embed(|e| e.description("Removed you from the lobby."))
                        })
                })
                .await
                .expect("Couldn't create response");
            } else {
                mci.create_interaction_response(&http_client, |r| {
                    r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.flags(
                                serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL,
                            )
                            .embed(|e| e.description("You are not in the lobby."))
                        })
                })
                .await
                .expect("Couldn't create response");
            }

            Ok(())
        }
        LobbyEvent::ChangeTime(time, active_lobbies) => {
            {
                let mut lobby_context = lobby_context_locked.write();
                lobby_context.lobby_time.0 = Some(time);
                if let Some(handle) = lobby_context.lobby_time.1.as_ref() {
                    handle.abort();
                }

                lobby_context.lobby_time.1 = Some(tokio::spawn({
                    let lobby_context_locked = lobby_context_locked.clone();
                    let db = db.clone();
                    let active_lobbies = active_lobbies.clone();
                    async move {
                        let time_left = time - Utc::now();
                        // We check the time range so unwrapping is okay
                        let time_left = std::time::Duration::from_millis(
                            time_left.num_milliseconds().try_into().unwrap(),
                        );

                        // Message the users when 10 mins left
                        tokio::time::sleep(time_left - std::time::Duration::from_secs(600)).await;

                        let _ = active_lobbies
                            .read()
                            .get(&lobby_context_locked.read().id_as_string)
                            .unwrap()
                            .send(LobbyEvent::LobbyIsDue);

                        tokio::time::sleep(std::time::Duration::from_secs(600)).await;

                        // Make lobby inactive
                        let lobby = get_lobby(lobby_context_locked.read().id, &db).await;
                        if let Ok(ref lobby) = lobby {
                            let _ = disable_lobby(lobby, &db).await;
                        }

                        active_lobbies
                            .write()
                            .remove(&lobby_context_locked.read().id_as_string);
                    }
                }))
            }

            let http_client = lobby_context_locked.read().http_client.clone();
            let (channel, message_id, lobby_embed, lobby_buttons) = {
                let lobby_context = lobby_context_locked.read();
                (
                    serenity::ChannelId(lobby_context.channel_id),
                    lobby_context.message_id,
                    lobby_context.create_embed(),
                    lobby_context.create_user_buttons(),
                )
            };

            // Edit the original message
            channel
                .edit_message(&http_client, message_id, |m| {
                    m.embed(|e| {
                        *e = lobby_embed;
                        e
                    })
                    .components(|c| c.set_action_row(lobby_buttons))
                })
                .await
                .expect("Couldn't edit the message");

            let users = get_users_from_ids(&lobby_context_locked.read()).await;

            let lobby_context = lobby_context_locked.read();
            let guild = GuildId(lobby_context.guild_id)
                .to_partial_guild(lobby_context.http_client.clone())
                .await?;

            for user in users {
                user.dm(lobby_context.http_client.clone(), |message| {
                    message.embed(|e| {
                        e.title("Your lobby is rescheduled.")
                            .description(format!(
                                "Your {} lobby in server {} has been rescheduled to (<t:{}:F>). Don't forget about it!",
                                lobby_context.content_info().name,
                                guild.name,
                                time.timestamp()
                            ))
                    })
                })
                .await?;
            }
            Ok(())
        }
        LobbyEvent::LobbyIsDue => {
            let lobby_context = lobby_context_locked.read();
            let guild = GuildId(lobby_context.guild_id)
                .to_partial_guild(lobby_context.http_client.clone())
                .await?;

            // Build users from active player ids
            let users: Vec<User> = get_users_from_ids(&lobby_context).await;

            for user in users {
                user.dm(lobby_context.http_client.clone(), |message| {
                    message.embed(|e| {
                        e.title("Your lobby starts in 10 minutes.")
                            .description(format!(
                                "Your {} lobby in server {} starts soon. Have fun!",
                                lobby_context.content_info().name,
                                guild.name
                            ))
                    })
                })
                .await?;
            }
            Ok(())
        }
        LobbyEvent::CloseLobby(active_lobbies) => {
            let mut lobby_context = lobby_context_locked.write();
            lobby_context.drop_timebomb();
            active_lobbies.write().remove(&lobby_context.id_as_string);
            Ok(())
        },
    }
}

async fn get_users_from_ids(lobby_context: &LobbyContext) -> Vec<User> {
    let mut users = vec![];
    for model in &lobby_context.active_players {
        let user = UserId(model.id.parse::<u64>().unwrap())
            .to_user(lobby_context.http_client.clone())
            .await;
        users.push(user);
    }
    users.into_iter().flatten().collect()
}
