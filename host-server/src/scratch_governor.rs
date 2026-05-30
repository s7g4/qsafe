use axum::{routing::get, Router};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

pub fn build_router() -> Router {
    let config = Box::new(
        GovernorConfigBuilder::default()
            .per_second(6) // 10 per min = 1 per 6s
            .burst_size(5)
            .finish()
            .unwrap(),
    );
    let rate_limiter = GovernorLayer { config };

    Router::new()
        .route("/", get(|| async { "Hello" }))
        .layer(rate_limiter)
}
