pub mod db;
pub mod routes;
pub mod schema;

use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub async fn server() -> Result<()> {
    let mypool = db::MyPool::new().await?;
    schema::migrate().await?;

    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&mypool.pool)
        .await?;
    assert_eq!(row.0, 150);

    let insert_sql = r#"
    INSERT INTO helloworld(id, set) VALUES (1, hll_empty())
    "#;

    let add_sql = r#"
    UPDATE helloworld SET set = hll_add(set, hll_hash_integer(12345)) WHERE id = 1;
    "#;

    mypool.q(insert_sql).await?;
    mypool.q(add_sql).await?;

    let row: (f64,) = sqlx::query_as("SELECT hll_cardinality(set) FROM helloworld WHERE id = 1")
        .fetch_one(&mypool.pool)
        .await?;
    println!("{}", row.0);

    let shared_pool = Arc::new(Mutex::new(mypool));

    // Start the web server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(shared_pool.clone()))
            .service(routes::hello)
            .service(routes::echo)
            .service(routes::add)
            .service(routes::get_value_count)
            .route("/hey", web::get().to(routes::manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
