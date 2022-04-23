use std::sync::Arc;

use entity::sea_orm_active_enums::Content;
use parking_lot::RwLock;
use parse_display::Display;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CreateSelectMenuOption;
use sea_orm::{DatabaseConnection, DbErr};

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
pub enum LobbyEvent {
    LobbyJoin,
    PlayerJoin,
    LobbyLeave,
}

#[derive(Debug, Display)]
#[display("...")]
pub struct EventParseError {}

impl std::error::Error for EventParseError {}

impl TryFrom<&str> for LobbyEvent {
    type Error = EventParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "lobby-join" => Ok(Self::LobbyJoin),
            "player-join" => Ok(Self::PlayerJoin),
            "lobby-leave" => Ok(Self::LobbyLeave),
            _ => Err(EventParseError {}),
        }
    }
}

pub async fn process_lobby_event(
    event_c: EventComponent,
    lobby_context_locked: Arc<RwLock<LobbyContext>>,
    db: &DatabaseConnection,
) -> Result<(), Error> {
    match event_c.event {
        LobbyEvent::LobbyJoin => {
            let lobby_context = lobby_context_locked.read();
            let mci = event_c.message_component_interaction;
            let http_client = event_c.http_client;

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
        LobbyEvent::PlayerJoin => {
            let mci = event_c.message_component_interaction;
            let http_client = event_c.http_client;
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
        LobbyEvent::LobbyLeave => {
            let mci = event_c.message_component_interaction;
            let http_client = event_c.http_client;

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
    }
}
