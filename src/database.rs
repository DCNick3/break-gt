use entity::sea_orm::sea_query::{Alias, Expr, Function, SelectStatement, SimpleExpr};
use entity::sea_orm::{
    ActiveValue, ColumnTrait, Condition, DatabaseConnection, DbBackend, EntityTrait,
    IntoActiveModel, IntoSimpleExpr, JoinType, QueryFilter, QuerySelect, QueryTrait,
};
use std::error;

use entity::submission;
use submission::Entity as Submission;

#[derive(Clone)]
pub struct Database(pub DatabaseConnection);

impl Database {
    pub async fn add_submission(
        &self,
        submission: submission::Model,
    ) -> Result<i32, anyhow::Error> {
        log::info!(
            "Add submission from {}, valid = {}",
            submission.id,
            submission.valid
        );

        let mut am = submission.into_active_model();

        am.id = ActiveValue::NotSet;

        let id = Submission::insert(am).exec(&self.0).await?.last_insert_id;

        Ok(id)
    }

    pub async fn get_active_submissions(&self) -> Result<Vec<submission::Model>, anyhow::Error> {
        let mut datetime_q = Submission::find()
            .select_only()
            .column(submission::Column::UserId)
            .filter(submission::Column::Valid.eq(true))
            .group_by(submission::Column::UserId);

        let dt = Alias::new("dt");
        let datetime_q = QuerySelect::query(&mut datetime_q).expr_as(
            SimpleExpr::FunctionCall(
                Function::Max,
                vec![submission::Column::Datetime.into_simple_expr()],
            ),
            dt.clone(),
        );

        let alias = Alias::new("s1");

        let mut q = Submission::find();
        QuerySelect::query(&mut q).join_subquery(
            JoinType::InnerJoin,
            datetime_q.clone(),
            alias.clone(),
            Condition::all()
                .add(
                    Expr::tbl(alias.clone(), submission::Column::UserId)
                        .equals(Submission {}, submission::Column::UserId),
                )
                .add(
                    Expr::tbl(alias.clone(), dt)
                        .equals(Submission {}, submission::Column::Datetime),
                ),
        );

        Ok(q.all(&self.0).await?)
    }
}
