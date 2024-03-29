//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use super::sea_orm_active_enums::Content;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "lobby")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub lobby_id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub guild_id: String,
    #[sea_orm(column_type = "Text")]
    pub channel_id: String,
    #[sea_orm(column_type = "Text")]
    pub message_id: String,
    #[sea_orm(column_type = "Text")]
    pub lobby_master: String,
    pub content: Content,
    pub created: DateTimeUtc,
    pub scheduled: Option<DateTimeUtc>,
    pub active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::servers::Entity",
        from = "Column::GuildId",
        to = "super::servers::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Servers,
    #[sea_orm(
        belongs_to = "super::guildmates::Entity",
        from = "Column::LobbyMaster",
        to = "super::guildmates::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Guildmates,
    #[sea_orm(has_many = "super::lobby_player::Entity")]
    LobbyPlayer,
}

impl Related<super::servers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Servers.def()
    }
}

impl Related<super::guildmates::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Guildmates.def()
    }
}

impl Related<super::lobby_player::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LobbyPlayer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
