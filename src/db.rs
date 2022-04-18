use sqlx::postgres::{PgPoolOptions, PgQueryResult};
use anyhow::Result;
use sqlx::{Database, Pool, Postgres};

pub struct MyPool {
    pub pool: Pool<Postgres>,
}

impl MyPool {
    pub async fn new() -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://admin:password@localhost:55432/test").await?;

        let p = MyPool { pool };
        p.setup().await?;
        Ok(p)
    }

    pub async fn q(&self, sql: &str) -> Result<PgQueryResult> {
        Ok(sqlx::query(sql).execute(&self.pool).await?)
    }

    /// Enable all required extensions, and other setup steps for the
    /// connection pool.
    pub async fn setup(&self) -> Result<()> {
        self.q("CREATE EXTENSION IF NOT EXISTS hll").await?;
        Ok(())
    }
}


pub async fn q(pool: &Pool<Postgres>, sql: &str) -> Result<PgQueryResult> {
    Ok(sqlx::query(sql).execute(pool).await?)
}

