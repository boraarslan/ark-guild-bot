table! {
    characters (name) {
        id -> Text,
        name -> Text,
        class -> Text,
        item_level -> Integer,
    }
}

table! {
    guildmates (id) {
        id -> Text,
        server_id -> Text,
    }
}

table! {
    servers (id) {
        id -> Text,
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
