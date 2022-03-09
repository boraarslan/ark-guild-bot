use ark_guild_bot::models::Character;
use ark_guild_bot::*;
use ark_guild_bot::{models::Server, schema::servers};
use diesel::prelude::*;
use dotenv::dotenv;
use poise::serenity_prelude as serenity;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

fn construct_character_list(characters: &Vec<Character>) -> String {
    let mut character_list = String::new();
    character_list.push_str("```");
    character_list.push_str(&format!(
        "{:<15} {:<15}    {}\n",
        "Name", "Class", "Item Level"
    ));
    character_list.push_str(&format!("{:-<15} {:-<15}    {:-<10}\n", "", "", ""));
    for character in characters {
        character_list.push_str(&format!(
            "{:<15} {:<15} -> {:<5} ilvl \n",
            character.name, character.class, character.item_level
        ));
    }
    character_list.push_str("```");
    character_list
}

#[poise::command(prefix_command, hide_in_help, required_permissions = "ADMINISTRATOR")]
async fn register_guild(ctx: Context<'_>, name: String, #[flag] global: bool) -> Result<(), Error> {
    if let Err(err) = poise::builtins::register_application_commands(ctx, global).await {
        ctx.say(format!("Failed to register application commands, {err}"))
            .await?;
    }

    if let Some(_) = get_server(ctx.guild_id().expect("No guild id").0) {
        remove_server(ctx.guild_id().expect("No guild id").0).expect("Failed to remove server");
    }
    let connection = establish_connection();

    let new_server = Server {
        id: ctx.guild_id().expect("No guild id").0.to_string(),
        guild_name: name,
    };

    diesel::insert_into(servers::table)
        .values(&new_server)
        .execute(&connection)
        .expect("Error saving new server");
    Ok(())
}

#[poise::command(slash_command, category = "Character")]
async fn character(
    ctx: Context<'_>,
    #[description = "Name of the character"] character_name: String,
    #[description = "Class of the character"] class: ark_guild_bot::Class,
    #[description = "Item level of the character"]
    #[min = 0]
    #[max = 1490]
    item_level: i32,
) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("You must be in a guild to use this command")
            .await?;
        return Ok(());
    };

    if let None = get_guildmate(ctx.author().id.0) {
        ctx.say("No guildmate found, registering with current name")
            .await?;
        insert_guildmate(guild_id, ctx.author().id.0).expect("Failed to insert guildmate");
    }

    if let None = get_single_character(&character_name) {
        insert_character(ctx.author().id.0, &character_name, class, item_level)
            .expect("Failed to insert character");
        ctx.say(format!(
            "Added **{character_name}** as *{class}* to your characters (__{item_level}__ Item Level)"
        ))
        .await?;
    } else {
        update_character(&character_name, class, item_level).expect("Error updating character");
        ctx.say(format!(
            "Updated character named **{character_name}** as *{class}* with __{item_level}__ Item Level"
        ))
        .await?;
    }

    Ok(())
}

#[poise::command(slash_command, category = "Character")]
async fn list_characters(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(characters) = get_all_characters(ctx.author().id.0) {
        if characters.len() == 0 {
            ctx.say("You have no characters.").await?;
            return Ok(());
        }
        let character_list = construct_character_list(&characters);
        ctx.send(|m| {
            m.embed(|e| {
                e.title(format!("Characters of {}", ctx.author().name))
                .field("Characters:", character_list, false)
                .thumbnail(ctx.author().avatar_url().unwrap_or_default())
            })
        })
        .await?;
    } else {
        ctx.say("Error getting characters from database.").await?;
    }

    Ok(())
}

#[poise::command(slash_command, track_edits, category = "Character")]
async fn delete_character(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(characters) = get_all_characters(ctx.author().id.0) {
        let custom_uuid = ctx.id();

        let character_list = construct_character_list(&characters);
        ctx.send(|m| {
            m.embed(|e| {
                e.title(format!("Characters of {}", ctx.author().name))
                    .field("Characters:", &character_list, false)
                    .thumbnail(ctx.author().avatar_url().unwrap_or_default())
            })
            .components(|c| {
                c.create_action_row(|r| {
                    r.create_select_menu(|m| {
                        m.placeholder(format!("Select a character to delete"))
                            .options(|o| {
                                for character in characters {
                                    o.create_option(|option| {
                                        option
                                            .label(&character.name)
                                            .description(format!(
                                                "{:<15} -> {:<5} ilvl",
                                                character.class, character.item_level
                                            ))
                                            .value(&character.name)
                                    });
                                }
                                o
                            })
                            .custom_id(custom_uuid)
                    })
                })
            })
        })
        .await?;

        if let Some(mci) = serenity::CollectComponentInteraction::new(ctx.discord())
            .author_id(ctx.author().id)
            .channel_id(ctx.channel_id())
            .timeout(std::time::Duration::from_secs(60))
            .filter(move |mci| mci.data.custom_id == custom_uuid.to_string())
            .await
        {
            remove_character(&mci.data.values[0]).expect("Failed to remove character");
            mci.create_interaction_response(ctx.discord(), |ir| {
                ir.kind(serenity::model::interactions::InteractionResponseType::UpdateMessage)
            })
            .await?;

            let mut msg = mci.message.clone();
            msg.edit(ctx.discord(), |m| {
                m.embed(|e| {
                    e.title("Character deleted")
                        .description(format!(
                            "```Deleted {} from your character list.```",
                            mci.data.values[0]
                        ))
                        .thumbnail(ctx.author().avatar_url().unwrap_or_default())
                })
                .components(|c| c)
            })
            .await?;
        }
    } else {
        ctx.say("No characters found").await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    poise::Framework::build()
        .token(std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"))
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }))
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            commands: vec![
                register_guild(),
                character(),
                list_characters(),
                delete_character(),
            ],
            ..Default::default()
        })
        .run()
        .await
        .unwrap();
}
