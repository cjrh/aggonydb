use aggonydb;
use anyhow::Result;
use log::info;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting up");
    aggonydb::server().await.map_err(anyhow::Error::from)
}
