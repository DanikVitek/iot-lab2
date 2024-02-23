use actix_web::{web, App, HttpServer};
use color_eyre::eyre::Result;
use sqlx::PgPool;
use std::thread;
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use lab2::{
    config::Configuration,
    control::{self, ws::Subscribers},
    data, FileStdoutWriter, KtConvenience,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let (non_blocking, _guard) = tracing_appender::non_blocking(FileStdoutWriter::new(
        tracing_appender::rolling::never("./logs", "lab2.log"),
    ));
    SubscriberInitExt::try_init(
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_writer(non_blocking)
            .with_thread_ids(true),
    )?;

    let config = Configuration::try_read()?;
    tracing::debug!("Configuration: {:#?}", config);

    let pool: PgPool = PgPool::connect_with(config.database().connect_options()).await?;
    tracing::info!("Connected to database");

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations successfully applied");

    let openapi = ApiDocs::openapi();

    HttpServer::new(move || {
        App::new()
            .service(control::ws::ws_endpoint)
            .service(control::http::create_processed_agent_data)
            .service(control::http::create_processed_agent_data_list)
            .service(control::http::read_processed_agent_data)
            .service(control::http::read_processed_agent_data_list)
            .service(control::http::update_processed_agent_data)
            .service(control::http::delete_processed_agent_data)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(Subscribers::new()))
            .also(|_| {
                tracing::info!("App built for worker {:?}", thread::current().id());
            })
    })
    .bind(config.server())?
    .run()
    .await?;

    Ok(())
}

#[derive(OpenApi)]
#[openapi(
    paths(
        control::http::create_processed_agent_data,
        control::http::create_processed_agent_data_list,
        control::http::read_processed_agent_data,
        control::http::read_processed_agent_data_list,
        control::http::update_processed_agent_data,
        control::http::delete_processed_agent_data,
    ),
    components(
        schemas(data::Accelerometer, data::Gps, data::Agent, data::ProcessedAgent),
        responses(data::Accelerometer, data::Gps, data::Agent, data::ProcessedAgent),
    ),
)]
struct ApiDocs;
