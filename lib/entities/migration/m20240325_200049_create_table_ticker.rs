use sea_orm_migration::{async_trait::async_trait, prelude::*};

#[derive(DeriveMigrationName)]
/// 🐎 » create table `ticker`
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Ticker::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Ticker::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Ticker::Symbol).string().not_null())
                    .col(ColumnDef::new(Ticker::QuoteSymbol).string().null())
                    .col(ColumnDef::new(Ticker::MarketId).string().not_null())
                    .col(ColumnDef::new(Ticker::Active).boolean().not_null().default(true))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ticker_market_id")
                            .from(Ticker::Table, Ticker::MarketId)
                            .to(Market::Table, Market::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Ticker::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Market {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Ticker {
    Table,
    Id,
    /// Ticker symbol (e.g. "GOOGL")
    Symbol,
    /// Quote symbol (e.g. "USD"): represents the currency in which the ticker is quoted.
    /// Could be null, and it will imply USD
    QuoteSymbol,
    /// Market ID
    MarketId,
    /// Active status of the ticker. This defines if quotes are generated for this ticker or not.
    Active,
}
