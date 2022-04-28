//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use super::sea_orm_active_enums::Class;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "characters")]
pub struct Model {
    #[sea_orm(column_type = "Text")]
    pub id: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub guild_id: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub name: String,
    pub class: Class,
    pub item_level: i32,
    pub last_updated: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Guildmates,
    LobbyPlayer,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Guildmates => {
                Entity::belongs_to(super::guildmates::Entity)
                .from((Column::Id, Column::GuildId))
                .to((super::guildmates::Column::Id, super::guildmates::Column::ServerId))
                .into()
            },
            Self::LobbyPlayer => {
                Entity::has_many(super::lobby_player::Entity).into()
            }
        }
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
