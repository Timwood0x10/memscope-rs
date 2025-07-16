use crate::tracker::MemoryTracker;
use std::fs;

/// Export interactive HTML dashboard with embedded SVG charts
pub fn export_interactive_dashboard<P: AsRef<std::path::Path>>(tracker: &MemoryTracker, path: P) -> crate::types::TrackingResult<()> {
    tracker.export_interactive_dashboard_impl(path)
}

impl MemoryTracker {
    /// Export interactive HTML dashboard with embedded SVG charts (internal implementation)
    pub fn export_interactive_dashboard_impl<P: AsRef<std::path::Path>>(&self, path: P) -> crate::types::TrackingResult<()> {
        let path = path.as_ref();
        
        println!("🚀 Generating interactive dashboard...");
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        // Generate SVG content (reuse existing functions)
        println!("📊 Generating lifecycle timeline...");
        let lifecycle_svg = self.generate_lifecycle_timeline_svg_content()?;
        
        println!("🧠 Generating memory analysis...");
        let memory_svg = self.generate_memory_analysis_svg_content()?;
        
        println!("⚠️ Generating unsafe FFI dashboard...");
        let unsafe_svg = self.generate_unsafe_ffi_svg_content()?;
        
        // Generate interactive script with data
        println!("⚡ Generating interactive features...");
        let interactive_script = self.generate_interactive_script()?;
        
        // Read HTML template
        let html_template = include_str!("dashboard_template.html");
        
        // Replace placeholders
        let html = html_template
            .replace("{{TIMESTAMP}}", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .replace("{{LIFECYCLE_SVG}}", &lifecycle_svg)
            .replace("{{MEMORY_ANALYSIS_SVG}}", &memory_svg)
            .replace("{{UNSAFE_FFI_SVG}}", &unsafe_svg)
            .replace("{{INTERACTIVE_SCRIPT}}", &interactive_script);
        
        // Write to file
        fs::write(path, html)?;
        
        println!("✅ Interactive dashboard exported to: {}", path.display());
        println!("   📱 Open in browser to view the analysis");
        println!("   🎯 Features: zoom, hover details, theme toggle, data export");
        
        Ok(())
    }
    
    /// Generate lifecycle timeline SVG content with interactive elements
    fn generate_lifecycle_timeline_svg_content(&self) -> crate::types::TrackingResult<String> {
        let active_allocations = self.get_active_allocations()?;
        let stats = self.get_stats()?;
        
        // Generate interactive SVG with data attributes
        let svg_content = self.create_interactive_lifecycle_svg(&active_allocations, &stats)?;
        Ok(svg_content)
    }
    
    /// Generate memory analysis SVG content with interactive elements
    fn generate_memory_analysis_svg_content(&self) -> crate::types::TrackingResult<String> {
        let active_allocations = self.get_active_allocations()?;
        let stats = self.get_stats()?;
        
        // Generate interactive SVG with data attributes
        let svg_content = self.create_interactive_memory_svg(&active_allocations, &stats)?;
        Ok(svg_content)
    }
    
    /// Generate unsafe FFI SVG content
    fn generate_unsafe_ffi_svg_content(&self) -> crate::types::TrackingResult<String> {
        let unsafe_stats = crate::unsafe_ffi_tracker::get_global_unsafe_tracker()
            .map(|tracker| tracker.get_stats())
            .unwrap_or_default();
            
        // Generate a simple placeholder SVG for now
        let svg_content = format!(r##"
<svg width="800" height="600" xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" style="stop-color:#ff6b6b;stop-opacity:0.1" />
            <stop offset="100%" style="stop-color:#ee5a24;stop-opacity:0.1" />
        </linearGradient>
    </defs>
    <rect width="100%" height="100%" fill="url(#bg)"/>
    <text x="400" y="50" text-anchor="middle" font-family="Arial" font-size="24" font-weight="bold" fill="#2c3e50">
        🔒 Unsafe FFI Dashboard
    </text>
    <text x="400" y="100" text-anchor="middle" font-family="Arial" font-size="16" fill="#7f8c8d">
        Total Operations: {}
    </text>
    <text x="400" y="130" text-anchor="middle" font-family="Arial" font-size="16" fill="#7f8c8d">
        High Risk Operations: {}
    </text>
    <rect x="50" y="200" width="700" height="300" fill="none" stroke="#bdc3c7" stroke-width="2" rx="10"/>
    <text x="400" y="350" text-anchor="middle" font-family="Arial" font-size="14" fill="#95a5a6">
        Detailed unsafe FFI analysis will be displayed here
    </text>
</svg>"##, 
            unsafe_stats.operations.len(),
            unsafe_stats.operations.iter().filter(|op| matches!(op.risk_level, crate::unsafe_ffi_tracker::RiskLevel::High | crate::unsafe_ffi_tracker::RiskLevel::Critical)).count()
        );
        
        Ok(svg_content)
    }
    
    /// Generate interactive JavaScript with embedded data
    fn generate_interactive_script(&self) -> crate::types::TrackingResult<String> {
        // Get comprehensive data
        let active_allocations = self.get_active_allocations()?;
        let allocation_history = self.get_allocation_history()?;
        let stats = self.get_stats()?;
        // Use lightweight approach - reference JSON file instead of embedding
        let timeline_data = serde_json::json!({
            "data_source": "dashboard_data.json",
            "load_external": true
        });
        
        // Build lightweight data object - only essential stats
        let dashboard_data = serde_json::json!({
            "metadata": {
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "version": env!("CARGO_PKG_VERSION"),
                "data_source": "dashboard_data.json"
            },
            "stats": {
                "total_allocations": stats.total_allocations,
                "total_deallocations": stats.total_deallocations,
                "active_allocations": stats.active_allocations,
                "peak_allocations": stats.peak_allocations,
                "total_allocated": stats.total_allocated,
                "total_deallocated": stats.total_deallocated,
                "active_memory": stats.active_memory,
                "peak_memory": stats.peak_memory
            },
            "load_external_data": true
        });
        
        let script = format!(r#"
            // Lightweight embedded data - full data loaded from JSON
            const memoryData = {};
            
            // Load full data from external JSON file
            let fullMemoryData = null;
            
            async function loadExternalData() {{
                try {{
                    const response = await fetch('dashboard_data.json');
                    if (!response.ok) {{
                        throw new Error(`HTTP error! status: ${{response.status}}`);
                    }}
                    fullMemoryData = await response.json();
                    console.log('📊 External data loaded:', fullMemoryData);
                    
                    // Update charts with full data
                    if (fullMemoryData.timeline) {{
                        renderTimelineChart();
                        renderHotspots();
                        renderMemoryGrowthChart();
                        renderFlameGraph();
                        renderVariableRelationshipGraph();
                    }}
                    
                    showNotification('📊 Full data loaded successfully!');
                }} catch (error) {{
                    console.error('Failed to load external data:', error);
                    showNotification('⚠️ Using embedded data only');
                    // Fallback to embedded data
                    fullMemoryData = memoryData;
                }}
            }}
            
            // Get data with fallback
            function getMemoryData() {{
                return fullMemoryData || memoryData;
            }}
            
            // Theme management
            let isDarkTheme = false;
            function toggleTheme() {{
                isDarkTheme = !isDarkTheme;
                document.body.classList.toggle('dark-theme', isDarkTheme);
                localStorage.setItem('darkTheme', isDarkTheme);
            }}
            
            // Load saved theme
            function loadTheme() {{
                const saved = localStorage.getItem('darkTheme');
                if (saved === 'true') {{
                    isDarkTheme = true;
                    document.body.classList.add('dark-theme');
                }}
            }}
            
            // Export functionality
            function exportReport() {{
                const content = document.documentElement.outerHTML;
                const blob = new Blob([content], {{type: 'text/html'}});
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `memory_report_${{new Date().toISOString().slice(0,19).replace(/:/g,'-')}}.html`;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
                
                // Show feedback
                showNotification('📄 Report exported successfully!');
            }}
            
            // Reset zoom for all SVGs
            function resetZoom() {{
                document.querySelectorAll('svg').forEach(svg => {{
                    svg.style.transform = 'scale(1)';
                    svg.style.transformOrigin = 'center';
                }});
                showNotification('🔍 Zoom reset');
            }}
            
            // Show data summary modal
            function showDataSummary() {{
                const modalTitle = document.getElementById('modal-title');
                const modalBody = document.getElementById('modal-body');
                
                modalTitle.textContent = '📊 Memory Analysis Summary';
                modalBody.innerHTML = `
                    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-bottom: 20px;">
                        <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; text-align: center;">
                            <div style="font-size: 24px; font-weight: bold; color: #4CAF50;">${{formatBytes(memoryData.stats.total_allocated)}}</div>
                            <div style="color: #666; margin-top: 5px;">Total Allocated</div>
                        </div>
                        <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; text-align: center;">
                            <div style="font-size: 24px; font-weight: bold; color: #2196F3;">${{formatBytes(memoryData.stats.active_memory)}}</div>
                            <div style="color: #666; margin-top: 5px;">Active Memory</div>
                        </div>
                        <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; text-align: center;">
                            <div style="font-size: 24px; font-weight: bold; color: #FF9800;">${{memoryData.stats.total_allocations.toLocaleString()}}</div>
                            <div style="color: #666; margin-top: 5px;">Total Allocations</div>
                        </div>
                        <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; text-align: center;">
                            <div style="font-size: 24px; font-weight: bold; color: #9C27B0;">${{memoryData.stats.active_allocations.toLocaleString()}}</div>
                            <div style="color: #666; margin-top: 5px;">Active Allocations</div>
                        </div>
                    </div>
                    
                    <h4>📈 Timeline Summary</h4>
                    <table class="data-table">
                        <tr><th>Memory Snapshots</th><td>${{memoryData.timeline.memory_snapshots.length}}</td></tr>
                        <tr><th>Allocation Events</th><td>${{memoryData.timeline.allocation_events.length}}</td></tr>
                        <tr><th>Stack Traces</th><td>${{Object.keys(memoryData.timeline.stack_traces.traces).length}}</td></tr>
                        <tr><th>Hotspots</th><td>${{memoryData.timeline.allocation_hotspots.length}}</td></tr>
                    </table>
                    
                    <h4>🔍 Top Allocations</h4>
                    <table class="data-table">
                        <thead>
                            <tr><th>Variable</th><th>Type</th><th>Size</th><th>Scope</th></tr>
                        </thead>
                        <tbody>
                            ${{memoryData.allocations.slice(0, 5).map(alloc => `
                                <tr>
                                    <td>${{alloc.var_name || 'unknown'}}</td>
                                    <td>${{alloc.type_name || 'unknown'}}</td>
                                    <td>${{formatBytes(alloc.size)}}</td>
                                    <td>${{alloc.scope_name || 'global'}}</td>
                                </tr>
                            `).join('')}}
                        </tbody>
                    </table>
                `;
                
                showModal();
            }}
            
            // SVG interaction enhancements
            function enhanceSVGInteractivity() {{
                // Add zoom functionality to all SVGs
                document.querySelectorAll('svg').forEach(svg => {{
                    let scale = 1;
                    let isDragging = false;
                    let startX, startY, translateX = 0, translateY = 0;
                    
                    // Mouse wheel zoom
                    svg.addEventListener('wheel', function(e) {{
                        e.preventDefault();
                        const rect = this.getBoundingClientRect();
                        const x = e.clientX - rect.left;
                        const y = e.clientY - rect.top;
                        
                        const delta = e.deltaY > 0 ? 0.9 : 1.1;
                        scale = Math.max(0.5, Math.min(3, scale * delta));
                        
                        this.style.transform = `scale(${{scale}}) translate(${{translateX}}px, ${{translateY}}px)`;
                        this.style.transformOrigin = `${{x}}px ${{y}}px`;
                    }});
                    
                    // Pan functionality
                    svg.addEventListener('mousedown', function(e) {{
                        if (e.button === 0) {{ // Left mouse button
                            isDragging = true;
                            startX = e.clientX - translateX;
                            startY = e.clientY - translateY;
                            this.style.cursor = 'grabbing';
                        }}
                    }});
                    
                    svg.addEventListener('mousemove', function(e) {{
                        if (isDragging) {{
                            translateX = e.clientX - startX;
                            translateY = e.clientY - startY;
                            this.style.transform = `scale(${{scale}}) translate(${{translateX}}px, ${{translateY}}px)`;
                        }}
                    }});
                    
                    svg.addEventListener('mouseup', function() {{
                        isDragging = false;
                        this.style.cursor = 'grab';
                    }});
                    
                    svg.addEventListener('mouseleave', function() {{
                        isDragging = false;
                        this.style.cursor = 'default';
                    }});
                    
                    svg.style.cursor = 'grab';
                }});
                
                // Enhanced hover effects for interactive elements
                document.querySelectorAll('.timeline-bar, .memory-segment, .risk-indicator, [data-info]').forEach(element => {{
                    element.addEventListener('mouseenter', function() {{
                        this.style.filter = 'brightness(1.1) drop-shadow(0 4px 8px rgba(0,0,0,0.2))';
                        this.style.transform = 'scale(1.02)';
                    }});
                    
                    element.addEventListener('mouseleave', function() {{
                        this.style.filter = '';
                        this.style.transform = '';
                    }});
                    
                    // Click to show details
                    element.addEventListener('click', function(e) {{
                        e.stopPropagation();
                        const info = this.getAttribute('data-info');
                        if (info) {{
                            try {{
                                const data = JSON.parse(info);
                                showDetailModal(data);
                            }} catch (err) {{
                                showDetailModal({{ title: 'Information', content: info }});
                            }}
                        }}
                    }});
                }});
            }}
            
            // Modal management
            function showModal() {{
                const modal = document.getElementById('modal');
                modal.classList.add('show');
            }}
            
            function closeModal() {{
                const modal = document.getElementById('modal');
                modal.classList.remove('show');
            }}
            
            function showDetailModal(data) {{
                const modalTitle = document.getElementById('modal-title');
                const modalBody = document.getElementById('modal-body');
                
                modalTitle.textContent = data.title || 'Details';
                
                if (typeof data === 'object') {{
                    // Enhanced modal content with better data visualization
                    if (data.type && data.size) {{
                        // Variable details modal
                        modalBody.innerHTML = `
                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-bottom: 20px;">
                                <div style="background: linear-gradient(135deg, #4CAF50, #45a049); color: white; padding: 20px; border-radius: 12px; text-align: center;">
                                    <div style="font-size: 24px; font-weight: bold;">${{formatBytes(data.size)}}</div>
                                    <div style="opacity: 0.9; margin-top: 5px;">Memory Size</div>
                                </div>
                                <div style="background: linear-gradient(135deg, #2196F3, #1976D2); color: white; padding: 20px; border-radius: 12px; text-align: center;">
                                    <div style="font-size: 24px; font-weight: bold;">${{data.type}}</div>
                                    <div style="opacity: 0.9; margin-top: 5px;">Variable Type</div>
                                </div>
                            </div>
                            
                            <div style="background: #f8f9fa; padding: 20px; border-radius: 12px; margin-bottom: 15px;">
                                <h4 style="margin: 0 0 15px 0; color: #333;">📋 Variable Information</h4>
                                <table style="width: 100%; border-collapse: collapse;">
                                    <tr><td style="padding: 8px; font-weight: bold; border-bottom: 1px solid #ddd;">Scope:</td><td style="padding: 8px; border-bottom: 1px solid #ddd;">${{data.scope}}</td></tr>
                                    <tr><td style="padding: 8px; font-weight: bold; border-bottom: 1px solid #ddd;">Thread:</td><td style="padding: 8px; border-bottom: 1px solid #ddd;">${{data.thread}}</td></tr>
                                    <tr><td style="padding: 8px; font-weight: bold; border-bottom: 1px solid #ddd;">Allocated At:</td><td style="padding: 8px; border-bottom: 1px solid #ddd;">${{new Date(data.allocated_at / 1000000).toLocaleString()}}</td></tr>
                                    ${{data.deallocated_at ? `<tr><td style="padding: 8px; font-weight: bold; border-bottom: 1px solid #ddd;">Deallocated At:</td><td style="padding: 8px; border-bottom: 1px solid #ddd;">${{new Date(data.deallocated_at / 1000000).toLocaleString()}}</td></tr>` : '<tr><td style="padding: 8px; font-weight: bold; border-bottom: 1px solid #ddd;">Status:</td><td style="padding: 8px; border-bottom: 1px solid #ddd; color: #4CAF50; font-weight: bold;">🟢 Still Active</td></tr>'}}
                                    <tr><td style="padding: 8px; font-weight: bold;">Borrow Count:</td><td style="padding: 8px;">${{data.borrow_count}}</td></tr>
                                </table>
                            </div>
                            
                            ${{data.allocated_at && data.deallocated_at ? `
                                <div style="background: #e3f2fd; padding: 15px; border-radius: 8px; border-left: 4px solid #2196F3;">
                                    <strong>⏱️ Lifetime:</strong> ${{((data.deallocated_at - data.allocated_at) / 1000000).toFixed(2)}} ms
                                </div>
                            ` : `
                                <div style="background: #e8f5e8; padding: 15px; border-radius: 8px; border-left: 4px solid #4CAF50;">
                                    <strong>💚 Active Variable:</strong> This variable is still in memory and has not been deallocated.
                                </div>
                            `}}
                        `;
                    }} else {{
                        // Fallback for other data types
                        modalBody.innerHTML = `
                            <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 15px;">
                                <pre style="margin: 0; white-space: pre-wrap; font-family: 'Monaco', 'Menlo', monospace; font-size: 12px;">${{JSON.stringify(data, null, 2)}}</pre>
                            </div>
                        `;
                    }}
                }} else {{
                    modalBody.innerHTML = `<p>${{data}}</p>`;
                }}
                
                showModal();
            }}
            
            // Utility functions
            function formatBytes(bytes) {{
                if (bytes === 0) return '0 B';
                const k = 1024;
                const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
                const i = Math.floor(Math.log(bytes) / Math.log(k));
                return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
            }}
            
            function showNotification(message) {{
                const notification = document.createElement('div');
                notification.style.cssText = `
                    position: fixed; top: 20px; right: 20px; z-index: 2000;
                    background: #4CAF50; color: white; padding: 15px 20px;
                    border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.2);
                    font-weight: 600; opacity: 0; transition: opacity 0.3s ease;
                `;
                notification.textContent = message;
                document.body.appendChild(notification);
                
                setTimeout(() => notification.style.opacity = '1', 100);
                setTimeout(() => {{
                    notification.style.opacity = '0';
                    setTimeout(() => document.body.removeChild(notification), 300);
                }}, 3000);
            }}
            
            // Modal management - fix the close button issue
            function hideModal() {{
                closeModal();
            }}
            
            // Click outside modal to close
            document.getElementById('modal').addEventListener('click', function(e) {{
                if (e.target === this) {{
                    closeModal();
                }}
            }});
            
            // Keyboard shortcuts
            document.addEventListener('keydown', function(e) {{
                if (e.key === 'Escape') {{
                    closeModal();
                }}
                if (e.key === 't' && e.ctrlKey) {{
                    e.preventDefault();
                    toggleTheme();
                }}
                if (e.key === 'r' && e.ctrlKey) {{
                    e.preventDefault();
                    resetZoom();
                }}
            }});
            
            // Render timeline chart using canvas
            function renderTimelineChart() {{
                const canvas = document.getElementById('timelineChart');
                if (!canvas) return;
                const ctx = canvas.getContext('2d');
                const data = getMemoryData();
                const snapshots = data.timeline?.memory_snapshots || [];
                
                if (!snapshots || snapshots.length === 0) {{
                    ctx.fillStyle = '#666';
                    ctx.font = '16px Arial';
                    ctx.textAlign = 'center';
                    ctx.fillText('No timeline data available', canvas.width / 2, canvas.height / 2);
                    return;
                }}
                
                // Clear canvas
                ctx.clearRect(0, 0, canvas.width, canvas.height);
                
                // Set up chart dimensions
                const padding = 60;
                const chartWidth = canvas.width - 2 * padding;
                const chartHeight = canvas.height - 2 * padding;
                
                // Find max memory for scaling
                const maxMemory = Math.max(...snapshots.map(s => s.total_memory));
                
                // Draw background
                ctx.fillStyle = 'rgba(102, 126, 234, 0.1)';
                ctx.fillRect(padding, padding, chartWidth, chartHeight);
                
                // Draw grid lines
                ctx.strokeStyle = 'rgba(0,0,0,0.1)';
                ctx.lineWidth = 1;
                for (let i = 0; i <= 10; i++) {{
                    const y = padding + (chartHeight / 10) * i;
                    ctx.beginPath();
                    ctx.moveTo(padding, y);
                    ctx.lineTo(padding + chartWidth, y);
                    ctx.stroke();
                }}
                
                // Draw memory usage line
                ctx.strokeStyle = '#4CAF50';
                ctx.lineWidth = 3;
                ctx.beginPath();
                
                snapshots.forEach((snapshot, i) => {{
                    const x = padding + (chartWidth / (snapshots.length - 1)) * i;
                    const y = padding + chartHeight - (snapshot.total_memory / maxMemory) * chartHeight;
                    
                    if (i === 0) {{
                        ctx.moveTo(x, y);
                    }} else {{
                        ctx.lineTo(x, y);
                    }}
                    
                    // Draw data points
                    ctx.fillStyle = '#4CAF50';
                    ctx.beginPath();
                    ctx.arc(x, y, 4, 0, 2 * Math.PI);
                    ctx.fill();
                }});
                
                ctx.stroke();
                
                // Draw labels
                ctx.fillStyle = '#333';
                ctx.font = '12px Arial';
                ctx.textAlign = 'center';
                ctx.fillText('Memory Usage Over Time', canvas.width / 2, 30);
                
                // Y-axis labels
                ctx.textAlign = 'right';
                for (let i = 0; i <= 5; i++) {{
                    const value = (maxMemory / 5) * i;
                    const y = padding + chartHeight - (chartHeight / 5) * i;
                    ctx.fillText(formatBytes(value), padding - 10, y + 4);
                }}
            }}
            
            // Render hotspots analysis
            function renderHotspots() {{
                const container = document.getElementById('hotspotsContainer');
                if (!container) return;
                const data = getMemoryData();
                const hotspots = data.timeline?.allocation_hotspots || [];
                const stackTraces = data.timeline?.stack_traces?.hotspots || [];
                
                // Allocation hotspots
                const hotspotsHtml = `
                    <div style="background: white; padding: 20px; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);">
                        <h3 style="margin: 0 0 15px 0; color: #333;">🔥 Allocation Hotspots</h3>
                        ${{hotspots.length > 0 ? hotspots.slice(0, 5).map(hotspot => `
                            <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 10px; border-left: 4px solid #FF6B6B;">
                                <div style="font-weight: bold; color: #333;">${{hotspot.location.function}}</div>
                                <div style="color: #666; font-size: 14px; margin: 5px 0;">
                                    📍 ${{hotspot.location.file || 'unknown'}}:${{hotspot.location.line || '?'}}
                                </div>
                                <div style="display: flex; justify-content: space-between; margin-top: 10px;">
                                    <span style="color: #4CAF50; font-weight: bold;">${{hotspot.allocation_count}} allocations</span>
                                    <span style="color: #2196F3; font-weight: bold;">${{formatBytes(hotspot.total_memory)}}</span>
                                </div>
                                <div style="background: #e3f2fd; padding: 8px; border-radius: 4px; margin-top: 8px; font-size: 12px;">
                                    Rate: ${{hotspot.allocation_rate.toFixed(1)}} allocs/sec | Pressure: ${{(hotspot.memory_pressure * 100).toFixed(1)}}%
                                </div>
                            </div>
                        `).join('') : '<p style="color: #666; text-align: center; padding: 40px;">No hotspots detected</p>'}}
                    </div>
                `;
                
                // Stack trace hotspots
                const stackHtml = `
                    <div style="background: white; padding: 20px; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);">
                        <h3 style="margin: 0 0 15px 0; color: #333;">📚 Stack Trace Analysis</h3>
                        ${{stackTraces.length > 0 ? stackTraces.slice(0, 5).map(trace => `
                            <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 10px; border-left: 4px solid #4ECDC4;">
                                <div style="font-weight: bold; color: #333;">Call Stack Pattern</div>
                                <div style="font-family: monospace; font-size: 12px; color: #666; margin: 8px 0; background: white; padding: 8px; border-radius: 4px;">
                                    ${{trace.stack_pattern.slice(0, 3).map(frame => frame.function).join(' → ')}}
                                    ${{trace.stack_pattern.length > 3 ? '...' : ''}}
                                </div>
                                <div style="display: flex; justify-content: space-between; margin-top: 10px;">
                                    <span style="color: #FF9800; font-weight: bold;">${{trace.allocation_count}} calls</span>
                                    <span style="color: #9C27B0; font-weight: bold;">Avg: ${{formatBytes(trace.average_size)}}</span>
                                </div>
                                <div style="background: #fff3e0; padding: 8px; border-radius: 4px; margin-top: 8px; font-size: 12px;">
                                    Frequency: ${{trace.frequency_per_second.toFixed(1)}} calls/sec
                                </div>
                            </div>
                        `).join('') : '<p style="color: #666; text-align: center; padding: 40px;">No stack traces available</p>'}}
                    </div>
                `;
                
                container.innerHTML = hotspotsHtml + stackHtml;
            }}
            
            // Initialize dashboard
            document.addEventListener('DOMContentLoaded', function() {{
                console.log('🚀 Memory Dashboard initialized');
                console.log('📊 Data loaded:', memoryData);
                console.log('📈 Timeline events:', memoryData.timeline.allocation_events.length);
                console.log('🔍 Stack traces:', Object.keys(memoryData.timeline.stack_traces.traces).length);
                
                loadTheme();
                enhanceSVGInteractivity();
                
                // Load external data first, then render charts
                loadExternalData().then(() => {{
                    renderTimelineChart();
                    renderHotspots();
                    renderMemoryGrowthChart();
                    renderFlameGraph();
                    renderVariableRelationshipGraph();
                }});
                
                // Show welcome notification
                setTimeout(() => {{
                    const data = getMemoryData();
                    const eventCount = data.timeline?.allocation_events?.length || 0;
                    const traceCount = Object.keys(data.timeline?.stack_traces?.traces || {{}}).length;
                    showNotification(`🎉 Dashboard loaded! ${{eventCount}} events, ${{traceCount}} stack traces`);
                }}, 2000);
                
            }});
            
            // Include advanced chart functions
            {}
            {}
            {}
            }});
        "#, 
            serde_json::to_string_pretty(&dashboard_data).map_err(|e| {
                crate::types::TrackingError::SerializationError(format!("JSON serialization failed: {}", e))
            })?,
            crate::advanced_charts::generate_memory_growth_chart(),
            crate::advanced_charts::generate_flame_graph(),
            crate::advanced_charts::generate_variable_relationship_graph()
        );
        
        Ok(script)
    }
    
    /// Create interactive lifecycle timeline SVG with clickable elements
    fn create_interactive_lifecycle_svg(&self, active_allocations: &[crate::types::AllocationInfo], stats: &crate::types::MemoryStats) -> crate::types::TrackingResult<String> {
        let mut svg = String::new();
        
        // SVG header with interactive styles
        svg.push_str(r##"<svg width="1600" height="800" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 800">
<defs>
    <linearGradient id="bgGradient" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" style="stop-color:#667eea;stop-opacity:1" />
        <stop offset="100%" style="stop-color:#764ba2;stop-opacity:1" />
    </linearGradient>
    <filter id="glow">
        <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
        <feMerge> 
            <feMergeNode in="coloredBlur"/>
            <feMergeNode in="SourceGraphic"/>
        </feMerge>
    </filter>
</defs>
<style>
    .timeline-bar { 
        cursor: pointer; 
        transition: all 0.3s ease;
        filter: drop-shadow(0 2px 4px rgba(0,0,0,0.2));
    }
    .timeline-bar:hover { 
        filter: brightness(1.2) drop-shadow(0 4px 8px rgba(0,0,0,0.4)) url(#glow);
        transform: scale(1.05);
    }
    .memory-segment {
        cursor: pointer;
        transition: all 0.2s ease;
    }
    .memory-segment:hover {
        filter: brightness(1.1);
        stroke-width: 3;
    }
    .interactive-text {
        cursor: pointer;
        transition: fill 0.2s ease;
    }
    .interactive-text:hover {
        fill: #FFD700;
    }
</style>
<rect width="100%" height="100%" fill="url(#bgGradient)"/>
"##);
        
        // Title
        svg.push_str(r##"<text x="800" y="50" text-anchor="middle" font-family="Arial, sans-serif" font-size="28" font-weight="bold" fill="white" class="interactive-text" data-info='{"title":"Memory Lifecycle Timeline","description":"Interactive visualization of variable lifecycles and memory events"}'>
🔄 Interactive Memory Lifecycle Timeline
</text>"##);
        
        // Generate timeline bars for each allocation
        let max_size = active_allocations.iter().map(|a| a.size).max().unwrap_or(1);
        let bar_width = 1400.0 / active_allocations.len().max(1) as f64;
        
        for (i, alloc) in active_allocations.iter().enumerate() {
            let x = 100.0 + i as f64 * bar_width;
            let height = (alloc.size as f64 / max_size as f64 * 600.0).max(5.0);
            let y = 700.0 - height;
            
            // Color based on size
            let color = if alloc.size > max_size / 2 {
                "#FF6B6B" // Red for large allocations
            } else if alloc.size > max_size / 4 {
                "#4ECDC4" // Teal for medium allocations
            } else {
                "#45B7D1" // Blue for small allocations
            };
            
            // Create interactive data
            let data_info = serde_json::json!({
                "title": format!("Variable: {}", alloc.var_name.as_deref().unwrap_or("unknown")),
                "type": alloc.type_name.as_deref().unwrap_or("unknown"),
                "size": alloc.size,
                "scope": alloc.scope_name.as_deref().unwrap_or("global"),
                "thread": alloc.thread_id,
                "allocated_at": alloc.timestamp_alloc,
                "deallocated_at": alloc.timestamp_dealloc,
                "borrow_count": alloc.borrow_count
            });
            
            svg.push_str(&format!(
                r##"<rect class="timeline-bar memory-segment" x="{}" y="{}" width="{}" height="{}" fill="{}" rx="4" data-info='{}'/>"##,
                x, y, bar_width - 2.0, height, color, 
                serde_json::to_string(&data_info).unwrap_or_default().replace("'", "&apos;")
            ));
            
            // Add variable name label (for larger bars)
            if height > 50.0 {
                svg.push_str(&format!(
                    r##"<text x="{}" y="{}" text-anchor="middle" font-family="Arial" font-size="10" fill="white" class="interactive-text" data-info='{}'>{}</text>"##,
                    x + bar_width / 2.0, y + height / 2.0,
                    serde_json::to_string(&data_info).unwrap_or_default().replace("'", "&apos;"),
                    alloc.var_name.as_deref().unwrap_or("?")
                ));
            }
        }
        
        // Add legend
        svg.push_str(r##"
<g transform="translate(50, 100)">
    <text x="0" y="0" font-family="Arial" font-size="16" font-weight="bold" fill="white">Legend:</text>
    <rect class="memory-segment" x="0" y="10" width="20" height="15" fill="#FF6B6B" data-info='{"title":"Large Allocations","description":"Memory allocations larger than 50% of maximum"}' rx="2"/>
    <text x="30" y="22" font-family="Arial" font-size="12" fill="white">Large Allocations</text>
    <rect class="memory-segment" x="0" y="35" width="20" height="15" fill="#4ECDC4" data-info='{"title":"Medium Allocations","description":"Memory allocations between 25% and 50% of maximum"}' rx="2"/>
    <text x="30" y="47" font-family="Arial" font-size="12" fill="white">Medium Allocations</text>
    <rect class="memory-segment" x="0" y="60" width="20" height="15" fill="#45B7D1" data-info='{"title":"Small Allocations","description":"Memory allocations smaller than 25% of maximum"}' rx="2"/>
    <text x="30" y="72" font-family="Arial" font-size="12" fill="white">Small Allocations</text>
</g>
"##);
        
        // Add statistics panel
        svg.push_str(&format!(r##"
<g transform="translate(1200, 100)">
    <rect width="350" height="200" fill="rgba(255,255,255,0.1)" stroke="rgba(255,255,255,0.3)" stroke-width="2" rx="10"/>
    <text x="175" y="25" text-anchor="middle" font-family="Arial" font-size="16" font-weight="bold" fill="white">📊 Statistics</text>
    <text x="20" y="50" font-family="Arial" font-size="14" fill="white">Total Allocations: {}</text>
    <text x="20" y="70" font-family="Arial" font-size="14" fill="white">Active Memory: {} bytes</text>
    <text x="20" y="90" font-family="Arial" font-size="14" fill="white">Peak Memory: {} bytes</text>
    <text x="20" y="110" font-family="Arial" font-size="14" fill="white">Active Variables: {}</text>
    <text x="20" y="140" font-family="Arial" font-size="12" fill="#FFD700">💡 Click on bars for details</text>
    <text x="20" y="160" font-family="Arial" font-size="12" fill="#FFD700">🔍 Scroll to zoom, drag to pan</text>
    <text x="20" y="180" font-family="Arial" font-size="12" fill="#FFD700">🎨 Use theme toggle above</text>
</g>
"##, stats.total_allocations, stats.active_memory, stats.peak_memory, active_allocations.len()));
        
        svg.push_str("</svg>");
        Ok(svg)
    }
    
    /// Create interactive memory analysis SVG with clickable elements
    fn create_interactive_memory_svg(&self, active_allocations: &[crate::types::AllocationInfo], stats: &crate::types::MemoryStats) -> crate::types::TrackingResult<String> {
        let mut svg = String::new();
        
        // SVG header
        svg.push_str(r##"<svg width="1600" height="800" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 800">
<defs>
    <radialGradient id="memoryGradient" cx="50%" cy="50%" r="50%">
        <stop offset="0%" style="stop-color:#4CAF50;stop-opacity:0.8" />
        <stop offset="100%" style="stop-color:#2E7D32;stop-opacity:0.9" />
    </radialGradient>
    <linearGradient id="bgGrad" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" style="stop-color:#1a1a2e;stop-opacity:1" />
        <stop offset="100%" style="stop-color:#16213e;stop-opacity:1" />
    </linearGradient>
</defs>
<style>
    .memory-circle { 
        cursor: pointer; 
        transition: all 0.3s ease;
    }
    .memory-circle:hover { 
        filter: brightness(1.3) drop-shadow(0 0 10px rgba(76, 175, 80, 0.8));
        transform: scale(1.1);
    }
    .type-segment {
        cursor: pointer;
        transition: all 0.2s ease;
    }
    .type-segment:hover {
        filter: brightness(1.2);
        stroke: #FFD700;
        stroke-width: 3;
    }
</style>
<rect width="100%" height="100%" fill="url(#bgGrad)"/>
"##);
        
        // Title
        svg.push_str(r##"<text x="800" y="50" text-anchor="middle" font-family="Arial, sans-serif" font-size="28" font-weight="bold" fill="white">
🧠 Interactive Memory Analysis
</text>"##);
        
        // Create memory usage pie chart
        let center_x = 400.0;
        let center_y = 400.0;
        let radius = 200.0;
        
        // Group allocations by type
        let mut type_usage: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for alloc in active_allocations {
            let type_name = alloc.type_name.as_deref().unwrap_or("unknown").to_string();
            *type_usage.entry(type_name).or_insert(0) += alloc.size;
        }
        
        let total_memory: usize = type_usage.values().sum();
        let mut current_angle: f64 = 0.0;
        let colors = ["#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F"];
        
        for (i, (type_name, size)) in type_usage.iter().enumerate() {
            let percentage = *size as f64 / total_memory as f64;
            let angle = percentage * 360.0;
            
            if angle > 1.0 { // Only show segments larger than 1 degree
                let color = colors[i % colors.len()];
                
                // Calculate arc path
                let start_angle_rad = current_angle.to_radians();
                let end_angle_rad = (current_angle + angle).to_radians();
                
                let x1 = center_x + radius * start_angle_rad.cos();
                let y1 = center_y + radius * start_angle_rad.sin();
                let x2 = center_x + radius * end_angle_rad.cos();
                let y2 = center_y + radius * end_angle_rad.sin();
                
                let large_arc = if angle > 180.0 { 1 } else { 0 };
                
                let data_info = serde_json::json!({
                    "title": format!("Type: {}", type_name),
                    "size": size,
                    "percentage": format!("{:.1}%", percentage * 100.0),
                    "allocations": active_allocations.iter().filter(|a| a.type_name.as_deref().unwrap_or("unknown") == type_name).count()
                });
                
                svg.push_str(&format!(
                    r##"<path class="type-segment memory-circle" d="M {} {} L {} {} A {} {} 0 {} 1 {} {} Z" fill="{}" data-info='{}'/>"##,
                    center_x, center_y, x1, y1, radius, radius, large_arc, x2, y2, color,
                    serde_json::to_string(&data_info).unwrap_or_default().replace("'", "&apos;")
                ));
                
                // Add label for larger segments
                if percentage > 0.05 {
                    let label_angle = (current_angle + angle / 2.0).to_radians();
                    let label_x = center_x + (radius * 0.7) * label_angle.cos();
                    let label_y = center_y + (radius * 0.7) * label_angle.sin();
                    
                    svg.push_str(&format!(
                        r##"<text x="{}" y="{}" text-anchor="middle" font-family="Arial" font-size="12" font-weight="bold" fill="white" class="interactive-text">{}</text>"##,
                        label_x, label_y, 
                        if type_name.len() > 8 { &type_name[..8] } else { type_name }
                    ));
                }
                
                current_angle += angle;
            }
        }
        
        // Add center circle with total memory
        svg.push_str(&format!(
            r##"<circle class="memory-circle" cx="{}" cy="{}" r="80" fill="rgba(255,255,255,0.9)" stroke="#4CAF50" stroke-width="4" data-info='{{"title":"Total Memory","size":{},"description":"Total active memory across all allocations"}}'/>
<text x="{}" y="{}" text-anchor="middle" font-family="Arial" font-size="16" font-weight="bold" fill="#2E7D32">Total</text>
<text x="{}" y="{}" text-anchor="middle" font-family="Arial" font-size="14" fill="#2E7D32">{} bytes</text>"##,
            center_x, center_y, total_memory,
            center_x, center_y - 10.0,
            center_x, center_y + 10.0, total_memory
        ));
        
        // Add type legend
        svg.push_str(r##"<g transform="translate(900, 150)">"##);
        svg.push_str(r##"<text x="0" y="0" font-family="Arial" font-size="18" font-weight="bold" fill="white">📋 Memory by Type</text>"##);
        
        for (i, (type_name, size)) in type_usage.iter().enumerate() {
            let y_pos = 30.0 + i as f64 * 25.0;
            let color = colors[i % colors.len()];
            let percentage = *size as f64 / total_memory as f64 * 100.0;
            
            svg.push_str(&format!(
                r##"<rect class="type-segment" x="0" y="{}" width="20" height="15" fill="{}" rx="3" data-info='{{"title":"{}","size":{},"percentage":"{:.1}%"}}'/>
<text x="30" y="{}" font-family="Arial" font-size="12" fill="white">{}: {:.1}%</text>"##,
                y_pos, color, type_name, size, percentage,
                y_pos + 12.0, type_name, percentage
            ));
        }
        svg.push_str("</g>");
        
        svg.push_str("</svg>");
        Ok(svg)
    }
}