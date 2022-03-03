use anyhow::anyhow;
use entity::sea_orm::sea_query::{Alias, Expr, Function, SimpleExpr};
use entity::sea_orm::{
    ActiveValue, ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel,
    IntoSimpleExpr, JoinType, QueryFilter, QueryOrder, QuerySelect,
};
use std::collections::BTreeMap;
use std::time::SystemTime;
use tracing::{info, instrument};

use crate::execution::matchmaker::RoundResult;
use entity::sea_orm::prelude::DateTimeUtc;
use entity::{round_result, submission};
use submission::Entity as Submission;

#[derive(Clone, Debug)]
pub struct Database(pub DatabaseConnection);

impl Database {
    #[instrument]
    pub async fn add_submission(&self, submission: submission::Model) -> anyhow::Result<i32> {
        let mut am = submission.into_active_model();

        am.id = ActiveValue::NotSet;

        let id = Submission::insert(am).exec(&self.0).await?.last_insert_id;

        Ok(id)
    }

    #[instrument]
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

    #[instrument]
    pub async fn add_round_result(
        &self,
        round_result: &RoundResult,
        player_strategies: BTreeMap<String, i32>,
    ) -> anyhow::Result<i32> {
        let rr = round_result::Model {
            id: 0,
            result: serde_json::to_string(round_result)?,
            participants: serde_json::to_string(&player_strategies)?,
            datetime: DateTimeUtc::from(SystemTime::now()),
        };

        let mut am = rr.into_active_model();
        am.id = ActiveValue::NotSet;

        let id = round_result::Entity::insert(am)
            .exec(&self.0)
            .await?
            .last_insert_id;

        Ok(id)
    }

    #[instrument]
    pub async fn get_last_rounds_results(&self) -> anyhow::Result<(Vec<RoundResult>, DateTimeUtc)> {
        info!("Getting last rounds results");
        let r = round_result::Entity::find()
            .order_by_desc(round_result::Column::Datetime)
            .limit(5)
            .all(&self.0)
            .await?;

        if r.is_empty() {
            return Err(anyhow!("No round results available"));
        }

        let last_time = r.iter().map(|f| f.datetime).max().unwrap();

        let round_results: serde_json::Result<Vec<RoundResult>> =
            r.iter().map(|f| serde_json::from_str(&f.result)).collect();
        let round_results = round_results?;

        Ok((round_results, last_time))
    }
}
