use chrono::{NaiveDate, NaiveTime};
use sqlx::FromRow;
use async_graphql::SimpleObject;

//Struct for the Attendance table
#[derive(FromRow, SimpleObject)]
pub struct Attendance {
    pub id: i32,
    pub date: NaiveDate,
    pub timein: NaiveTime,
    pub timeout: NaiveTime,
    pub present: bool,
}
