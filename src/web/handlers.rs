//! Web page handlers for MemScope server
//!
//! Provides HTML page handlers for the web interface

use axum::{
    extract::State,
    response::Html,
};
use std::sync::Arc;

use super::server::ServerState;

/// Main dashboard page
pub async fn index(State(_state): State<Arc<ServerState>>) -> Html<String> {
    // Load the dashboard template from file
    let template_content = std::fs::read_to_string("templates/dashboard.html")
        .unwrap_or_else(|_| create_fallback_dashboard_html());
    
    Html(template_content)
}

/// Create fallback dashboard HTML if template file is not found
fn create_fallback_dashboard_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MemScope Dashboard - Fallback</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; }
        .error { color: #e74c3c; background: #fdf2f2; padding: 15px; border-radius: 4px; margin: 20px 0; }
    </style>
</head>
<body>
    <div class="container">
        <h1>MemScope Dashboard</h1>
        <div class="error">
            <strong>Template Error:</strong> Could not load dashboard template file.
            <br>Please ensure templates/dashboard.html exists.
        </div>
        <p>This is a fallback page. The main dashboard template could not be loaded.</p>
    </div>
</body>
</html>"#.to_string()
}
