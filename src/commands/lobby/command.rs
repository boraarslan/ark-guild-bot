use crate::{check::is_guild_init, commands::lobby::context::LobbyContext, info::*};
use chrono::Utc;
use helper::*;
use parking_lot::RwLock;
use poise::{
    serenity_prelude::{CollectComponentInteraction, CreateComponents, Message},
    Context,
};
use sea_orm::DbErr;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;

use super::*;
use crate::*;

#[derive(Debug, Clone, Copy)]
pub enum State {
    ContentSelection,
    LobbyFirstPrompt,
    CollectPlayers,
    PrivateLobby,
    PublicLobby,
    Generated,
}

#[poise::command(slash_command, category = "Lobby", guild_only, check = "is_guild_init")]
pub async fn create_lobby(
    ctx: Context<'_, Data, Error>,
    #[description = "(Optional) Time of the lobby. You can choose another time later"]
    lobby_time: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;
    let db = ctx.data().db;

    let lobby_master = match get_guildmate(ctx.author().id.0, guild_id, db).await {
        Ok(guildmate) => {
            if guildmate.role == Role::Guildmate {
                ctx.say("Only guild administration can create lobbies.")
                    .await?;
                return Ok(());
            }
            ctx.author()
        }
        Err(DbErr::RecordNotFound(_)) => {
            ctx.say("You are not registered in guild.").await?;
            return Ok(());
        }
        Err(_) => {
            ctx.say("Error getting guildmate record from database.")
                .await?;
            return Ok(());
        }
    };

    let lobby_time = if let Some(lobby_time) = lobby_time {
        let offset = chrono::offset::FixedOffset::east(
            get_server(guild_id, db).await?.timezone as i32 * 3600,
        );
        dateparser::parse_with_timezone(&lobby_time, &offset).ok()
    } else {
        None
    };

    let lobby_id = uuid::Uuid::new_v4();
    let lobby_id_string = lobby_id.to_hyphenated().to_string();

    // Check if lobby time is valid
    let lobby_time = match lobby_time {
        None => {
            ctx.send(|m| {
                m.embed(|e| {
                    e.title("No Time Specified")
                    .description("Couldn't set lobby time. Either you did not specify a lobby time or the time format is false")
                    .field("Example usage", "`/create_lobby <lobby time>`\n`/create_lobby 6:00pm`\n`/create_lobby May 02, 2021 15:51 UTC+2`", false)
                    .field("\0", "If no time zone is specified the guild time zone is used.", false)
                })
            }).await?;
            None
        }
        Some(lobby_time) => {
            if lobby_time <= (chrono::Utc::now() + chrono::Duration::minutes(15)) {
                ctx.send(|m| {
                    m.embed(|e| {
                        e.title("Lobby time cannot be within 15 minutes")
                        .description(format!("Couldn't set lobby time. Got time: {}",lobby_time))
                        .field("Example usage", "`/create_lobby <lobby time>`\n`/create_lobby 6:00pm`\n`/create_lobby May 02, 2021 15:51 UTC+2`", false)
                        .field("\0", "If no time zone is specified the guild time zone is used.", false)
                    })
                }).await?;
                None
            } else if lobby_time >= (chrono::Utc::now() + chrono::Duration::weeks(2)) {
                ctx.send(|m| {
                    m.embed(|e| {
                        e.title("Lobby time must be within 2 weeks.")
                        .description(format!("Couldn't set lobby time. Got time: {}",lobby_time))
                        .field("Example usage", "`/create_lobby <lobby time>`\n`/create_lobby 6:00pm`\n`/create_lobby May 02, 2021 15:51 UTC+2`", false)
                        .field("\0", "If no time zone is specified the guild time zone is used.", false)
                    })
                }).await?;
                None
            } else {
                Some(lobby_time)
            }
        }
    };

    let reply_handle = ctx
        .send(|m| {
            m.embed(|e| {
                e.title("Please select the content.")
                //TODO!: Add fields for each content
            })
            .components(|c| {
                c.create_action_row(|r| {
                    r.create_button(|b| {
                        b.style(serenity::ButtonStyle::Danger)
                            .custom_id(lobby_id_string.clone() + "guardian-raid")
                            .label("Guardian Raid")
                    });
                    r.create_button(|b| {
                        b.style(serenity::ButtonStyle::Primary)
                            .custom_id(lobby_id_string.clone() + "abyss-dungeon")
                            .label("Abyss Dungeon")
                    });
                    r.create_button(|b| {
                        b.style(serenity::ButtonStyle::Success)
                            .custom_id(lobby_id_string.clone() + "abyss-raid")
                            .label("Abyss Raid")
                    })
                })
            })
        })
        .await?;

    let message_id = reply_handle.message().await?.id.0;

    let lobby_context_locked = Arc::new(RwLock::new(LobbyContext {
        id: lobby_id,
        id_as_string: lobby_id_string.clone(),
        guild_id,
        channel_id: ctx.channel_id().0,
        message_id,
        lobby_master: lobby_master.id.0,
        state: State::ContentSelection,
        content: None,
        content_info: None,
        lobby_time: (lobby_time, None),
        players: vec![],
        active_players: vec![],
        player_list: vec![],
        http_client: ctx.discord().http.clone(),
    }));

    while let Some(mci) = CollectComponentInteraction::new(ctx.discord())
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(600))
        .filter({
            let slice = lobby_id_string.clone();
            move |mci| mci.data.custom_id.starts_with(slice.as_str())
        })
        .await
    {
        let mut lobby_context = lobby_context_locked.write();
        match lobby_context.state {
            State::ContentSelection => {
                lobby_context.set_content(Some(LobbyContent::from(
                    mci.data
                        .custom_id
                        .get(lobby_id_string.len()..mci.data.custom_id.len())
                        .unwrap(),
                )));

                mci.create_interaction_response(ctx.discord(), |ir| {
                    ir.kind(serenity::model::interactions::InteractionResponseType::UpdateMessage)
                })
                .await?;

                create_select_content_message(
                    mci.message.clone(),
                    ctx,
                    lobby_context.lobby_content(),
                    &lobby_id_string,
                )
                .await?;
                lobby_context.state = State::LobbyFirstPrompt;
            }

            State::LobbyFirstPrompt => {
                mci.create_interaction_response(ctx.discord(), |ir| {
                    ir.kind(serenity::model::interactions::InteractionResponseType::UpdateMessage)
                })
                .await?;

                lobby_context
                    .set_content_info(Some(CONTENT_DATA.get(&mci.data.values[0]).unwrap()));

                let characters =
                    get_guildmates_by_min_ilvl(guild_id, lobby_context.content_info().ilvl_req, db)
                        .await;

                lobby_context.player_list = vec![
                    "\n*This slot is empty*".to_string();
                    lobby_context.content_info().content_size as usize
                ];

                // I hate this fucking design. Just spent the last hour trying to move the embed and component creation
                // logic to lobby_context struct just to see E0521 which makes PERFECT sense because the `EditMessage`
                // is OUTSIDE of the closure not the INSIDE and don't get me started with the nested closures like I am some
                // kind of fucking JS developer holy shit this is an ugly and unreadable mess. Like seriously just let me
                // pass a struct with a Default implementation and save me from this atrocity so i can move my logic PLUS
                // it wouldn't look as bad as this. Maybe I don't know or couldn't find it but all of Embed, Menu, Message
                // types are either Vecs or HashMaps because (i think) they are converted to JSON but I can't explain
                // myself enough how mad I am because I had to write this paragraph HOLYSHITAHSUGDFUAYFSDYATSD
                //
                // Edit from the future: Turns out I can kind of modularize the embed and action rows but not the message
                // because it assigns stored data to builder type at background and its kind of pointless to do the same
                // myself but I am ok with what I have now. Not deleting the original rant so I can remember the tough times
                // our humankind went through. My ancestors would be proud.
                mci.message
                    .clone()
                    .edit(ctx.discord(), |m| {
                        m.embed(|e| {
                            *e = lobby_context.create_embed();
                            e
                        })
                        .components(|c| {
                            c.create_action_row(|r| {
                                r.create_select_menu(|m| {
                                    match characters {
                                        Err(DbErr::RecordNotFound(_)) => {
                                            m.disabled(true)
                                            .placeholder("There are no characters who can participate to this lobby :(")
                                            .options(|o| o.create_option(|option|
                                                option.value("0").label("None")))
                                            .custom_id(&lobby_id_string)

                                        }
                                        Err(_) => {
                                            m.disabled(true)
                                            .placeholder("Error getting characters from database!")
                                            .options(|o| o.create_option(|option|
                                                option.value("0").label("None")))
                                            .custom_id(&lobby_id_string)

                                        }
                                        _ => {
                                            lobby_context.players = characters.unwrap();
                                            *m = lobby_context.players_as_add_options();
                                            m
                                        }
                                    }
                                })
                            }).create_action_row(|r| {
                                r.create_select_menu(|m| {
                                    *m = lobby_context.active_players_as_remove_options();
                                    m
                                })
                            }).create_action_row(|r| {
                                *r = lobby_context.create_lobby_buttons();
                                r
                            })
                        })
                    })
                    .await?;
                lobby_context.state = State::CollectPlayers;
            }

            State::CollectPlayers => {
                mci.create_interaction_response(ctx.discord(), |ir| {
                    ir.kind(serenity::model::interactions::InteractionResponseType::UpdateMessage)
                })
                .await?;

                // Extract the str payload
                match mci
                    .data
                    .custom_id
                    .get(lobby_id_string.len()..mci.data.custom_id.len())
                    .unwrap()
                {
                    "add" => {
                        lobby_context
                            .add_active_player(mci.data.values[0].parse::<usize>().unwrap(), db)
                            .await;
                    }
                    "remove" => {
                        lobby_context
                            .remove_active_player(mci.data.values[0].parse::<usize>().unwrap(), db)
                            .await;
                    }
                    "post-lobby" => {
                        lobby_context.state = State::PrivateLobby;

                        mci.message
                            .clone()
                            .edit(ctx.discord(), |m| {
                                m.embed(|e| {
                                    *e = lobby_context.create_embed();
                                    e
                                })
                                .set_components(CreateComponents::default())
                            })
                            .await?;

                        break;
                    }
                    "open-lobby" => {
                        lobby_context.state = State::PublicLobby;

                        mci.message
                            .clone()
                            .edit(ctx.discord(), |m| {
                                m.embed(|e| {
                                    *e = lobby_context.create_embed();
                                    e
                                })
                                .components(|c| {
                                    c.set_action_row(lobby_context.create_user_buttons())
                                })
                            })
                            .await?;

                        break;
                    }
                    "close-lobby" => {
                        mci.message.clone().delete(ctx.discord()).await?;
                        ctx.say("Lobby is closed").await?;

                        return Ok(());
                    }
                    _ => unreachable!(),
                }

                mci.message
                    .clone()
                    .edit(ctx.discord(), |m| {
                        m.embed(|e| {
                            *e = lobby_context.create_embed();
                            e
                        })
                        .components(|c| {
                            c.create_action_row(|r| {
                                r.create_select_menu(|m| {
                                    *m = lobby_context.players_as_add_options();
                                    m
                                })
                            })
                            .create_action_row(|r| {
                                r.create_select_menu(|m| {
                                    *m = lobby_context.active_players_as_remove_options();
                                    m
                                })
                            })
                            .create_action_row(|r| {
                                *r = lobby_context.create_lobby_buttons();
                                r
                            })
                        })
                    })
                    .await?;
            }
            _ => unreachable!(),
        }
    }

    {
        let mut lobby_context = lobby_context_locked.write();
        // We do not need players after lobby is posted.
        lobby_context.players = vec![];
    }

    // From this point on lobby is created and inserted to database
    // Further component interactions will be collected from event listener
    // and be sent to seperate task spawned below from channels.
    // That way the active lobbies can be reinitialized after a shutdown.
    //
    // I spent too much time thinking about this and i am not proud of it.

    insert_lobby(&lobby_context_locked.read(), db).await?;
    let (sender, mut reciever) = unbounded_channel::<LobbyEvent>();
    ctx.data().active_lobbies.write().insert(
        lobby_context_locked.read().id_as_string.clone(),
        sender.clone(),
    );

    println!(
        "Inserted lobby id: {}",
        lobby_context_locked.read().id_as_string
    );

    // It is the first time creating a timer task so second field is always None
    if let Some(time) = lobby_context_locked.read().lobby_time.0 {
        lobby_context_locked.write().lobby_time.1 = Some(tokio::spawn({
            let channel = sender;
            let lobby_context_locked = lobby_context_locked.clone();
            let db = db;
            let active_lobbies = ctx.data().active_lobbies.clone();
            async move{
                let time_left = time - Utc::now();
                // We check the time range so unwrapping is okay
                let time_left = std::time::Duration::from_millis(
                    time_left.num_milliseconds().try_into().unwrap(),
                );

                // Message the users when 10 mins left
                tokio::time::sleep(time_left - std::time::Duration::from_secs(600)).await;

                let _ = channel.send(LobbyEvent::LobbyIsDue);

                tokio::time::sleep(std::time::Duration::from_secs(600)).await;

                // Make lobby inactive
                let lobby = get_lobby(lobby_context_locked.read().id, db).await;
                if let Ok(ref lobby) = lobby {
                    let _ = disable_lobby(lobby, db).await;
                }

                active_lobbies.write().remove(&lobby_context_locked.read().id_as_string);
            }
        }))
    }

    // End the command context here and spawn a background task
    tokio::spawn({
        let db = ctx.data().db.clone();
        async move {
            while let Some(event) = reciever.recv().await {
                match process_lobby_event(event, lobby_context_locked.clone(), &db).await {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error processing event: {err}")
                    }
                }
            }
        }
    });

    Ok(())
}

async fn create_select_content_message(
    mut message: Message,
    ctx: Context<'_, Data, Error>,
    content: LobbyContent,
    custom_id: &str,
) -> Result<(), Error> {
    message
        .edit(ctx.discord(), |m| {
            m.embed(|e| e.title(format!("Select the {}", content)))
                .components(|c| {
                    c.create_action_row(|r| {
                        r.create_select_menu(|m| {
                            m.placeholder(format!("Please select the {}", content))
                                .options(|o| {
                                    for instance in content.get_content_list() {
                                        o.create_option(|option| {
                                            option.label(instance).value(instance)
                                        });
                                    }
                                    o
                                })
                                .custom_id(custom_id)
                        })
                    })
                })
        })
        .await?;
    Ok(())
}
