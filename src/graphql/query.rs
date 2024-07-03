use async_graphql::{Context, Object};
use sqlx::PgPool;
use std::sync::Arc;
use chrono::NaiveDate;


use crate::db::{member::Member, attendance::Attendance};

pub struct QueryRoot;

#[Object]
impl QueryRoot {

    //Query for retrieving the Members
    async fn get_member(&self, ctx: &Context<'_>) -> Result<Vec<Member>, sqlx::Error> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool not found in context");
        let users = sqlx::query_as::<_, Member>("SELECT * FROM Member")
            .fetch_all(pool.as_ref())
            .await?;
        Ok(users)
    }

    //Query for retrieving the Attendance based on date
    async fn get_attendance(
        &self,
        ctx: &Context<'_>,
        date: NaiveDate,
    ) -> Result<Vec<Attendance>, sqlx::Error> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool not found in context");

        let attendance_list = sqlx::query_as::<_, Attendance>(
            "SELECT id, date, timein, timeout FROM Attendance WHERE date = $1"
        )
        .bind(date)
        .fetch_all(pool.as_ref())
        .await?;
        Ok(attendance_list)
    }
}
