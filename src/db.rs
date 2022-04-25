use anyhow::Result;
use log::*;
use sqlx::postgres::{PgPoolOptions, PgQueryResult};
use sqlx::{Connection, Executor, PgConnection, Pool, Postgres};

#[derive(Debug)]
pub struct MyPool {
    pub pool: Pool<Postgres>,
}

impl MyPool {
    pub async fn new() -> Result<Self> {
        let url = "postgres://admin:password@localhost:55432/";

        // First ensure that the /test database exists
        ensure_test_database(url).await?;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(format!("{url}/test").as_ref())
            .await?;

        let p = MyPool { pool };
        p.setup().await?;
        Ok(p)
    }

    pub async fn q(&self, sql: &str) -> Result<PgQueryResult> {
        Ok(sqlx::query(sql).execute(&self.pool).await?)
    }

    // Enable all required extensions, and other setup steps for the
    // connection pool.
    pub async fn setup(&self) -> Result<()> {
        self.q("CREATE EXTENSION IF NOT EXISTS datasketches;")
            .await?;
        Ok(())
    }
}

async fn ensure_test_database(url: &str) -> Result<()> {
    // First ensure that the /test database exists
    let mut conn = PgConnection::connect(format!("{url}/postgres").as_ref()).await?;
    if let Err(e) = conn.execute("CREATE DATABASE test;").await {
        // Can't do IF NOT EXISTS for create database, so just log out
        // the error.
        warn!("{e}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works_db() {
        assert_eq!(2 + 2, 4);
    }
}
