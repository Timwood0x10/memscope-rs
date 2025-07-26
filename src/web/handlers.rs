use axum::{
    extract::Query,
    response::{Html, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub theme: Option<String>,
    pub debug: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

pub async fn serve_dashboard(Query(params): Query<PageQuery>) -> Result<Html<String>, StatusCode> {
    let theme = params.theme.unwrap_or_else(|| "default".to_string());
    let debug_mode = params.debug.unwrap_or(false);
    
    // This would be replaced with actual HTML template loading
    let html_content = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>MemScope Dashboard</title>
            <meta name="theme" content="{}">
            <meta name="debug" content="{}">
        </head>
        <body>
            <h1>MemScope Memory Analysis Dashboard</h1>
            <p>Theme: {}</p>
            <p>Debug Mode: {}</p>
            <div id="dashboard-content">
                <!-- Dashboard content would be loaded here -->
            </div>
        </body>
        </html>
        "#,
        theme, debug_mode, theme, debug_mode
    );
    
    Ok(Html(html_content))
}

pub async fn handle_not_found() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Resource not found".to_string(),
            code: 404,
        }),
    )
}

pub async fn handle_error(error: String) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error,
            code: 500,
        }),
    )
}