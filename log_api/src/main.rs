#[macro_use] extern crate rocket;

use rocket::serde::json::Json;
use rocket::State;
use sqlx::mysql::MySqlPool;
use dotenv::dotenv;
use std::env;

#[derive(serde::Serialize, sqlx::FromRow)]
struct Log {
    timestamp: String,
    hostname: String,
    log_level: String,
    message: String,
}

#[get("/logs?<hostname>&<log_level>&<message>")]
async fn search_logs(
    pool: &State<MySqlPool>,
    hostname: Option<String>,
    log_level: Option<String>,
    message: Option<String>,
) -> Json<Vec<Log>> {
    let query = 
        "SELECT timestamp, hostname, log_level, message
        FROM logs
        WHERE (:hostname IS NULL OR hostname = :hostname)
        AND (:log_level IS NULL OR log_level = :log_level)
        AND (:message IS NULL OR message LIKE CONCAT('%', :message, '%'))";

    let rows = sqlx::query_as::<_, Log>(query)
        .bind(&hostname)
        .bind(&log_level)
        .bind(&message)
        .fetch_all(pool.inner()).await.unwrap();

    Json(rows)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = MySqlPool::connect(&database_url).await?;

    rocket::build()
        .manage(pool)
        .mount("/", routes![search_logs])
        .launch()
        .await?;

    Ok(())
}