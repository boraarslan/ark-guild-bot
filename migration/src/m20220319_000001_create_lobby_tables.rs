use entity::lobby::Entity as Lobby;
use entity::lobby_player::Entity as LobbyPlayer;
use entity::sea_orm::sea_query::extension::postgres::TypeDropStatement;
use entity::sea_orm::Iterable;
use entity::{guildmates, prelude::*};
use entity::{lobby, lobby_player};
use entity::{sea_orm_active_enums, servers};
use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220319_000001_create_lobby_tables"
    }
}

struct IdenContent;

impl Iden for IdenContent {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", "content").unwrap();
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if let Err(err) = manager
            .drop_type(TypeDropStatement::new().name(IdenContent).to_owned())
            .await
        {
            println!(
                "Couldn't delete type {}: {} \nContinuing...",
                IdenContent.quoted('"'),
                &err
            );
        }
        let db = manager.get_database_backend();
        let schema = sea_orm::Schema::new(db);
        manager
            .create_type(schema.create_enum_from_active_enum::<sea_orm_active_enums::Content>())
            .await?;
        // TODO!: Create Types
        manager
            .create_table(
                sea_query::Table::create()
                    .table(Lobby)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(lobby::Column::LobbyId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(lobby::Column::GuildId).text().not_null())
                    .col(ColumnDef::new(lobby::Column::LobbyMaster).text().not_null())
                    .col(
                        ColumnDef::new(lobby::Column::Content)
                            .enumeration("content", sea_orm_active_enums::Content::iter())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(lobby::Column::Created)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(lobby::Column::Scheduled)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(lobby::Column::Active).boolean().not_null())
                    .foreign_key(
                        sea_query::ForeignKey::create()
                            .name("fk-lobby-guild")
                            .from(Lobby, lobby::Column::GuildId)
                            .to(Servers, servers::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        sea_query::ForeignKey::create()
                            .name("fk-lobbymaster-guildmate")
                            .from(Lobby, lobby::Column::LobbyMaster)
                            .to(Guildmates, guildmates::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(LobbyPlayer)
                    .if_not_exists()
                    .col(ColumnDef::new(lobby_player::Column::LobbyId).uuid().not_null())
                    .col(ColumnDef::new(lobby_player::Column::GuildId).text().not_null())
                    .col(ColumnDef::new(lobby_player::Column::PlayerId).text().not_null())
                    .col(ColumnDef::new(lobby_player::Column::CharacterName).text().not_null())
                    .col(ColumnDef::new(lobby_player::Column::Slot).small_integer().not_null())
                    .col(ColumnDef::new(lobby_player::Column::Active).boolean().not_null())
                    .primary_key(
                        Index::create()
                            .col(lobby_player::Column::LobbyId)
                            .col(lobby_player::Column::Slot),
                    )
                    .foreign_key(
                        sea_query::ForeignKey::create()
                            .name("fk-lobbyplayer-lobby")
                            .from(LobbyPlayer, lobby_player::Column::LobbyId)
                            .to(Lobby, lobby_player::Column::LobbyId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                sea_query::Table::drop()
                    .if_exists()
                    .table(LobbyPlayer)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                sea_query::Table::drop()
                    .if_exists()
                    .table(Lobby)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(TypeDropStatement::new().name(IdenContent).to_owned())
            .await?;
        Ok(())
    }
}
