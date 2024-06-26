use async_graphql::{Context, Object};
use chrono::{NaiveDate, NaiveTime};
use sqlx::PgPool;
use  sqlx::types::chrono;
use std::sync::Arc;

use crate::db::{member::Member, attendance::Attendance};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn add_member(
        &self, 
        ctx: &Context<'_>, 
        rollno: String, 
        name: String, 
        hostel: String, 
        email: String, 
        sex: String, 
        year: i32
    ) -> Result<Member, sqlx::Error> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool not found in context");

        let member = sqlx::query_as::<_, Member>(
            "INSERT INTO Member (rollno, name, hostel, email, sex, year) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
        )
        .bind(rollno)
        .bind(name)
        .bind(hostel)
        .bind(email)
        .bind(sex)
        .bind(year)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(member)
    }

    async fn add_attendance(
        &self,
        ctx: &Context<'_>,
        id: i32,
        date: NaiveDate,
        timein: NaiveTime,
        timeout: NaiveTime,
    ) -> Result<Attendance, sqlx::Error> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool not found in context");

        let attendance = sqlx::query_as::<_, Attendance>(
            "INSERT INTO Attendance (id, date, timein, timeout) VALUES ($1, $2, $3, $4) RETURNING *"
        )
        .bind(id)
        .bind(date)
        .bind(timein)
        .bind(timeout)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(attendance)
    }
}
