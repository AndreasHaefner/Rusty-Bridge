use sqlx::postgres::PgPool;
use redis::{AsyncCommands, Client as RedisClient};
use std::env;
use std::time::Duration;
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};


pub struct DbConfig {
    pub db: PgPool,
    pub redis: RedisClient,
}

impl DbConfig {
    fn get_retry_strategy() -> impl Iterator<Item = Duration> + Clone {
        ExponentialBackoff::from_millis(500)
            .map(jitter)
            .take(10)
        }

    pub async fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let db_url = env::var("DATABASE_URL")?;
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

       
        let postgr_db = Retry::spawn(Self::get_retry_strategy(), || async {
            PgPool::connect(&db_url).await
        }).await.expect("Postgres Verbindung fehlgeschlagen");

        sqlx::migrate!("./migrations")
            .run(&postgr_db)
            .await
            .expect("Migrationen konnten nicht ausgeführt werden");
        println!("✅ Datenbank-Schema ist aktuell!");
        
        // Redis Setup
        let redis_client = RedisClient::open(redis_url)
            .expect("Ungültige Redis-URL");

          
        
        Ok(DbConfig { db: postgr_db, redis: redis_client })
    }
    pub async fn get_redis_conn(&self) -> Result<redis::aio::MultiplexedConnection, redis::RedisError> {
        self.redis.get_multiplexed_async_connection().await
    }
   pub async fn redis_test_conn(&self) -> Result<(), redis::RedisError> {
    Retry::spawn(Self::get_retry_strategy(), || async {

        let mut con = self.redis.get_multiplexed_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut con).await?;
        Ok::<(), redis::RedisError>(())
    }).await
}

    pub fn get_db(&self) -> &PgPool {
        &self.db
    }
}