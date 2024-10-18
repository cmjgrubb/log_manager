#[macro_use]
extern crate rocket;

use dotenv::dotenv;
use rocket::serde::json::Json;
use rocket::State;
use sqlx::mysql::MySqlPool;
use sqlx::FromRow;
use std::env;

#[derive(FromRow, serde::Serialize)]
struct Log {
    id: i32,
    hostname: String,
    timestamp: chrono::NaiveDateTime,
    log_level: String,
    message: String,
}

#[get("/search?<hostname>&<log_level>&<message>")]
async fn search_logs(
    hostname: Option<String>,
    log_level: Option<String>,
    message: Option<String>,
    pool: &State<MySqlPool>,
) -> Result<Json<Vec<Log>>, rocket::http::Status> {
    let query = "
        SELECT id, hostname, timestamp, log_level, message
        FROM logs
        WHERE (:hostname IS NULL OR hostname = :hostname)
        AND (:log_level IS NULL OR log_level = :log_level)
        AND (:message IS NULL OR message LIKE CONCAT('%', :message, '%'))";

    let rows = sqlx::query_as::<_, Log>(query)
        .bind(&hostname)
        .bind(&log_level)
        .bind(&message)
        .fetch_all(pool.inner())
        .await;

    match rows {
        Ok(logs) => Ok(Json(logs)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPool::connect(&database_url).await?;

    rocket::build()
        .manage(pool)
        .mount("/", routes![search_logs])
        .launch()
        .await?;

    Ok(())
}
