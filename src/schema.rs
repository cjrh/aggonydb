use crate::db::MyPool;
use anyhow::Result;

struct Migration<'a, F>
where
    F: std::future::Future,
{
    id: u32,
    /// Forward migration
    fwd: fn(&'a MyPool) -> F,
    /// Reverse migration
    rev: Option<fn(&'a MyPool) -> F>,
}

pub async fn migrate() -> Result<()> {
    let mypool = MyPool::new().await?;

    /// This is the list of migrations
    let migrations = vec![Migration {
        id: 1,
        fwd: initial_schema,
        rev: None,
    }];

    for m in migrations.iter() {
        (m.fwd)(&mypool).await;
        if let Some(f) = m.rev {
            f(&mypool).await;
        }
    }

    Ok(())
}

async fn initial_schema(pool: &MyPool) {
    let create_table_sql = r#"
        --- Make a dummy table
        CREATE TABLE IF NOT EXISTS helloworld (
            id      integer,
            set     hll
        );
    "#;

    pool.q(create_table_sql).await.unwrap();

    pool.q(r#"
        CREATE TABLE IF NOT EXISTS dataset (
            id serial primary key,
            name text unique not null
        );
    "#)
        .await
        .unwrap();

    // We're going to coerce the incoming value to string anyway,
    // but we need to record its type?
    pool.q(r#"
        CREATE TABLE IF NOT EXISTS counter (
            dataset_id integer REFERENCES dataset(id) ON DELETE CASCADE,
            field_name text not null,
            value text not null, 
            ds hll
        );
    "#)
        .await
        .unwrap();

    pool.q(r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_counter ON counter(
            dataset_id, field_name, value
        );
    "#)
        .await
        .unwrap();
}

pub async fn add_dataset_if_missing(pool: &MyPool, dataset: &str) -> Result<i32> {
    let sql = "INSERT INTO dataset (name) VALUES ($1) ON CONFLICT DO NOTHING";
    let x = sqlx::query(sql).bind(dataset).execute(&pool.pool).await?;
    let fetch = "SELECT id FROM dataset where name = $1";
    let row: (i32,) = sqlx::query_as(fetch)
        .bind(dataset)
        .fetch_one(&pool.pool)
        .await?;
    Ok(row.0)
}

pub async fn add_event(pool: &MyPool, dataset: i32, field_name: &str, value: &str) -> Result<()> {
    // Ensure the field/value combo exists.
    let sql = r#"
        INSERT INTO counter
            (dataset_id, field_name, value, ds)
        VALUES
            ($1, $2, $3, hll_empty()) ON CONFLICT DO NOTHING;
    "#;
    sqlx::query(sql)
        .bind(dataset)
        .bind(field_name)
        .bind(value)
        .execute(&pool.pool)
        .await?;

    // Now update the count
    let sql = r#"
        UPDATE counter SET
            ds = hll_add(ds, hll_hash_text($4))
        WHERE
            dataset_id = $1
            AND field_name = $2
            AND value = $3;
    "#;
    let u = uuid::Uuid::new_v4().to_simple().to_string();
    sqlx::query(sql)
        .bind(dataset)
        .bind(field_name)
        .bind(value)
        .bind(u)
        .execute(&pool.pool)
        .await?;

    Ok(())
}

pub async fn count(pool: &MyPool, dataset: &str, field_name: &str, value: &str) -> Result<f64> {
    let sql = r#"
        SELECT
            hll_cardinality(ds)
        FROM counter
            INNER JOIN dataset ON dataset.id = counter.dataset_id
        WHERE
            dataset.name = $1
            AND field_name = $2
            AND value = $3;
    "#;
    let result = match sqlx::query_as(sql)
        .bind(dataset)
        .bind(field_name)
        .bind(value)
        .fetch_one(&pool.pool)
        .await
    {
        Ok((c,)) => c,
        Err(_) => 0.0,
    };
    println!("{}", result);

    Ok(result)
}
