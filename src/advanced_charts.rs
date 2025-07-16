/// Advanced chart generation for HTML dashboard
use serde_json::Value;

/// Generate memory growth chart using Canvas
pub fn generate_memory_growth_chart() -> String {
    r#"
    function renderMemoryGrowthChart() {
        const canvas = document.getElementById('memoryGrowthChart');
        const ctx = canvas.getContext('2d');
        const events = memoryData.timeline.allocation_events || [];
        
        if (events.length === 0) {
            ctx.fillStyle = '#666';
            ctx.font = '16px Arial';
            ctx.textAlign = 'center';
            ctx.fillText('No memory growth data available', canvas.width / 2, canvas.height / 2);
            return;
        }
        
        // Clear canvas
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        
        // Group events by time windows to show growth
        const timeWindows = {};
        let currentMemory = 0;
        
        events.forEach(event => {
            const timeKey = Math.floor(event.timestamp / 1000000000); // 1 second windows
            if (!timeWindows[timeKey]) {
                timeWindows[timeKey] = { allocations: 0, deallocations: 0, memory: currentMemory };
            }
            
            if (event.event_type === 'Allocate') {
                timeWindows[timeKey].allocations += event.size;
                currentMemory += event.size;
            } else if (event.event_type === 'Deallocate') {
                timeWindows[timeKey].deallocations += event.size;
                currentMemory -= event.size;
            }
            timeWindows[timeKey].memory = currentMemory;
        });
        
        const windows = Object.entries(timeWindows).map(([time, data]) => ({
            time: parseInt(time),
            ...data
        })).sort((a, b) => a.time - b.time);
        
        if (windows.length === 0) return;
        
        // Chart dimensions
        const padding = 60;
        const chartWidth = canvas.width - 2 * padding;
        const chartHeight = canvas.height - 2 * padding;
        
        // Find max memory for scaling
        const maxMemory = Math.max(...windows.map(w => w.memory));
        
        // Draw background gradient
        const gradient = ctx.createLinearGradient(0, padding, 0, padding + chartHeight);
        gradient.addColorStop(0, 'rgba(76, 175, 80, 0.1)');
        gradient.addColorStop(1, 'rgba(76, 175, 80, 0.05)');
        ctx.fillStyle = gradient;
        ctx.fillRect(padding, padding, chartWidth, chartHeight);
        
        // Draw grid
        ctx.strokeStyle = 'rgba(0,0,0,0.1)';
        ctx.lineWidth = 1;
        for (let i = 0; i <= 10; i++) {
            const y = padding + (chartHeight / 10) * i;
            ctx.beginPath();
            ctx.moveTo(padding, y);
            ctx.lineTo(padding + chartWidth, y);
            ctx.stroke();
        }
        
        // Draw memory growth area
        ctx.fillStyle = 'rgba(76, 175, 80, 0.3)';
        ctx.beginPath();
        ctx.moveTo(padding, padding + chartHeight);
        
        windows.forEach((window, i) => {
            const x = padding + (chartWidth / (windows.length - 1)) * i;
            const y = padding + chartHeight - (window.memory / maxMemory) * chartHeight;
            if (i === 0) {
                ctx.lineTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
        });
        
        ctx.lineTo(padding + chartWidth, padding + chartHeight);
        ctx.closePath();
        ctx.fill();
        
        // Draw memory growth line
        ctx.strokeStyle = '#4CAF50';
        ctx.lineWidth = 3;
        ctx.beginPath();
        
        windows.forEach((window, i) => {
            const x = padding + (chartWidth / (windows.length - 1)) * i;
            const y = padding + chartHeight - (window.memory / maxMemory) * chartHeight;
            
            if (i === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
            
            // Draw data points
            ctx.fillStyle = '#2E7D32';
            ctx.beginPath();
            ctx.arc(x, y, 4, 0, 2 * Math.PI);
            ctx.fill();
        });
        
        ctx.stroke();
        
        // Add labels and title
        ctx.fillStyle = '#333';
        ctx.font = 'bold 16px Arial';
        ctx.textAlign = 'center';
        ctx.fillText('📈 Memory Growth Over Time', canvas.width / 2, 30);
        
        // Y-axis labels
        ctx.font = '12px Arial';
        ctx.textAlign = 'right';
        for (let i = 0; i <= 5; i++) {
            const value = (maxMemory / 5) * i;
            const y = padding + chartHeight - (chartHeight / 5) * i;
            ctx.fillText(formatBytes(value), padding - 10, y + 4);
        }
        
        // X-axis labels (time)
        ctx.textAlign = 'center';
        const timeStep = Math.max(1, Math.floor(windows.length / 5));
        for (let i = 0; i < windows.length; i += timeStep) {
            const x = padding + (chartWidth / (windows.length - 1)) * i;
            ctx.fillText(`${i}s`, x, canvas.height - 20);
        }
    }
    "#.to_string()
}

/// Generate flame graph for stack traces
pub fn generate_flame_graph() -> String {
    r#"
    function renderFlameGraph() {
        const container = document.getElementById('flameGraphContainer');
        const stackTraces = memoryData.timeline.stack_traces.traces || {};
        const hotspots = memoryData.timeline.stack_traces.hotspots || [];
        
        if (Object.keys(stackTraces).length === 0) {
            container.innerHTML = '<div style="text-align: center; padding: 40px; color: #666;">No stack trace data available for flame graph</div>';
            return;
        }
        
        // Build flame graph data structure
        const flameData = {};
        
        // Process hotspots to build hierarchical structure
        hotspots.forEach(hotspot => {
            let current = flameData;
            hotspot.stack_pattern.forEach((frame, depth) => {
                const key = frame.function || 'unknown';
                if (!current[key]) {
                    current[key] = {
                        name: key,
                        value: 0,
                        children: {},
                        depth: depth,
                        file: frame.file,
                        line: frame.line
                    };
                }
                current[key].value += hotspot.total_memory;
                current = current[key].children;
            });
        });
        
        // Render flame graph as nested rectangles
        let html = '<div style="font-family: monospace; font-size: 12px;">';
        html += '<h3 style="margin: 0 0 15px 0; color: #333;">🔥 Call Stack Flame Graph</h3>';
        
        function renderFlameLevel(data, level = 0, totalWidth = 800) {
            const entries = Object.values(data);
            if (entries.length === 0) return '';
            
            const totalValue = entries.reduce((sum, entry) => sum + entry.value, 0);
            let currentX = 0;
            let html = '';
            
            entries.forEach(entry => {
                const width = (entry.value / totalValue) * totalWidth;
                const color = `hsl(${(level * 60) % 360}, 70%, ${70 - level * 5}%)`;
                
                if (width > 20) { // Only show if wide enough
                    html += `
                        <div style="
                            position: relative;
                            display: inline-block;
                            width: ${width}px;
                            height: 25px;
                            background: ${color};
                            border: 1px solid rgba(0,0,0,0.2);
                            margin: 1px;
                            cursor: pointer;
                            overflow: hidden;
                            vertical-align: top;
                        " 
                        title="${entry.name} - ${formatBytes(entry.value)} (${entry.file || 'unknown'}:${entry.line || '?'})"
                        onclick="showDetailModal({
                            title: 'Stack Frame: ${entry.name}',
                            function: '${entry.name}',
                            file: '${entry.file || 'unknown'}',
                            line: '${entry.line || '?'}',
                            memory: ${entry.value},
                            depth: ${entry.depth}
                        })">
                            <div style="padding: 2px 4px; color: white; text-shadow: 1px 1px 1px rgba(0,0,0,0.5); white-space: nowrap; overflow: hidden;">
                                ${entry.name.length > 15 ? entry.name.substring(0, 12) + '...' : entry.name}
                            </div>
                        </div>
                    `;
                }
                currentX += width;
            });
            
            // Render children
            entries.forEach(entry => {
                if (Object.keys(entry.children).length > 0) {
                    html += '<br>' + renderFlameLevel(entry.children, level + 1, totalWidth);
                }
            });
            
            return html;
        }
        
        html += renderFlameLevel(flameData);
        html += '</div>';
        
        container.innerHTML = html;
    }
    "#.to_string()
}

/// Generate variable relationship graph
pub fn generate_variable_relationship_graph() -> String {
    r#"
    function renderVariableRelationshipGraph() {
        const container = document.getElementById('relationshipGraphContainer');
        const allocations = memoryData.allocations || [];
        
        // Build relationship data
        const variables = {};
        const relationships = [];
        
        // Group variables by scope to find relationships
        const scopeGroups = {};
        allocations.forEach(alloc => {
            const scope = alloc.scope_name || 'global';
            if (!scopeGroups[scope]) scopeGroups[scope] = [];
            scopeGroups[scope].push(alloc);
            
            variables[alloc.var_name || 'unknown'] = {
                name: alloc.var_name || 'unknown',
                type: alloc.type_name || 'unknown',
                size: alloc.size,
                scope: scope
            };
        });
        
        // Create relationships between variables in same scope
        Object.entries(scopeGroups).forEach(([scope, vars]) => {
            for (let i = 0; i < vars.length; i++) {
                for (let j = i + 1; j < vars.length; j++) {
                    relationships.push({
                        from: vars[i].var_name || 'unknown',
                        to: vars[j].var_name || 'unknown',
                        type: 'same_scope',
                        scope: scope
                    });
                }
            }
        });
        
        // Render as a simple network diagram
        let html = '<div style="position: relative; width: 100%; height: 400px; background: #f8f9fa; border-radius: 8px; overflow: hidden;">';
        html += '<h3 style="position: absolute; top: 10px; left: 20px; margin: 0; color: #333;">🔗 Variable Relationships</h3>';
        
        // Position variables in a circular layout
        const centerX = 400;
        const centerY = 200;
        const radius = 150;
        const varNames = Object.keys(variables).slice(0, 20); // Limit to 20 for visibility
        
        varNames.forEach((varName, i) => {
            const angle = (i / varNames.length) * 2 * Math.PI;
            const x = centerX + radius * Math.cos(angle);
            const y = centerY + radius * Math.sin(angle);
            const variable = variables[varName];
            
            // Size based on memory usage
            const maxSize = Math.max(...Object.values(variables).map(v => v.size));
            const nodeSize = 10 + (variable.size / maxSize) * 20;
            
            // Color based on type
            const colors = {
                'Vec': '#4CAF50',
                'String': '#2196F3',
                'Box': '#FF9800',
                'HashMap': '#9C27B0',
                'unknown': '#607D8B'
            };
            const color = colors[variable.type] || colors['unknown'];
            
            html += `
                <div style="
                    position: absolute;
                    left: ${x - nodeSize/2}px;
                    top: ${y - nodeSize/2}px;
                    width: ${nodeSize}px;
                    height: ${nodeSize}px;
                    background: ${color};
                    border-radius: 50%;
                    border: 2px solid white;
                    cursor: pointer;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.2);
                    z-index: 10;
                " 
                title="${varName} (${variable.type}) - ${formatBytes(variable.size)}"
                onclick="showDetailModal({
                    title: 'Variable: ${varName}',
                    type: '${variable.type}',
                    size: ${variable.size},
                    scope: '${variable.scope}'
                })"></div>
                <div style="
                    position: absolute;
                    left: ${x - 30}px;
                    top: ${y + nodeSize/2 + 5}px;
                    width: 60px;
                    text-align: center;
                    font-size: 10px;
                    color: #666;
                    pointer-events: none;
                ">${varName.length > 8 ? varName.substring(0, 6) + '..' : varName}</div>
            `;
        });
        
        // Draw relationship lines
        relationships.slice(0, 50).forEach(rel => { // Limit relationships for performance
            const fromIndex = varNames.indexOf(rel.from);
            const toIndex = varNames.indexOf(rel.to);
            
            if (fromIndex !== -1 && toIndex !== -1) {
                const fromAngle = (fromIndex / varNames.length) * 2 * Math.PI;
                const toAngle = (toIndex / varNames.length) * 2 * Math.PI;
                const fromX = centerX + radius * Math.cos(fromAngle);
                const fromY = centerY + radius * Math.sin(fromAngle);
                const toX = centerX + radius * Math.cos(toAngle);
                const toY = centerY + radius * Math.sin(toAngle);
                
                html += `
                    <svg style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 1;">
                        <line x1="${fromX}" y1="${fromY}" x2="${toX}" y2="${toY}" 
                              stroke="rgba(0,0,0,0.1)" stroke-width="1"/>
                    </svg>
                `;
            }
        });
        
        // Add legend
        html += `
            <div style="position: absolute; bottom: 10px; left: 20px; background: rgba(255,255,255,0.9); padding: 10px; border-radius: 6px; font-size: 12px;">
                <div><span style="display: inline-block; width: 12px; height: 12px; background: #4CAF50; border-radius: 50%; margin-right: 5px;"></span>Vec</div>
                <div><span style="display: inline-block; width: 12px; height: 12px; background: #2196F3; border-radius: 50%; margin-right: 5px;"></span>String</div>
                <div><span style="display: inline-block; width: 12px; height: 12px; background: #FF9800; border-radius: 50%; margin-right: 5px;"></span>Box</div>
                <div style="margin-top: 5px; color: #666;">Node size = memory usage</div>
            </div>
        `;
        
        html += '</div>';
        container.innerHTML = html;
    }
    "#.to_string()
}