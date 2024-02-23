use actix_web::{web, App, HttpServer};
use color_eyre::eyre::Result;
use sqlx::PgPool;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use lab2::{
    config::Configuration,
    control::{self, ws::Subscribers},
    FileStdoutWriter,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let (non_blocking, _guard) = tracing_appender::non_blocking(FileStdoutWriter::new(
        tracing_appender::rolling::never("./log", "lab1.log"),
    ));
    SubscriberInitExt::try_init(
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_writer(non_blocking)
            .with_thread_ids(true),
    )?;

    let config = Configuration::try_read()?;

    let pool: PgPool = PgPool::connect_lazy_with(config.database().connect_options());

    sqlx::migrate!("./migrations").run(&pool).await?;

    HttpServer::new(move || {
        App::new()
            .service(control::ws::ws_endpoint)
            .service(control::http::create_processed_agent_data)
            .service(control::http::create_processed_agent_data_list)
            .service(control::http::read_processed_agent_data)
            .service(control::http::read_processed_agent_data_list)
            .service(control::http::update_processed_agent_data)
            .service(control::http::delete_processed_agent_data)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(Subscribers::new()))
    })
    .bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8000))?
    .run()
    .await?;

    Ok(())
}
