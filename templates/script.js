// MemScope Dashboard JavaScript - Simplified version
// This file contains only the essential functions for the existing dashboard

// Global data store - will be populated by HTML template
window.analysisData = window.analysisData || {};

// Initialize lifetime visualization from JSON data
function initLifetimeVisualization() {
    console.log('🔄 Initializing lifetime visualization...');
    
    // Get lifetime data from the global data store
    const lifetimeData = window.analysisData.lifetime;
    if (!lifetimeData || !lifetimeData.lifecycle_events) {
        console.warn('⚠️ No lifetime data found');
        showEmptyLifetimeState();
        return;
    }
    
    // Filter for user-defined variables (non-unknown var_name and type_name)
    const userVariables = lifetimeData.lifecycle_events.filter(event => 
        event.var_name && event.var_name !== 'unknown' && 
        event.type_name && event.type_name !== 'unknown'
    );
    
    console.log(`📊 Found ${userVariables.length} user-defined variables in lifetime data`);
    
    if (userVariables.length === 0) {
        showEmptyLifetimeState();
        return;
    }
    
    // Group by variable name to get allocation/deallocation pairs
    const variableGroups = groupVariablesByName(userVariables);
    
    // Render the lifetime visualization
    renderLifetimeVisualization(variableGroups);
}

// Group variables by name to track their lifecycle
function groupVariablesByName(events) {
    const groups = {};
    
    events.forEach(event => {
        const varName = event.var_name;
        if (!groups[varName]) {
            groups[varName] = {
                var_name: varName,
                type_name: event.type_name,
                events: []
            };
        }
        groups[varName].events.push(event);
    });
    
    return Object.values(groups);
}

// Render the lifetime visualization
function renderLifetimeVisualization(variableGroups) {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;
    
    // Clear loading state
    container.innerHTML = '';
    
    // Get color scheme for different types
    const typeColors = {
        'Vec': { bg: 'bg-blue-500', border: 'border-blue-500' },
        'Box': { bg: 'bg-purple-500', border: 'border-purple-500' },
        'Rc': { bg: 'bg-yellow-500', border: 'border-yellow-500' },
        'Arc': { bg: 'bg-green-500', border: 'border-green-500' },
        'String': { bg: 'bg-pink-500', border: 'border-pink-500' },
        'default': { bg: 'bg-gray-500', border: 'border-gray-500' }
    };
    
    // Calculate timeline bounds
    const allTimestamps = variableGroups.flatMap(group => 
        group.events.map(e => e.timestamp)
    );
    const minTime = Math.min(...allTimestamps);
    const maxTime = Math.max(...allTimestamps);
    const timeRange = maxTime - minTime;
    
    console.log(`📊 Timeline: ${minTime} to ${maxTime} (range: ${timeRange})`);
    
    // Render each variable
    variableGroups.forEach((group) => {
        const varDiv = document.createElement('div');
        varDiv.className = 'flex items-end py-3 border-b border-gray-100';
        
        // Determine color based on type
        const typeKey = Object.keys(typeColors).find(key => 
            group.type_name.includes(key)
        ) || 'default';
        const colors = typeColors[typeKey];
        
        // Calculate position and width based on timestamps
        const firstEvent = group.events[0];
        const startTime = firstEvent.timestamp;
        const startPercent = timeRange > 0 ? ((startTime - minTime) / timeRange) * 100 : 0;
        
        // For now, assume a fixed width since we don't have deallocation events
        // In a real implementation, you'd track deallocation events too
        const widthPercent = 60; // Default width
        
        // Format type name for display
        const displayTypeName = formatTypeName(group.type_name);
        
        varDiv.innerHTML = `
            <div class="w-40 flex-shrink-0 text-sm font-medium">
                ${group.var_name} (${displayTypeName})
            </div>
            <div class="flex-grow relative">
                <div class="lifespan-indicator ${colors.bg}" 
                     style="width: ${widthPercent}%; margin-left: ${startPercent}%;" 
                     title="Variable: ${group.var_name}, Type: ${displayTypeName}">
                    <div class="absolute -top-6 left-0 text-xs ${colors.bg} text-white px-2 py-1 rounded whitespace-nowrap">
                        Allocated: ${formatTimestamp(startTime, minTime)}
                    </div>
                </div>
            </div>
        `;
        
        container.appendChild(varDiv);
    });
    
    console.log(`✅ Rendered ${variableGroups.length} variables in lifetime visualization`);
}

// Format type name for better display
function formatTypeName(typeName) {
    // Simplify complex type names
    return typeName
        .replace(/alloc::/g, '')
        .replace(/std::/g, '')
        .replace(/::Vec/g, 'Vec')
        .replace(/::Box/g, 'Box')
        .replace(/::Rc/g, 'Rc')
        .replace(/::Arc/g, 'Arc')
        .replace(/::String/g, 'String');
}

// Format timestamp relative to start time
function formatTimestamp(timestamp, minTime) {
    const relativeMs = Math.round((timestamp - minTime) / 1000000); // Convert nanoseconds to milliseconds
    return `${relativeMs}ms`;
}

// Show empty state when no user variables found
function showEmptyLifetimeState() {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;
    
    container.innerHTML = `
        <div class="text-center py-8 text-gray-500">
            <i class="fa fa-info-circle text-2xl mb-2"></i>
            <p>No user-defined variables found in lifetime data</p>
            <p class="text-sm mt-1">Use track_var! macro to track variable lifetimes</p>
        </div>
    `;
}

// Initialize dashboard when DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
    console.log('MemScope dashboard loaded');
    initLifetimeVisualization();
});