use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use prometheus::{Encoder, Registry, TextEncoder};

pub(super) fn collect_metrics(quasar_metrics: Registry, axum_metrics: PrometheusHandle) -> String {
    let mut all_families = Vec::new();
    all_families.append(&mut quasar_metrics.gather());
    all_families.append(&mut prometheus::gather());

    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(&all_families, &mut buffer).unwrap();

    // join encoded metrics and axum_metrics.render()
    String::from_utf8(buffer).unwrap() + &axum_metrics.render()
}
