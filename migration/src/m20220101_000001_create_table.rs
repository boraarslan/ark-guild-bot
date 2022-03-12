use entity::prelude::*;
use entity::{characters, guildmates, servers};
use sea_schema::migration::prelude::*;
use sea_schema::migration::{
    sea_query::{self, *},
    *,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_query::Table::create()
                    .table(Servers)
                    .col(
                        ColumnDef::new(servers::Column::Id)
                            .text()
                            .primary_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(servers::Column::GuildName).text().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                sea_query::Table::create()
                    .table(Guildmates)
                    .col(
                        ColumnDef::new(guildmates::Column::Id)
                            .text()
                            .primary_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(guildmates::Column::ServerId)
                            .text()
                            .not_null(),
                    )
                    .foreign_key(
                        sea_query::ForeignKey::create()
                            .from(Guildmates, guildmates::Column::ServerId)
                            .to(Servers, servers::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                sea_query::Table::create()
                    .table(Characters)
                    .col(ColumnDef::new(characters::Column::Id).text().not_null())
                    .col(
                        ColumnDef::new(characters::Column::Name)
                            .text()
                            .primary_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(characters::Column::Class).text().not_null())
                    .col(
                        ColumnDef::new(characters::Column::ItemLevel)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(characters::Column::LastUpdated)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        sea_query::ForeignKey::create()
                            .from(Characters, characters::Column::Id)
                            .to(Guildmates, guildmates::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(sea_query::Table::drop().table(Characters).to_owned()).await?;
        manager.drop_table(sea_query::Table::drop().table(Guildmates).to_owned()).await?;
        manager.drop_table(sea_query::Table::drop().table(Servers).to_owned()).await
    }
}
