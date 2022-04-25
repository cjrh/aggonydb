use actix_rt;
use aggonydb::{db::MyPool, schema};
use anyhow::Result;
use log::*;
use rstest::*;
use sqlx;
use sqlx::postgres::PgRow;
use sqlx::Row;
use std::future::Future;

#[fixture]
#[once]
fn setup() {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();
}

#[fixture]
#[once]
fn rt(setup: ()) -> actix_rt::Runtime {
    actix_rt::Runtime::new().unwrap()
}

#[fixture]
async fn pool(setup: ()) -> MyPool {
    let pool = aggonydb::db::MyPool::new().await.unwrap();
    schema::migrate().await.unwrap();
    pool
}

#[rstest]
#[actix_rt::test]
async fn add_dataset_if_missing(#[future] pool: MyPool) -> Result<()> {
    let p = pool.await;

    let dataset_id = schema::add_dataset_if_missing(&p, "test dataset").await?;
    let rows = sqlx::query_as::<_, schema::Dataset>(
        "select * from dataset where name = 'test dataset'",
    )
    .fetch_all(&p.pool)
    .await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "test dataset");

    schema::remove_dataset(&p, dataset_id).await?;
    let rows = sqlx::query_as::<_, schema::Dataset>(
        "select * from dataset where name = 'test dataset'",
    )
    .fetch_all(&p.pool)
    .await?;
    assert_eq!(rows.len(), 0);

    Ok(())
}

#[rstest]
#[actix_rt::test]
async fn add_events(#[future] pool: MyPool) -> Result<()> {
    let p = pool.await;

    // Add a dataset. It will return the dataset id, which will be necessary
    // to provide when adding data
    let did = schema::add_dataset_if_missing(&p, "d2").await?;
    println!("dataset id: {:?}", did);

    // Add data!
    for i in 0..100 {
        schema::add_event(&p, did, "City", "Brisbane", i).await?;
    }

    // Check the count for the value "Brisbane" in field "City"
    let count = schema::count(&p, "d2", "City", "Brisbane").await?;
    assert_eq!(count, 100.0);

    schema::remove_dataset(&p, did).await?;
    Ok(())
}

#[rstest]
#[actix_rt::test]
async fn intersection(#[future] pool: MyPool) -> Result<()> {
    let p = pool.await;

    // Add a dataset. It will return the dataset id, which will be necessary
    // to provide when adding data
    let did = schema::add_dataset_if_missing(&p, "intersection test").await?;
    println!("dataset id: {:?}", did);

    // Add data!
    for i in 0..5 {
        schema::add_event(&p, did, "City", "Brisbane", i).await?;
        schema::add_event(&p, did, "Name", "Caleb", i).await?;
    }
    // Add data!
    for i in 5..10 {
        schema::add_event(&p, did, "City", "Cape Town", i).await?;
        schema::add_event(&p, did, "Name", "Caleb", i).await?;
    }
    // Add data!
    for i in 10..15 {
        schema::add_event(&p, did, "City", "Cape Town", i).await?;
        schema::add_event(&p, did, "Name", "Gina", i).await?;
    }

    // Check the counts for some combinations of filters
    let count = schema::count_filter(
        &p,
        "intersection test",
        &vec![
            schema::Filter::new("City", "Brisbane"),
            schema::Filter::new("Name", "Caleb"),
        ],
    )
    .await?;
    println!("{count:?}");
    assert_eq!(count, vec![5.0, 5.0, 5.0]);

    // Check the counts for some combinations of filters
    let count = schema::count_filter(
        &p,
        "intersection test",
        &vec![schema::Filter::new("City", "Cape Town")],
    )
    .await?;
    println!("{count:?}");
    assert_eq!(count, vec![10.0, 10.0, 10.0]);

    // Check the counts for some combinations of filters
    let count = schema::count_filter(
        &p,
        "intersection test",
        &vec![schema::Filter::new("Name", "Gina")],
    )
    .await?;
    println!("{count:?}");
    assert_eq!(count, vec![5.0, 5.0, 5.0]);

    schema::remove_dataset(&p, did).await?;
    Ok(())
}
