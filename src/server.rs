use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::GraphQL;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_prometheus::PrometheusMetricLayer;
use log::info;
use prometheus::Registry;
use sea_orm::DatabaseConnection;

use crate::{configuration::Api, schema::build_schema, metrics::collect_metrics};

pub(crate) async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}

pub async fn serve(api: &Api, database: DatabaseConnection, metrics: Registry) {
    let schema = build_schema(api.depth_limit, api.complexity_limit, database);

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app = Router::new()
        .route(
            "/",
            get(graphql_playground).post_service(GraphQL::new(schema)),
        )
        .route(
            "/metrics",
            get(|| async move { collect_metrics(metrics, metric_handle) }),
        )
        .layer(prometheus_layer);

    let socket_addr = (api.host, api.port).into();

    tokio::spawn(async move {
        axum::Server::bind(&socket_addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    info!("API started on {}", socket_addr);
}
