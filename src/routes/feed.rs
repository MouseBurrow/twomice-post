use crate::errors::PostError;
use crate::service;
use axum::extract::{Query, State};
use axum::Json;
use config::app_data::AppData;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FeedQuery {
    #[serde(default = "default_sort")]
    pub sort: String,
}

fn default_sort() -> String {
    "hot".to_string()
}

pub async fn get_feed(
    State(app): State<AppData>,
    Query(query): Query<FeedQuery>,
) -> Result<Json<Vec<service::PostData>>, PostError> {
    let posts = service::get_feed_posts(&app.pool, &query.sort).await?;
    Ok(Json(posts))
}
