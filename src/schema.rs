table! {
    characters (name) {
        id -> Integer,
        name -> Text,
        class -> Text,
        item_level -> Integer,
    }
}

table! {
    guildmates (id) {
        id -> Integer,
        server_id -> Integer,
    }
}

table! {
    servers (id) {
        id -> Integer,
        guild_name -> Text,
    }
}

joinable!(characters -> guildmates (id));
joinable!(guildmates -> servers (id));

allow_tables_to_appear_in_same_query!(
    characters,
    guildmates,
    servers,
);
