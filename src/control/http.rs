use actix_web::http::StatusCode;
use actix_web::web::Redirect;
use actix_web::{get, http::header, post, put, web, HttpResponse};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::num::{NonZeroU32, NonZeroU8};
use tracing::instrument;

use crate::data::ProcessedAgentId;
use crate::{control::ws, data::ProcessedAgent, service};

#[instrument(skip(subs, pool))]
#[post("/api/processed-agent-data")]
pub async fn create_processed_agent_data_list(
    data: web::Json<Vec<ProcessedAgent>>,
    subs: web::Data<ws::Subscribers>,
    pool: web::Data<sqlx::PgPool>,
) -> actix_web::Result<HttpResponse> {
    let ids = service::create_processed_agent_data_list(data.into_inner(), &subs, &pool).await?;
    let mut response = HttpResponse::Created();
    for id in ids {
        response.append_header((header::LOCATION, format!("/processed-agent-data/{id}")));
    }
    Ok(response.finish())
}

#[instrument(skip(subs, pool))]
#[post("/api/processed-agent-data")]
pub async fn create_processed_agent_data(
    data: web::Json<ProcessedAgent>,
    subs: web::Data<ws::Subscribers>,
    pool: web::Data<sqlx::PgPool>,
) -> actix_web::Result<Redirect> {
    let id = service::create_processed_agent_data(data.into_inner(), &subs, &pool).await?;
    Ok(Redirect::to(format!("/processed-agent-data/{id}")).using_status_code(StatusCode::CREATED))
}

#[instrument(skip(pool))]
#[get("/api/processed-agent-data/{id}")]
pub async fn read_processed_agent_data(
    id: web::Path<ProcessedAgentId>,
    pool: web::Data<sqlx::PgPool>,
) -> actix_web::Result<Option<web::Json<ProcessedAgent>>> {
    let result = service::fetch_processed_agent_data(id.into_inner(), &pool).await?;
    Ok(result.map(web::Json))
}

#[instrument(skip(pool))]
#[get("/api/processed-agent-data")]
pub async fn read_processed_agent_data_list(
    pagination: web::Query<Option<Pagination>>,
    pool: web::Data<sqlx::PgPool>,
) -> actix_web::Result<web::Json<Vec<ProcessedAgent>>> {
    let pagination = pagination.into_inner().unwrap_or_default();
    let result =
        service::fetch_processed_agent_data_list(pagination.page, pagination.size.0, &pool).await?;
    Ok(web::Json(result))
}

#[derive(Debug, Deserialize)]
struct Pagination {
    page: NonZeroU32,
    size: PageSize,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
struct PageSize(NonZeroU8);

#[instrument(skip(pool, subs))]
#[put("/api/processed-agent-data/{id}")]
pub async fn update_processed_agent_data(
    id: web::Path<ProcessedAgentId>,
    data: web::Json<ProcessedAgent>,
    subs: web::Data<ws::Subscribers>,
    pool: web::Data<sqlx::PgPool>,
) -> actix_web::Result<HttpResponse> {
    let id = id.into_inner();
    let data = data.into_inner();
    let updated = service::update_processed_agent_data(id, data, &pool, &subs).await?;
    Ok(if updated {
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    })
}

#[instrument(skip(pool, subs))]
#[post("/api/processed-agent-data/{id}")]
pub async fn delete_processed_agent_data(
    id: web::Path<ProcessedAgentId>,
    subs: web::Data<ws::Subscribers>,
    pool: web::Data<sqlx::PgPool>,
) -> actix_web::Result<HttpResponse> {
    let id = id.into_inner();
    let _ = service::delete_processed_agent_data(id, &pool, &subs).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl Default for Pagination {
    #[inline(always)]
    fn default() -> Self {
        Pagination {
            page: NonZeroU32::MIN,
            size: PageSize::default(),
        }
    }
}

impl Default for PageSize {
    #[inline(always)]
    fn default() -> Self {
        PageSize(unsafe { NonZeroU8::new(5).unwrap_unchecked() })
    }
}

impl fmt::Display for PageSize {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.0,
            if self.0 == NonZeroU8::MIN {
                "item"
            } else {
                "items"
            }
        )
    }
}

impl From<PageSize> for NonZeroU8 {
    #[inline(always)]
    fn from(PageSize(value): PageSize) -> Self {
        value
    }
}

impl<'de> Deserialize<'de> for PageSize {
    fn deserialize<D>(deserializer: D) -> Result<PageSize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = NonZeroU8::deserialize(deserializer)?;
        match value.get() {
            ..=20 => Ok(PageSize(value)),
            _ => Err(serde::de::Error::custom(
                "page size must be between 1 and 20",
            )),
        }
    }
}
