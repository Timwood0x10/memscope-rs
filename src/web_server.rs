//! Simple HTTP web server for serving the interactive memory analysis dashboard
//! 
//! This module provides a lightweight web server that:
//! - Serves static HTML/CSS/JS files from the web_dashboard directory
//! - Provides REST API endpoints for memory data
//! - Supports real-time data updates via polling

use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;
use std::time::Duration;
use crate::types::TrackingResult;
use crate::web_export::{WebDashboardData, export_web_dashboard_data};
use crate::tracker::get_global_tracker;
use crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker;

/// Simple HTTP web server for the memory analysis dashboard
pub struct MemoryAnalysisWebServer {
    port: u16,
    web_root: String,
    data_cache: Option<WebDashboardData>,
    cache_timestamp: std::time::Instant,
    cache_duration: Duration,
}

impl MemoryAnalysisWebServer {
    /// Create a new web server instance
    pub fn new(port: u16, web_root: &str) -> Self {
        Self {
            port,
            web_root: web_root.to_string(),
            data_cache: None,
            cache_timestamp: std::time::Instant::now(),
            cache_duration: Duration::from_secs(5), // Cache for 5 seconds
        }
    }

    /// Start the web server
    pub fn start(&mut self) -> TrackingResult<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .map_err(|e| crate::types::TrackingError::IoError(e))?;
        
        println!("🌐 Memory Analysis Web Server started at http://127.0.0.1:{}", self.port);
        println!("📊 Dashboard available at: http://127.0.0.1:{}/", self.port);
        println!("🔧 API endpoints:");
        println!("   - GET /api/memory-data - Current memory analysis data");
        println!("   - GET /api/health - Server health check");
        println!("📁 Serving files from: {}", self.web_root);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.handle_connection(stream) {
                        eprintln!("Error handling connection: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handle incoming HTTP connection
    fn handle_connection(&mut self, mut stream: TcpStream) -> TrackingResult<()> {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer)
            .map_err(|e| crate::types::TrackingError::IoError(e))?;

        let request = String::from_utf8_lossy(&buffer[..]);
        let request_line = request.lines().next().unwrap_or("");
        
        // Parse HTTP request
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return self.send_error_response(stream, 400, "Bad Request");
        }

        let method = parts[0];
        let path = parts[1];

        println!("📥 {} {}", method, path);

        match (method, path) {
            ("GET", "/") => self.serve_file(stream, "index.html"),
            ("GET", "/api/memory-data") => self.serve_memory_data(stream),
            ("GET", "/api/health") => self.serve_health_check(stream),
            ("GET", path) if path.starts_with("/api/") => {
                self.send_error_response(stream, 404, "API endpoint not found")
            }
            ("GET", path) => {
                // Serve static files
                let file_path = if path == "/" { "index.html" } else { &path[1..] };
                self.serve_file(stream, file_path)
            }
            _ => self.send_error_response(stream, 405, "Method Not Allowed"),
        }
    }

    /// Serve static files from web_root directory
    fn serve_file(&self, mut stream: TcpStream, file_path: &str) -> TrackingResult<()> {
        let full_path = Path::new(&self.web_root).join(file_path);
        
        match fs::read(&full_path) {
            Ok(contents) => {
                let content_type = self.get_content_type(file_path);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                    content_type,
                    contents.len()
                );
                
                stream.write_all(response.as_bytes())
                    .map_err(|e| crate::types::TrackingError::IoError(e))?;
                stream.write_all(&contents)
                    .map_err(|e| crate::types::TrackingError::IoError(e))?;
                stream.flush()
                    .map_err(|e| crate::types::TrackingError::IoError(e))?;
            }
            Err(_) => {
                self.send_error_response(stream, 404, "File not found")?;
            }
        }
        
        Ok(())
    }

    /// Serve current memory analysis data as JSON
    fn serve_memory_data(&mut self, mut stream: TcpStream) -> TrackingResult<()> {
        // Check if we need to refresh the cache
        if self.data_cache.is_none() || self.cache_timestamp.elapsed() > self.cache_duration {
            self.refresh_data_cache()?;
        }

        let json_data = match &self.data_cache {
            Some(data) => serde_json::to_string_pretty(data)
                .map_err(|e| crate::types::TrackingError::SerializationError(e.to_string()))?,
            None => "{}".to_string(),
        };

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
            json_data.len(),
            json_data
        );

        stream.write_all(response.as_bytes())
            .map_err(|e| crate::types::TrackingError::IoError(e))?;
        stream.flush()
            .map_err(|e| crate::types::TrackingError::IoError(e))?;

        Ok(())
    }

    /// Serve health check endpoint
    fn serve_health_check(&self, mut stream: TcpStream) -> TrackingResult<()> {
        let health_data = serde_json::json!({
            "status": "healthy",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            "server": "memscope-rs web server",
            "version": env!("CARGO_PKG_VERSION")
        });

        let json_data = health_data.to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
            json_data.len(),
            json_data
        );

        stream.write_all(response.as_bytes())
            .map_err(|e| crate::types::TrackingError::IoError(e))?;
        stream.flush()
            .map_err(|e| crate::types::TrackingError::IoError(e))?;

        Ok(())
    }

    /// Send HTTP error response
    fn send_error_response(&self, mut stream: TcpStream, status_code: u16, message: &str) -> TrackingResult<()> {
        let status_text = match status_code {
            400 => "Bad Request",
            404 => "Not Found",
            405 => "Method Not Allowed",
            500 => "Internal Server Error",
            _ => "Error",
        };

        let body = format!(r#"
<!DOCTYPE html>
<html>
<head><title>{} {}</title></head>
<body>
    <h1>{} {}</h1>
    <p>{}</p>
    <hr>
    <p><em>memscope-rs web server</em></p>
</body>
</html>
"#, status_code, status_text, status_code, status_text, message);

        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            status_code,
            status_text,
            body.len(),
            body
        );

        stream.write_all(response.as_bytes())
            .map_err(|e| crate::types::TrackingError::IoError(e))?;
        stream.flush()
            .map_err(|e| crate::types::TrackingError::IoError(e))?;

        Ok(())
    }

    /// Get MIME content type for file extension
    fn get_content_type(&self, file_path: &str) -> &'static str {
        match file_path.split('.').last() {
            Some("html") => "text/html; charset=utf-8",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            _ => "text/plain",
        }
    }

    /// Refresh the cached memory data
    fn refresh_data_cache(&mut self) -> TrackingResult<()> {
        let tracker = get_global_tracker();
        let unsafe_ffi_tracker = get_global_unsafe_ffi_tracker();

        // Create temporary file for export
        let temp_path = "tmp_rovodev_web_data.json";
        export_web_dashboard_data(&tracker, &unsafe_ffi_tracker, temp_path)?;

        // Read the exported data
        let json_content = fs::read_to_string(temp_path)
            .map_err(|e| crate::types::TrackingError::IoError(e))?;
        
        let data: WebDashboardData = serde_json::from_str(&json_content)
            .map_err(|e| crate::types::TrackingError::SerializationError(e.to_string()))?;

        // Clean up temporary file
        let _ = fs::remove_file(temp_path);

        self.data_cache = Some(data);
        self.cache_timestamp = std::time::Instant::now();

        Ok(())
    }
}

/// Start the memory analysis web server in a separate thread
pub fn start_web_server_async(port: u16, web_root: &str) -> TrackingResult<thread::JoinHandle<()>> {
    let web_root = web_root.to_string();
    
    let handle = thread::spawn(move || {
        let mut server = MemoryAnalysisWebServer::new(port, &web_root);
        if let Err(e) = server.start() {
            eprintln!("Web server error: {}", e);
        }
    });

    // Give the server a moment to start
    thread::sleep(Duration::from_millis(100));
    
    Ok(handle)
}

/// Convenience function to start web server with default settings
pub fn start_memory_dashboard(port: Option<u16>) -> TrackingResult<thread::JoinHandle<()>> {
    let port = port.unwrap_or(8080);
    let web_root = "web_dashboard";
    
    println!("🚀 Starting Memory Analysis Dashboard...");
    start_web_server_async(port, web_root)
}