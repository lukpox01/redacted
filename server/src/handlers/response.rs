use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};
use protocol::ErrorResponse;

pub fn json_response<T: serde::Serialize>(data: T, status: StatusCode) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(&data).unwrap();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap()
}

pub fn error_response(error: &str, status: StatusCode) -> Response<Full<Bytes>> {
    json_response(
        ErrorResponse {
            error: error.to_string(),
        },
        status,
    )
}
