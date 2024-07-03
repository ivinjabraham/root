
use std::{env, sync::Arc};
use tokio::task;
use tokio::time::{sleep, sleep_until, Instant};
use std::time::Duration;
use async_graphql_axum::GraphQL;
use axum::{routing::get, Router};
use chrono::{ Local, NaiveTime};
use db::attendance::Attendance;
use db::member::Member;
use sqlx::PgPool;
use async_graphql::{ Schema, EmptySubscription};

use crate::graphql::mutations::MutationRoot;
use crate::graphql::query::QueryRoot;
use crate::routes::graphiql;

mod db;
mod graphql;
mod routes;

#[derive(Clone)]
struct MyState {
    pool: Arc<PgPool>,
}

//Main method
#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    env::set_var("PGOPTIONS", "-c ignore_version=true");
    
    sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");

    let pool = Arc::new(pool);
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool.clone())
        .finish();

    let state = MyState { pool: pool.clone() };
    
    let router = Router::new()
        .route("/", get(graphiql).post_service(GraphQL::new(schema.clone())))
        .with_state(state);
    task::spawn(async move {
         
        schedule_task_at_midnight(pool.clone()).await; // Call the function after 10 seconds
    });


    Ok(router.into())
}



//Scheduled task for moving all members to Attendance table at midnight.
async fn scheduled_task(pool: Arc<PgPool>) {
    let members: Result<Vec<Member>, sqlx::Error> = sqlx::query_as::<_, Member>("SELECT * FROM Member")
        .fetch_all(pool.as_ref())
        .await;

    match members {
        Ok(members) => {
            let today = Local::now().naive_local();

            for member in members {
                let timein = NaiveTime::from_hms_opt(0, 0, 0);
                let timeout = NaiveTime::from_hms_opt(0, 0, 0); // Default time, can be modified as needed
                
                let attendance = sqlx::query(
                    "INSERT INTO Attendance (id, date, timein, timeout, present) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id, date) DO NOTHING RETURNING *"
                )
                .bind(member.id)
                .bind(today)
                .bind(timein)
                .bind(timeout)
                .bind(false)
                .execute(pool.as_ref())
                .await;
            
                match attendance {
                    Ok(_) => println!("Attendance record added for member ID: {}", member.id),
                    Err(e) => eprintln!("Failed to insert attendance for member ID: {}: {:?}", member.id, e),
                }
            }
        },
        Err(e) => eprintln!("Failed to fetch members: {:?}", e),
    }
}

//Ticker for calling the scheduled task
async fn schedule_task_at_midnight(pool: Arc<PgPool>) {
    loop {
    let now = Local::now();

    let tomorrow = now.date_naive().succ_opt().unwrap();
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let next_midnight = tomorrow.and_time(midnight);

    let now_naive = now.naive_local();
    let duration_until_midnight = next_midnight.signed_duration_since(now_naive);
    let sleep_duration = Duration::from_secs(duration_until_midnight.num_seconds()  as u64 + 60);

    sleep_until(Instant::now() + sleep_duration).await;
    scheduled_task(pool.clone()).await;
    print!("done");
    }
}