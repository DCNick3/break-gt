use entity::{round_result, submission};
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
                    .table(submission::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(submission::Column::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(submission::Column::UserId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(submission::Column::Code).text().not_null())
                    .col(
                        ColumnDef::new(submission::Column::Datetime)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(submission::Column::Valid)
                            .boolean()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .name("IX_Submission_UserId")
                    .table(submission::Entity)
                    .col(submission::Column::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .name("IX_Submission_Datetime")
                    .table(submission::Entity)
                    .col(submission::Column::Datetime)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(round_result::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(round_result::Column::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(round_result::Column::Result)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(round_result::Column::Datetime)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .name("IX_RoundResult_Datetime")
                    .table(round_result::Entity)
                    .col(round_result::Column::Datetime)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
