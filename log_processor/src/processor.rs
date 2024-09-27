use sqlx::mysql::MySqlPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct Processor {
    db_pool: Arc<MySqlPool>,
}

impl Processor {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db_pool = MySqlPool::connect(database_url).await?;

        Ok(Processor {
            db_pool: Arc::new(db_pool),
        })
    }

    pub async fn process_log(&self, log: &str) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query("INSERT INTO syslogs (message) VALUES (?)")
            .bind(log)
            .execute(&*self.db_pool)
            .await?;
        Ok(())
    }
}
