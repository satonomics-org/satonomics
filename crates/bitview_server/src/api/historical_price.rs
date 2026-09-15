use axum::{http::HeaderMap, response::Response};
use bitview_query::RepresentationId;
use brk_types::{Timestamp, Version};
use serde_json::to_vec;

use crate::{
    AppState, CacheParams, CdnCacheMode,
    error::Result,
    extended::{HeaderMapExtended, ResponseExtended},
    raw_body::RawBodyPermit,
};

pub async fn serve(
    state: AppState,
    headers: HeaderMap,
    timestamp: Option<Timestamp>,
) -> Result<Response> {
    let cache = state.historical_price_cache.clone();
    let bodies = state.historical_price_bodies.clone();
    state
        .read_body(
            &state.sync_query,
            &state.historical_price_bodies,
            move |query, permit| {
                let source = query.resolve_historical_price()?;
                let (bytes, identity) = if timestamp.is_none() {
                    cache.get_or_try_init(source.revision(), || source.get(None))?
                } else {
                    let bytes = to_vec(&source.get(timestamp)?)?;
                    let identity = RepresentationId::content(&bytes);
                    (bytes.into(), identity)
                };
                let params = CacheParams::resolve(
                    &AppState::representation_strategy(Version::ONE, identity),
                    CdnCacheMode::Live,
                );
                if params.matches_etag(&headers) {
                    return Ok(Some(Response::new_not_modified(&params)));
                }
                let Some(permit) = permit.or_else(|| RawBodyPermit::try_acquire(&bodies)) else {
                    return Ok(None);
                };
                Ok(Some(permit.response(
                    params,
                    bytes,
                    HeaderMapExtended::insert_content_type_application_json,
                )))
            },
        )
        .await
        .map_err(Into::into)
}
