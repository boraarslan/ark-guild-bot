Ark Guild Helper
====================

Ark Guild Helper is a guild Discord bot for managing *Lost Ark* lobbies. It allows guilds and players to plan the content they want to do with an easy user interface using slash commands and Discord components.

Features
====================
Aside from being fully async (thanks to Poise, Serenity and SeaORM) Ark Guild Helper can reinitialize active lobbies when restarted so you can use the original bot messages to interact with the lobbies.

## Screenshots

You can register your characters:

<img src=./images/list_char.png width="300">

Example lobby:

<img src=./images/lobby_player_select.png width="300">

After opening the lobby, players can join or leave the lobby:

<img src=./images/open_lobby.png width="300">

<img src=./images/player_join.png width="300">

## Usage

To self-host the bot you need to set up a PostgreSQL server and a discord bot application.
After setting those up, create a `.env` file in project folder with two parameters:

```
DISCORD_TOKEN=
DATABASE_URL=
```

Then you can run the bot using 

```
cargo run --release
```

Note: When running the bot first time, you need to register the slash commands using `!register_commands` command. For more information visit [poise's documentation](https://docs.rs/poise/latest/poise/#introduction-to-slash-commands).