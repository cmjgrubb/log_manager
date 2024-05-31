#[macro_use] extern crate rocket;

use rocket::serde::json::Json;
use rocket::State;
use mysql::*;
use mysql::prelude::*;
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
fn search_logs(conn: &State<PooledConn>, hostname: Option<String>, log_level: Option<String>, message: Option<String>) -> Json<Vec<Log>> {
    let mut logs = vec![];

    let mut stmt = conn.prepare(
        r"SELECT timestamp, hostname, log_level, message
           FROM logs
           WHERE (:hostname IS NULL OR hostname = :hostname)
             AND (:log_level IS NULL OR log_level = :log_level)
             AND (:message IS NULL OR message LIKE CONCAT('%', :message, '%'))"
    ).unwrap();

    let result = stmt.execute(params! {
        "hostname" => hostname,
        "log_level" => log_level,
        "message" => message.map(|m| format!("%{}%", m)),
    }).unwrap();

    for row in result {
        let row = row.unwrap();
        logs.push(Log {
            timestamp: row.get(0).unwrap(),
            hostname: row.get(1).unwrap(),
            log_level: row.get(2).unwrap(),
            message: row.get(3).unwrap(),
        });
    }

    Json(logs)
}

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = Pool::new(database_url).unwrap();
    let conn = pool.get_conn().unwrap();

    rocket::ignite()
        .manage(conn)
        .mount("/", routes![search_logs])
        .launch();
}