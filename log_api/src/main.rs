use dotenv::dotenv;
use rocket::get;
use rocket::routes;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use sqlx::FromRow;
use std::env;

#[derive(FromRow, Serialize, Deserialize)]
struct Log {
    id: i32,
    hostname: String,
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
        WHERE (? IS NULL OR hostname = ?)
        AND (? IS NULL OR log_level = ?)
        AND (? IS NULL OR message LIKE CONCAT('%', ?, '%'))";

    let rows = sqlx::query_as::<_, Log>(query)
        .bind(&hostname)
        .bind(&hostname)
        .bind(&log_level)
        .bind(&log_level)
        .bind(&message)
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
