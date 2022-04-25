use crate::db::MyPool;
use anyhow::Result;
use log::*;
use serde::Deserialize;

#[derive(Debug, sqlx::FromRow)]
pub struct Dataset {
    pub id: i32,
    pub name: String,
}

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

    // This is the list of migrations
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
            set     theta_sketch
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
            ds theta_sketch
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

pub async fn add_dataset_if_missing(
    pool: &MyPool,
    dataset: &str,
) -> Result<i32> {
    let sql = "INSERT INTO dataset (name) VALUES ($1) ON CONFLICT DO NOTHING";
    let _x = sqlx::query(sql).bind(dataset).execute(&pool.pool).await?;
    let fetch = "SELECT id FROM dataset where name = $1";
    let row: (i32,) = sqlx::query_as(fetch)
        .bind(dataset)
        .fetch_one(&pool.pool)
        .await?;
    Ok(row.0)
}

pub async fn remove_dataset(pool: &MyPool, dataset: i32) -> Result<()> {
    let sql = "DELETE FROM dataset WHERE id = $1";
    sqlx::query(sql).bind(dataset).execute(&pool.pool).await?;
    Ok(())
}

pub async fn add_event(
    pool: &MyPool,
    dataset: i32,
    field_name: &str,
    value: &str,
    id: u32,
) -> Result<()> {
    let sql = r#"
        UPDATE counter SET
            -- ds = theta_sketch_add_item(ds, $4)
            ds = theta_sketch_union(ds, (select theta_sketch_build($4)))
        WHERE
            dataset_id = $1
            AND field_name = $2
            AND value = $3;
    "#;
    let result = sqlx::query(sql)
        .bind(dataset)
        .bind(field_name)
        .bind(value)
        .bind(id)
        .execute(&pool.pool)
        .await?;
    info!("result: {result:?}");
    if result.rows_affected() == 0 {
        let sql = r#"
        INSERT INTO counter
            (dataset_id, field_name, value, ds)
        VALUES
            ($1, $2, $3, (select theta_sketch_build($4)));
    "#;
        sqlx::query(sql)
            .bind(dataset)
            .bind(field_name)
            .bind(value)
            .bind(id)
            .execute(&pool.pool)
            .await?;
    }

    Ok(())
}

pub async fn count(
    pool: &MyPool,
    dataset: &str,
    field_name: &str,
    value: &str,
) -> Result<f64> {
    let sql = r#"
        SELECT
            theta_sketch_get_estimate(ds)
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

#[derive(Deserialize, Debug)]
pub struct Filter {
    pub field_name: String,
    pub value: String,
}

impl Filter {
    pub fn new(field_name: &str, value: &str) -> Self {
        Filter {
            field_name: field_name.to_string(),
            value: value.to_string(),
        }
    }
}

pub async fn count_filter(
    pool: &MyPool,
    dataset: &str,
    filters: &[Filter],
) -> Result<Vec<f64>> {
    let mut fragments = vec![];
    let mut params = vec![];
    let mut i = 2;
    for f in filters.iter() {
        let ip1 = i + 1;
        let fragment = format!("OR (field_name = ${i} AND value = ${ip1})");
        fragments.push(fragment);
        params.push(f.field_name.clone());
        params.push(f.value.clone());
        i += 2;
    }

    let all_fragments = fragments.join("\n            ");

    let sql = format!(
        r#"
        SELECT
            -- theta_sketch_get_estimate(
            theta_sketch_get_estimate_and_bounds(
                theta_sketch_intersection(ds)
            )
        FROM counter
            INNER JOIN dataset ON dataset.id = counter.dataset_id
        WHERE
            dataset.name = $1
            AND (
                false
                {all_fragments}
            );
    "#
    );
    let mut qb = sqlx::query_as(sql.as_ref()).bind(dataset);

    // Bind all the other parameters in the order given
    for p in params {
        qb = qb.bind(p);
    }

    let result = match qb.fetch_one(&pool.pool).await {
        Ok((c,)) => c,
        Err(e) => {
            println!("{e}");
            vec![0.0, 0.0, 0.0]
        }
    };
    info!("{:?}", result);

    Ok(result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
