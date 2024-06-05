#[macro_use] extern crate rocket;

use rocket::serde::json::Json;
use rocket::State;
use mysql::*;
use dotenv::dotenv;
use std::env;

#[derive(serde::Serialize)]
struct Log {
    timestamp: String,
    hostname: String,
    log_level: String,
    message: String,
}

#[get("/logs?<hostname>&<log_level>&<message>")]
fn search_logs(
    pool: &State<mysql::Pool>,
    hostname: Option<String>,
    log_level: Option<String>,
    message: Option<String>,
) -> Json<Vec<Log>> {
    let mut conn = pool.inner().get_conn().unwrap();

    let query = 
        "SELECT timestamp, hostname, log_level, message
        FROM logs
        WHERE (? IS NULL OR hostname = ?)
        AND (? IS NULL OR log_level = ?)
        AND (? IS NULL OR message LIKE CONCAT('%', ?, '%'))";

    let params = (&hostname, &hostname, &log_level, &log_level, &message, &message);

    let mut stmt = conn.prep(query).unwrap();
    let result = stmt.execute(params).unwrap();

    let logs: Vec<Log> = result.map(|row| {
        let row = row.unwrap();
        Log {
            timestamp: row.get(0).unwrap(),
            hostname: row.get(1).unwrap(),
            log_level: row.get(2).unwrap(),
            message: row.get(3).unwrap(),
        }
    }).collect();

    Json(logs)
}

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = Pool::new(database_url);

    rocket::build()
        .manage(pool)
        .mount("/", routes![search_logs])
        .launch();
}