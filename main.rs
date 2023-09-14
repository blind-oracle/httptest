use axum::{response::IntoResponse, Router};
use std::time::Instant;

use axum::routing::method_routing::get;
use axum_prometheus::lifecycle::{layer::LifeCycleLayer, Callbacks};
use tower_http::classify::{ClassifiedResponse, ServerErrorsAsFailures};

#[derive(Clone)]
pub struct MetricsData {
    pub start: Instant,
    pub method: String,
}

#[derive(Clone)]
pub struct Traffic {}

impl Traffic {
    pub(crate) fn new() -> Self {
        Traffic {}
    }
}

impl<FailureClass> Callbacks<FailureClass> for Traffic {
    type Data = Option<MetricsData>;

    fn prepare<B>(&mut self, request: &http::Request<B>) -> Self::Data {
        let now = std::time::Instant::now();
        let method = request.method().to_string();

        Some(MetricsData { start: now, method })
    }

    fn on_response<B>(
        &mut self,
        _res: &http::Response<B>,
        _cls: ClassifiedResponse<FailureClass, ()>,
        _data: &mut Self::Data,
    ) {
        println!("response generated");
    }

    fn on_eos(
        self,
        _trailers: Option<&http::HeaderMap>,
        _classification: Result<(), FailureClass>,
        _data: Self::Data,
    ) {
        println!("stream ended");
    }

    fn on_body_chunk<B: bytes::Buf>(&self, _check: &B, _data: &Self::Data) {
        println!("chunk");
    }

    fn on_failure(
        self,
        _failed_at: axum_prometheus::lifecycle::FailedAt,
        _failure_classification: FailureClass,
        _data: Self::Data,
    ) {
        println!("failed!");
    }
}

pub async fn handler() -> impl IntoResponse {
    let foo = "bar".to_string().repeat(10000);
    foo.into_response()
}

#[tokio::main]
async fn main() {
    let make_classifier = ServerErrorsAsFailures::make_classifier();
    let layer = LifeCycleLayer::new(make_classifier, Traffic::new());

    let router = Router::new().route("/", get(handler)).layer(layer);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
