#!/usr/bin/env python3
"""
🦀 Memory Analysis Web Dashboard Server

This script starts a local web server to serve the memory analysis dashboard
with multiple interfaces for different types of analysis.

Features:
- 🏠 Main Dashboard Index
- 🔒 Unsafe/FFI Analysis Dashboard  
- 📊 Memory Analysis Dashboard
- ⏱️ Lifecycle Tracking Dashboard
- 🎛️ Classic Dashboard Interface

Usage:
    python start_web_dashboard.py              # Start on port 8080
    python start_web_dashboard.py --port 3000  # Start on custom port
    python start_web_dashboard.py --no-browser # Don't auto-open browser
"""

import http.server
import socketserver
import webbrowser
import os
import sys
import threading
import time
import json
from pathlib import Path

class DashboardHandler(http.server.SimpleHTTPRequestHandler):
    """Custom handler for the dashboard server with enhanced routing"""
    
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory="web_dashboard", **kwargs)
    
    def end_headers(self):
        # Add CORS headers to allow local file access
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        # Cache control for better performance
        if self.path.endswith(('.css', '.js', '.svg', '.png', '.jpg')):
            self.send_header('Cache-Control', 'public, max-age=3600')
        super().end_headers()
    
    def do_GET(self):
        # Enhanced routing for better UX
        if self.path == '/':
            self.path = '/dashboard_index.html'
        elif self.path == '/unsafe' or self.path == '/unsafe/':
            self.path = '/unsafe_ffi_dashboard_v2.html'
        elif self.path == '/memory' or self.path == '/memory/':
            self.path = '/memory_analysis_dashboard.html'
        elif self.path == '/classic' or self.path == '/classic/':
            self.path = '/index.html'
        elif self.path == '/lifecycle' or self.path == '/lifecycle/':
            # Check if lifecycle dashboard exists, otherwise redirect to memory
            if not os.path.exists('web_dashboard/lifecycle_dashboard.html'):
                self.path = '/memory_analysis_dashboard.html'
            else:
                self.path = '/lifecycle_dashboard.html'
        
        # Serve the requested file
        return super().do_GET()
    
    def log_message(self, format, *args):
        # Enhanced logging with timestamps and colors
        timestamp = time.strftime('%H:%M:%S')
        message = format % args
        if '200' in message:
            print(f"[{timestamp}] ✅ {message}")
        elif '404' in message:
            print(f"[{timestamp}] ❌ {message}")
        else:
            print(f"[{timestamp}] 📡 {message}")

def create_dashboard_index():
    """Create an enhanced main dashboard index page"""
    index_content = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🦀 Memory Analysis Dashboard - Home</title>
    <link rel="stylesheet" href="styles.css">
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 40px 20px;
        }
        
        .header {
            text-align: center;
            color: white;
            margin-bottom: 50px;
        }
        
        .header h1 {
            font-size: 3rem;
            margin-bottom: 10px;
            text-shadow: 0 2px 4px rgba(0,0,0,0.3);
        }
        
        .header p {
            font-size: 1.2rem;
            opacity: 0.9;
        }
        
        .dashboard-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
            gap: 30px;
            margin-top: 40px;
        }
        
        .dashboard-card {
            background: white;
            border-radius: 16px;
            padding: 30px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
            text-decoration: none;
            color: inherit;
            position: relative;
            overflow: hidden;
        }
        
        .dashboard-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 20px 40px rgba(0,0,0,0.3);
        }
        
        .dashboard-card::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 4px;
            background: linear-gradient(90deg, #667eea, #764ba2);
        }
        
        .card-icon {
            font-size: 3rem;
            margin-bottom: 20px;
            display: block;
        }
        
        .card-title {
            font-size: 1.5rem;
            font-weight: bold;
            margin-bottom: 15px;
            color: #2c3e50;
        }
        
        .card-description {
            color: #666;
            line-height: 1.6;
            margin-bottom: 20px;
        }
        
        .card-features {
            list-style: none;
            padding: 0;
            margin: 0;
        }
        
        .card-features li {
            padding: 5px 0;
            color: #7f8c8d;
            font-size: 0.9rem;
        }
        
        .card-features li:before {
            content: "✓ ";
            color: #27ae60;
            font-weight: bold;
        }
        
        .status-indicator {
            display: inline-block;
            width: 10px;
            height: 10px;
            border-radius: 50%;
            margin-right: 8px;
            animation: pulse 2s infinite;
        }
        
        @keyframes pulse {
            0% { opacity: 1; }
            50% { opacity: 0.5; }
            100% { opacity: 1; }
        }
        
        .status-ready { background: #27ae60; }
        .status-beta { background: #f39c12; }
        .status-dev { background: #e74c3c; }
        
        .footer {
            text-align: center;
            color: white;
            margin-top: 60px;
            opacity: 0.8;
        }
        
        .data-status {
            background: rgba(255,255,255,0.1);
            border-radius: 12px;
            padding: 25px;
            margin-bottom: 30px;
            color: white;
            backdrop-filter: blur(10px);
        }
        
        .data-status h3 {
            margin-top: 0;
            display: flex;
            align-items: center;
            gap: 10px;
        }
        
        .quick-links {
            display: flex;
            justify-content: center;
            gap: 15px;
            margin-top: 30px;
            flex-wrap: wrap;
        }
        
        .quick-link {
            background: rgba(255,255,255,0.2);
            color: white;
            padding: 10px 20px;
            border-radius: 25px;
            text-decoration: none;
            font-size: 0.9rem;
            transition: background 0.3s ease;
        }
        
        .quick-link:hover {
            background: rgba(255,255,255,0.3);
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔍 Memory Analysis Dashboard</h1>
            <p>Comprehensive Rust memory tracking and analysis suite</p>
            
            <div class="quick-links">
                <a href="/unsafe" class="quick-link">🔒 Unsafe/FFI</a>
                <a href="/memory" class="quick-link">📊 Memory</a>
                <a href="/lifecycle" class="quick-link">⏱️ Lifecycle</a>
                <a href="/classic" class="quick-link">🎛️ Classic</a>
            </div>
        </div>
        
        <div class="data-status">
            <h3>📊 Data Status</h3>
            <p id="data-status-text">🔄 Checking for analysis data...</p>
        </div>
        
        <div class="dashboard-grid">
            <a href="unsafe_ffi_dashboard_v2.html" class="dashboard-card">
                <span class="card-icon">🔒</span>
                <div class="card-title">
                    <span class="status-indicator status-ready"></span>
                    Unsafe/FFI Analysis
                </div>
                <div class="card-description">
                    Monitor unsafe operations and foreign function interface calls with real-time risk assessment and memory violation detection.
                </div>
                <ul class="card-features">
                    <li>Unsafe operation tracking</li>
                    <li>FFI call monitoring</li>
                    <li>Memory violation detection</li>
                    <li>Risk scoring system</li>
                    <li>SVG export support</li>
                </ul>
            </a>
            
            <a href="memory_analysis_dashboard.html" class="dashboard-card">
                <span class="card-icon">📊</span>
                <div class="card-title">
                    <span class="status-indicator status-ready"></span>
                    Memory Analysis
                </div>
                <div class="card-description">
                    Comprehensive memory usage analysis with allocation tracking, performance metrics, and efficiency monitoring.
                </div>
                <ul class="card-features">
                    <li>Memory usage timeline</li>
                    <li>Allocation size distribution</li>
                    <li>Performance metrics</li>
                    <li>Memory efficiency tracking</li>
                    <li>Interactive charts</li>
                </ul>
            </a>
            
            <a href="unsafe_ffi_dashboard.html" class="dashboard-card">
                <span class="card-icon">🛡️</span>
                <div class="card-title">
                    <span class="status-indicator status-ready"></span>
                    Unsafe/FFI (Classic)
                </div>
                <div class="card-description">
                    Original unsafe operations dashboard with detailed memory safety analysis and visualization.
                </div>
                <ul class="card-features">
                    <li>Classic interface design</li>
                    <li>Detailed safety metrics</li>
                    <li>Memory violation tracking</li>
                    <li>Risk assessment</li>
                    <li>Legacy compatibility</li>
                </ul>
            </a>
            
            <a href="index.html" class="dashboard-card">
                <span class="card-icon">🎛️</span>
                <div class="card-title">
                    <span class="status-indicator status-ready"></span>
                    Classic Dashboard
                </div>
                <div class="card-description">
                    Original dashboard interface with basic memory tracking and visualization features for simple analysis.
                </div>
                <ul class="card-features">
                    <li>Basic memory tracking</li>
                    <li>Simple visualization</li>
                    <li>Legacy compatibility</li>
                    <li>Lightweight interface</li>
                    <li>Quick overview</li>
                </ul>
            </a>
        </div>
        
        <div class="footer">
            <p>🦀 Rust Memory Analysis Dashboard | Built with ❤️ for memory safety</p>
            <p>Server running on <span id="server-info"></span></p>
            <p><small>Use Ctrl+C to stop the server</small></p>
        </div>
    </div>
    
    <script>
        // Check data availability
        async function checkDataStatus() {
            try {
                const response = await fetch('data.json');
                if (response.ok) {
                    const data = await response.json();
                    const allocCount = data.memory_stats?.total_allocations || 0;
                    const unsafeOps = data.unsafe_stats?.total_operations || 0;
                    
                    document.getElementById('data-status-text').innerHTML = 
                        `✅ Analysis data loaded successfully<br>` +
                        `📈 ${allocCount.toLocaleString()} allocations tracked<br>` +
                        `🔒 ${unsafeOps.toLocaleString()} unsafe operations monitored<br>` +
                        `🕒 Last updated: ${new Date().toLocaleString()}`;
                } else {
                    throw new Error('Data not found');
                }
            } catch (error) {
                document.getElementById('data-status-text').innerHTML = 
                    `⚠️ No analysis data found. Run a Rust example to generate data:<br>` +
                    `<code style="background: rgba(255,255,255,0.2); padding: 4px 8px; border-radius: 4px;">cargo run --example unsafe_ffi_demo</code><br>` +
                    `<code style="background: rgba(255,255,255,0.2); padding: 4px 8px; border-radius: 4px;">cargo run --example memory_stress_test</code>`;
            }
        }
        
        // Update server info
        document.getElementById('server-info').textContent = window.location.host;
        
        // Check data status on load
        checkDataStatus();
        
        // Refresh data status every 30 seconds
        setInterval(checkDataStatus, 30000);
    </script>
</body>
</html>"""
    
    with open("web_dashboard/dashboard_index.html", "w", encoding="utf-8") as f:
        f.write(index_content)

def check_data_files():
    """Check if required data files exist and return status info"""
    data_file = Path("web_dashboard/data.json")
    status = {
        'has_data': data_file.exists(),
        'data_size': 0,
        'last_modified': None
    }
    
    if data_file.exists():
        status['data_size'] = data_file.stat().st_size
        status['last_modified'] = data_file.stat().st_mtime
    else:
        print("⚠️  Warning: data.json not found in web_dashboard/")
        print("   Run a Rust example to generate data:")
        print("   🔒 cargo run --example unsafe_ffi_demo")
        print("   📊 cargo run --example memory_stress_test")
        print("   ⏱️  cargo run --example lifecycles_simple")
        print()
    
    return status

def find_available_port(start_port=8080):
    """Find an available port starting from start_port"""
    import socket
    
    for port in range(start_port, start_port + 100):
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.bind(('localhost', port))
                return port
        except OSError:
            continue
    
    raise RuntimeError("No available ports found")

def list_available_dashboards():
    """List all available dashboard files"""
    dashboard_dir = Path("web_dashboard")
    dashboards = []
    
    # Define known dashboards with descriptions
    known_dashboards = {
        'dashboard_index.html': '🏠 Main Dashboard Index',
        'index.html': '🎛️ Classic Dashboard',
        'memory_analysis_dashboard.html': '📊 Memory Analysis Dashboard',
        'unsafe_ffi_dashboard.html': '🔒 Unsafe/FFI Dashboard (Classic)',
        'unsafe_ffi_dashboard_v2.html': '🔒 Unsafe/FFI Dashboard (Enhanced)',
    }
    
    for file_name, description in known_dashboards.items():
        file_path = dashboard_dir / file_name
        if file_path.exists():
            dashboards.append((file_name, description, file_path.stat().st_size))
    
    return dashboards

def start_server(port=8080, auto_open=True):
    """Start the enhanced dashboard server"""
    
    # Ensure we're in the right directory
    if not os.path.exists("web_dashboard"):
        print("❌ Error: web_dashboard directory not found!")
        print("   Make sure you're running this script from the project root.")
        sys.exit(1)
    
    # Create dashboard index if it doesn't exist
    if not os.path.exists("web_dashboard/dashboard_index.html"):
        print("📝 Creating enhanced dashboard index...")
        create_dashboard_index()
    
    # Check for data files
    data_status = check_data_files()
    
    # List available dashboards
    dashboards = list_available_dashboards()
    
    # Find available port
    try:
        port = find_available_port(port)
    except RuntimeError as e:
        print(f"❌ Error: {e}")
        sys.exit(1)
    
    # Start server
    try:
        with socketserver.TCPServer(("", port), DashboardHandler) as httpd:
            server_url = f"http://localhost:{port}"
            
            print("🚀 Memory Analysis Dashboard Server")
            print("=" * 60)
            print(f"📡 Server running at: {server_url}")
            print(f"📁 Serving from: {os.path.abspath('web_dashboard')}")
            print(f"📊 Data available: {'✅ Yes' if data_status['has_data'] else '❌ No'}")
            if data_status['has_data']:
                size_kb = data_status['data_size'] / 1024
                print(f"📦 Data size: {size_kb:.1f} KB")
            print()
            
            print("Available dashboards:")
            print(f"  🏠 Main Dashboard:     {server_url}/")
            print(f"  🔒 Unsafe/FFI (New):   {server_url}/unsafe")
            print(f"  📊 Memory Analysis:    {server_url}/memory") 
            print(f"  🎛️  Classic Dashboard:  {server_url}/classic")
            print()
            
            print("Direct links:")
            for file_name, description, size in dashboards:
                size_kb = size / 1024
                print(f"  {description}: {server_url}/{file_name} ({size_kb:.1f} KB)")
            print()
            
            print("Quick commands:")
            print("  Press Ctrl+C to stop the server")
            print("  Visit any URL above to access the dashboards")
            print("=" * 60)
            
            # Open browser automatically
            if auto_open:
                def open_browser():
                    time.sleep(1.5)  # Wait a moment for server to start
                    try:
                        webbrowser.open(server_url)
                        print(f"🌐 Opened {server_url} in your default browser")
                    except Exception as e:
                        print(f"⚠️  Could not open browser automatically: {e}")
                        print(f"   Please manually visit: {server_url}")
                
                browser_thread = threading.Thread(target=open_browser)
                browser_thread.daemon = True
                browser_thread.start()
            
            # Start serving
            httpd.serve_forever()
            
    except KeyboardInterrupt:
        print("\n👋 Server stopped by user")
        print("   Thanks for using the Memory Analysis Dashboard!")
    except Exception as e:
        print(f"❌ Server error: {e}")
        sys.exit(1)

def main():
    """Main function with enhanced CLI"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="🦀 Start the Memory Analysis Dashboard web server",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python start_web_dashboard.py                    # Start on port 8080
  python start_web_dashboard.py --port 3000        # Start on port 3000  
  python start_web_dashboard.py --no-browser       # Don't open browser
  python start_web_dashboard.py --port 8080        # Explicit port

Available Dashboards:
  🏠 Main Dashboard Index - Overview of all dashboards
  🔒 Unsafe/FFI Analysis - Monitor unsafe operations and FFI calls
  📊 Memory Analysis - Comprehensive memory usage tracking  
  🎛️ Classic Dashboard - Original simple interface

Data Generation:
  Run these commands to generate analysis data:
  cargo run --example unsafe_ffi_demo
  cargo run --example memory_stress_test
  cargo run --example lifecycles_simple
        """
    )
    
    parser.add_argument(
        "--port", "-p",
        type=int,
        default=8080,
        help="Port to run the server on (default: 8080)"
    )
    
    parser.add_argument(
        "--no-browser",
        action="store_true",
        help="Don't automatically open the browser"
    )
    
    parser.add_argument(
        "--version", "-v",
        action="version",
        version="Memory Analysis Dashboard Server v2.0"
    )
    
    args = parser.parse_args()
    
    start_server(port=args.port, auto_open=not args.no_browser)

if __name__ == "__main__":
    main()