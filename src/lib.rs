pub mod db;
pub mod routes;
pub mod schema;

use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub async fn server() -> Result<()> {
    let mypool = db::MyPool::new().await?;
    schema::migrate().await?;

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
            .service(routes::filter_count)
            .route("/hey", web::get().to(routes::manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
