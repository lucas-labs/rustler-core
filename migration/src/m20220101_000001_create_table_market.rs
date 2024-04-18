use sea_orm_migration::{async_trait::async_trait, prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Market::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Market::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Market::ShortName).string().not_null())
                    .col(ColumnDef::new(Market::FullName).string().not_null())
                    .col(ColumnDef::new(Market::OpensFrom).unsigned().null())
                    .col(ColumnDef::new(Market::OpensTill).unsigned().null())
                    .col(ColumnDef::new(Market::OpenTime).string().null())
                    .col(ColumnDef::new(Market::CloseTime).string().null())
                    .col(ColumnDef::new(Market::PreMarketOffset).unsigned().null())
                    .col(ColumnDef::new(Market::PostMarketOffset).unsigned().null())
                    .col(ColumnDef::new(Market::TimeZoneOffset).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Market::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Market {
    Table,
    Id,
    /// Short name of the market (e.g. "NYSE")
    ShortName,
    /// Full name of the market (e.g. "New York Stock Exchange")
    FullName,
    /// Day of the week the market opens (using UTC 0-6, where 0 is Sunday)
    OpensFrom,
    /// Day of the week the market closes (using UTC 0-6, where 0 is Sunday)
    OpensTill,
    /// Time the market opens (string in HH:MM format)
    OpenTime,
    /// Time the market closes (string in HH:MM format)
    CloseTime,
    /// Pre-market offset (in hours)
    PreMarketOffset,
    /// Post-market offset (in hours)
    PostMarketOffset,
    /// Timezone offset (UTC offset for the opening and closing times in hours)
    TimeZoneOffset,
}
