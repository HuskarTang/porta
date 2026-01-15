use axum::{http::StatusCode, Json};
use serde::Serialize;

use crate::response::ApiResponse;

pub fn ok<T: Serialize>(data: Option<T>) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 0,
            message: "ok".into(),
            data,
        }),
    )
}

pub fn err<T: Serialize>(message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse {
            code: 40000,
            message: message.into(),
            data: None,
        }),
    )
}
