use dotenv::dotenv;
use mysql::*;
use mysql::prelude::*;
use std::env;
use regex::Regex;

pub struct Processor {
    conn: PooledConn,
}

impl Processor {
    pub fn new() -> Result<Processor, mysql::Error> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let pool = Pool::new(database_url)?;
        let conn = pool.get_conn()?;
        Ok(Processor { conn })
    }

    pub fn process_log(&mut self, log: &str) -> Result<(), mysql::Error> {
        let re = Regex::new(r"<\d+>\d+ (\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z) (\S+) (\S+) - .* - (.*)").unwrap();
        let caps = re.captures(log).expect("Failed to parse log message");
    
        let timestamp = caps.get(1).map_or("", |m| m.as_str());
        let hostname = caps.get(2).map_or("", |m| m.as_str());
        let log_level = caps.get(3).map_or("", |m| m.as_str());
        let message = caps.get(4).map_or("", |m| m.as_str());
    
        self.conn.exec_drop(
            r"INSERT INTO logs (timestamp, hostname, log_level, message) VALUES (:timestamp, :hostname, :log_level, :message)",
            params! {
                "timestamp" => timestamp,
                "hostname" => hostname,
                "log_level" => log_level,
                "message" => message
            }
        )
    }
}