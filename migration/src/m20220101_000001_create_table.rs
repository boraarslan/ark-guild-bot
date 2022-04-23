use entity::sea_orm::sea_query::extension::postgres::TypeDropStatement;
use entity::sea_orm::Iterable;
use entity::{characters, guildmates, servers};
use entity::{prelude::*, sea_orm_active_enums};
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}
struct IdenRole;
struct IdenClass;

impl Iden for IdenRole {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "role").unwrap();
    }
}

impl Iden for IdenClass {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "class").unwrap();
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Try to delete the types first.
        if let Err(err) = manager
            .drop_type(TypeDropStatement::new().name(IdenClass).to_owned())
            .await
        {
            println!(
                "Couldn't delete type {}: {} \nContinuing...",
                IdenClass.quoted('"'),
                &err
            );
        }
        if let Err(err) = manager
            .drop_type(TypeDropStatement::new().name(IdenRole).to_owned())
            .await
        {
            println!(
                "Couldn't delete type {}: {} \nContinuing...",
                IdenRole.quoted('"'),
                &err
            );
        }

        let db = manager.get_database_backend();
        let schema = sea_orm::Schema::new(db);
        manager
            .create_type(schema.create_enum_from_active_enum::<sea_orm_active_enums::Class>())
            .await?;
        manager
            .create_type(schema.create_enum_from_active_enum::<sea_orm_active_enums::Role>())
            .await?;

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
                    .col(
                        ColumnDef::new(servers::Column::Timezone)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                sea_query::Table::create()
                    .table(Guildmates)
                    .col(ColumnDef::new(guildmates::Column::Id).text().not_null())
                    .col(
                        ColumnDef::new(guildmates::Column::ServerId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(guildmates::Column::Role)
                            .enumeration("role", sea_orm_active_enums::Role::iter())
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(guildmates::Column::Id)
                            .col(guildmates::Column::ServerId),
                    )
                    .foreign_key(
                        sea_query::ForeignKey::create()
                            .name("fk-guildmates-servers")
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
                        ColumnDef::new(characters::Column::GuildId)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(characters::Column::Name).text().not_null())
                    .col(
                        ColumnDef::new(characters::Column::Class)
                            .enumeration("class", sea_orm_active_enums::Class::iter())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(characters::Column::ItemLevel)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(characters::Column::LastUpdated)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(characters::Column::Name)
                            .col(characters::Column::GuildId),
                    )
                    .foreign_key(
                        sea_query::ForeignKey::create()
                            .name("fk-characters-guildmates")
                            .from(
                                Characters,
                                (characters::Column::Id, characters::Column::GuildId),
                            )
                            .to(
                                Guildmates,
                                (guildmates::Column::Id, guildmates::Column::ServerId),
                            )
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
                    .table(Characters)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                sea_query::Table::drop()
                    .table(Guildmates)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                sea_query::Table::drop()
                    .table(Servers)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        // Drop Enums
        manager
            .drop_type(TypeDropStatement::new().name(IdenClass).to_owned())
            .await?;
        manager
            .drop_type(TypeDropStatement::new().name(IdenRole).to_owned())
            .await?;

        Ok(())
    }
}
