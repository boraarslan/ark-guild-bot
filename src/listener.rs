use crate::{
    commands::{Data, Error, lobby::helper::{LOBBY_EVENTS, LobbyEvent}},
};
use poise::serenity_prelude as serenity;

pub async fn listener(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: &poise::Framework<Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot: _ } => println!("Ready to do stuff."),
        poise::Event::InteractionCreate { interaction } => {
            match interaction {
                serenity::Interaction::MessageComponent(mci) => {
                    // UUIDv4 length is 36 characters
                    let (lobby_id_str, event_str) = mci.data.custom_id.split_at(36);

                    if !LOBBY_EVENTS.contains(&event_str) {
                        println!("Event is not tracked");
                        return Ok(());
                    }

                    let event = LobbyEvent::new().component_interaction(mci.clone()).http_client(ctx.http.clone()).build(event_str)?;
                    
                    println!("Lobby id: ({lobby_id_str})");

                    let res = user_data
                        .active_lobbies
                        .read()
                        .get(lobby_id_str)
                        .expect("No active lobby found with given id")
                        .send(event);

                    if let Err(err) = res {
                        println!("Error sending event component to task {err}");
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}
