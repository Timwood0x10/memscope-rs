// MemScope Dashboard JavaScript
// Enhanced memory analysis dashboard with lifetime visualization and FFI tracking

// Tailwind configuration
tailwind.config = {
    theme: {
        extend: {
            colors: {
                primary: '#3B82F6',
                secondary: '#10B981',
                accent: '#8B5CF6',
                neutral: '#1F2937',
                'neutral-light': '#F3F4F6',
                'ffi-red': '#EF4444',
                'safe-yellow': '#F59E0B',
                'safe-green': '#10B981',
            },
            fontFamily: {
                sans: ['Inter', 'system-ui', 'sans-serif'],
            },
        }
    }
}

// Global data store - will be populated by HTML template
window.analysisData = window.analysisData || {};

// Data validation and interaction utilities
window.dataUtils = {
    hasUnsafeFFIData: function () {
        const unsafeData = window.analysisData.basic_usage_snapshot_unsafe_ffi;
        if (!unsafeData) return false;

        const summary = unsafeData.summary || {};
        return (summary.unsafe_count > 0 ||
            summary.ffi_count > 0 ||
            summary.enhanced_entries > 0 ||
            summary.boundary_events > 0 ||
            summary.safety_violations > 0);
    },

    hasFFICallFlowData: function () {
        const unsafeData = window.analysisData.basic_usage_snapshot_unsafe_ffi;
        if (!unsafeData) return false;

        const enhancedData = unsafeData.enhanced_ffi_data || [];
        const boundaryEvents = unsafeData.boundary_events || [];
        return enhancedData.length > 0 || boundaryEvents.length > 0;
    },

    getEmptyStateMessage: function (type) {
        const messages = {
            'unsafe_ffi': 'No unsafe/FFI code detected in your codebase',
            'ffi_flow': 'No FFI call flow patterns found in the analysis'
        };
        return messages[type] || 'No data available';
    },

    createEmptyStateElement: function (message, icon = 'fa-info-circle') {
        return `
            <div class="data-empty-state text-center py-12">
                <div class="mb-4">
                    <i class="fa ${icon} text-6xl text-gray-300"></i>
                </div>
                <h3 class="text-lg font-medium text-gray-600 mb-2">${message}</h3>
                <p class="text-sm text-gray-500">This indicates good memory safety practices in your Rust code.</p>
            </div>
        `;
    },

    // Interactive data filtering and highlighting
    filterDataByRisk: function (riskLevel) {
        const unsafeData = window.analysisData.basic_usage_snapshot_unsafe_ffi;
        if (!unsafeData) return [];

        const filtered = [];

        // Filter enhanced FFI data by risk level
        if (unsafeData.enhanced_ffi_data) {
            unsafeData.enhanced_ffi_data.forEach(item => {
                if (item.source && typeof item.source === 'string') {
                    const riskMatch = item.source.match(/risk_level:\\s*(\\w+)/i);
                    if (riskMatch && riskMatch[1].toLowerCase() === riskLevel.toLowerCase()) {
                        filtered.push(item);
                    }
                }
            });
        }

        return filtered;
    },

    // Get interactive statistics
    getInteractiveStats: function () {
        const unsafeData = window.analysisData.basic_usage_snapshot_unsafe_ffi;
        if (!unsafeData) return null;

        const stats = {
            totalRiskItems: 0,
            riskLevels: { low: 0, medium: 0, high: 0 },
            boundaryEvents: (unsafeData.boundary_events || []).length,
            safetyViolations: (unsafeData.safety_violations || []).length,
            ffiPatterns: (unsafeData.ffi_patterns || []).length
        };

        // Analyze risk levels from enhanced data
        if (unsafeData.enhanced_ffi_data) {
            unsafeData.enhanced_ffi_data.forEach(item => {
                if (item.source && typeof item.source === 'string') {
                    const riskMatch = item.source.match(/risk_level:\\s*(\\w+)/i);
                    if (riskMatch) {
                        const level = riskMatch[1].toLowerCase();
                        if (stats.riskLevels[level] !== undefined) {
                            stats.riskLevels[level]++;
                            stats.totalRiskItems++;
                        }
                    }
                }
            });
        }

        return stats;
    }
};

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
// Data 
processing functions
function getComplexTypesData() {
    if (window.analysisData && window.analysisData.complex_types) {
        console.log('Using embedded complex types data');
        return window.analysisData.complex_types;
    }
    if (window.embeddedJsonData && window.embeddedJsonData.complex_types) {
        console.log('Using embedded complex types data');
        return window.embeddedJsonData.complex_types;
    }
    console.log('Using fallback fake complex types data');
    return complexTypesDataFallback;
}

function getFfiSnapshotData() {
    if (window.analysisData && window.analysisData.unsafe_ffi) {
        console.log('Using embedded FFI data');
        return window.analysisData.unsafe_ffi.enhanced_ffi_data || [];
    }
    if (window.embeddedJsonData && window.embeddedJsonData.unsafe_ffi) {
        console.log('Using embedded FFI data');
        return window.embeddedJsonData.unsafe_ffi.enhanced_ffi_data || [];
    }
    console.log('Using fallback fake FFI data');
    return ffiSnapshotDataFallback;
}

// Fallback data definitions
const complexTypesDataFallback = {
    "categorized_types": {
        "collections": [],
        "generic_types": [
            {
                "complexity_score": 6,
                "normalized_type": "alloc::vec::Vec<T>",
                "ptr": "0x145005bd0",
                "size": 20,
                "type_name": "alloc::vec::Vec<i32>",
                "var_name": "numbers_vec",
                "lifetime_ms": 800
            },
            {
                "complexity_score": 10,
                "normalized_type": "alloc::sync::Arc<T>",
                "ptr": "0x6dd34c96",
                "size": 56,
                "type_name": "alloc::sync::Arc<alloc::string::String>",
                "var_name": "arc_data",
                "lifetime_ms": 600
            },
            {
                "complexity_score": 8,
                "normalized_type": "alloc::boxed::Box<T>",
                "ptr": "0x123e04720",
                "size": 4,
                "type_name": "alloc::boxed::Box<i32>",
                "var_name": "boxed_value",
                "lifetime_ms": 400
            },
            {
                "complexity_score": 14,
                "normalized_type": "alloc::rc::Rc<T>",
                "ptr": "0x5dd34c26",
                "size": 56,
                "type_name": "alloc::rc::Rc<alloc::vec::Vec<i32>>",
                "var_name": "rc_data",
                "lifetime_ms": 700
            },
            {
                "complexity_score": 14,
                "normalized_type": "alloc::rc::Rc<T>",
                "ptr": "0x5dd34d06",
                "size": 56,
                "type_name": "alloc::rc::Rc<alloc::vec::Vec<i32>>",
                "var_name": "rc_data_clone",
                "lifetime_ms": 500
            },
            {
                "complexity_score": 8,
                "normalized_type": "alloc::boxed::Box<T>",
                "ptr": "0x123e05460",
                "size": 4,
                "type_name": "alloc::boxed::Box<i32>",
                "var_name": "boxed_value2",
                "lifetime_ms": 300
            }
        ],
        "smart_pointers": [],
        "trait_objects": []
    },
    "complex_type_analysis": [
        {
            "allocation_count": 2,
            "average_size": 56,
            "category": "Generic",
            "complexity_score": 14,
            "max_size": 56,
            "memory_efficiency": 80,
            "optimization_suggestions": [
                "High complexity type - consider simplifying or using type aliases"
            ],
            "total_size": 112,
            "type_name": "alloc::rc::Rc<T>"
        },
        {
            "allocation_count": 1,
            "average_size": 56,
            "category": "Generic",
            "complexity_score": 10,
            "max_size": 56,
            "memory_efficiency": 80,
            "optimization_suggestions": [],
            "total_size": 56,
            "type_name": "alloc::sync::Arc<T>"
        },
        {
            "allocation_count": 2,
            "average_size": 4,
            "category": "Generic",
            "complexity_score": 8,
            "max_size": 4,
            "memory_efficiency": 90,
            "optimization_suggestions": [],
            "total_size": 8,
            "type_name": "alloc::boxed::Box<T>"
        },
        {
            "allocation_count": 1,
            "average_size": 20,
            "category": "Generic",
            "complexity_score": 6,
            "max_size": 20,
            "memory_efficiency": 60,
            "optimization_suggestions": [],
            "total_size": 20,
            "type_name": "alloc::vec::Vec<T>"
        },
        {
            "allocation_count": 1,
            "average_size": 19,
            "category": "Simple",
            "complexity_score": 1,
            "max_size": 19,
            "memory_efficiency": 85,
            "optimization_suggestions": [],
            "total_size": 19,
            "type_name": "String"
        }
    ],
    "metadata": {
        "analysis_type": "complex_types_analysis_optimized",
        "export_version": "2.0",
        "optimization_level": "high",
        "processing_mode": "sequential",
        "timestamp": 1753328255,
        "total_allocations_analyzed": 639,
        "unique_complex_types": 5
    },
    "optimization_recommendations": [
        "Use 'cargo clippy' to identify additional optimization opportunities",
        "Consider profiling with 'perf' or 'valgrind' for detailed performance analysis",
        "Optimize Vec capacity to improve memory efficiency",
        "Use appropriate smart pointers based on concurrency requirements"
    ],
    "summary": {
        "collection_count": 0,
        "complexity_distribution": {
            "high_complexity": 3,
            "low_complexity": 1,
            "medium_complexity": 1,
            "very_high_complexity": 0
        },
        "generic_type_count": 6,
        "smart_pointer_count": 0,
        "total_complex_types": 5,
        "trait_object_count": 0
    }
};

// Fallback fake FFI data
const ffiSnapshotDataFallback = [
    {
        "base": {
            "ptr": 5408595456,
            "size": 256,
            "var_name": null,
            "type_name": null,
            "scope_name": null,
            "timestamp_alloc": 1753414296355925000,
            "timestamp_dealloc": null,
            "borrow_count": 0,
            "stack_trace": null,
            "is_leaked": false,
            "lifetime_ms": null,
            "smart_pointer_info": null,
            "memory_layout": null,
            "generic_info": null,
            "dynamic_type_info": null,
            "runtime_state": null,
            "stack_allocation": null,
            "temporary_object": null,
            "fragmentation_analysis": null,
            "generic_instantiation": null,
            "type_relationships": null,
            "type_usage": null,
            "function_call_tracking": null,
            "lifecycle_tracking": null,
            "access_tracking": null
        },
        "source": {
            "FfiC": {
                "library_name": "libc",
                "function_name": "malloc",
                "call_stack": [
                    {
                        "function_name": "current_function",
                        "file_name": "src/unsafe_ffi_tracker.rs",
                        "line_number": 42,
                        "is_unsafe": true
                    }
                ],
                "libc_hook_info": {
                    "hook_method": "DynamicLinker",
                    "original_function": "malloc",
                    "hook_timestamp": 1753414296355937000,
                    "allocation_metadata": {
                        "requested_size": 256,
                        "actual_size": 256,
                        "alignment": 8,
                        "allocator_info": "libc malloc",
                        "protection_flags": {
                            "readable": true,
                            "writable": true,
                            "executable": false,
                            "shared": false
                        }
                    },
                    "hook_overhead_ns": 100
                }
            }
        },
        "call_stack": [
            {
                "function_name": "current_function",
                "file_name": "src/unsafe_ffi_tracker.rs",
                "line_number": 42,
                "is_unsafe": true
            }
        ],
        "cross_boundary_events": [
            {
                "event_type": "FfiToRust",
                "timestamp": 1753414296356,
                "from_context": "libc",
                "to_context": "rust_main",
                "stack": [
                    {
                        "function_name": "current_function",
                        "file_name": "src/unsafe_ffi_tracker.rs",
                        "line_number": 42,
                        "is_unsafe": true
                    }
                ]
            }
        ],
        "safety_violations": [],
        "ffi_tracked": true,
        "memory_passport": null,
        "ownership_history": null
    },
    {
        "base": {
            "ptr": 5408583952,
            "size": 40,
            "var_name": null,
            "type_name": null,
            "scope_name": null,
            "timestamp_alloc": 1753414296355789000,
            "timestamp_dealloc": null,
            "borrow_count": 0,
            "stack_trace": null,
            "is_leaked": false,
            "lifetime_ms": null,
            "smart_pointer_info": null,
            "memory_layout": null,
            "generic_info": null,
            "dynamic_type_info": null,
            "runtime_state": null,
            "stack_allocation": null,
            "temporary_object": null,
            "fragmentation_analysis": null,
            "generic_instantiation": null,
            "type_relationships": null,
            "type_usage": null,
            "function_call_tracking": null,
            "lifecycle_tracking": null,
            "access_tracking": null
        },
        "source": {
            "UnsafeRust": {
                "unsafe_block_location": "examples/unsafe_ffi_demo.rs:37:13",
                "call_stack": [
                    {
                        "function_name": "current_function",
                        "file_name": "src/unsafe_ffi_tracker.rs",
                        "line_number": 42,
                        "is_unsafe": true
                    }
                ],
                "risk_assessment": {
                    "risk_level": "Medium",
                    "risk_factors": [
                        {
                            "factor_type": "ManualMemoryManagement",
                            "severity": 5.0,
                            "description": "Manual memory management in unsafe block",
                            "source_location": "examples/unsafe_ffi_demo.rs:37:13"
                        }
                    ],
                    "mitigation_suggestions": [
                        "Ensure proper memory cleanup",
                        "Use RAII patterns where possible"
                    ],
                    "confidence_score": 0.7,
                    "assessment_timestamp": 1753414296355826000
                }
            }
        },
        "call_stack": [
            {
                "function_name": "current_function",
                "file_name": "src/unsafe_ffi_tracker.rs",
                "line_number": 42,
                "is_unsafe": true
            }
        ],
        "cross_boundary_events": [
            {
                "event_type": "RustToFfi",
                "timestamp": 1753414296355,
                "from_context": "unsafe_rust_block",
                "to_context": "potential_ffi_target",
                "stack": [
                    {
                        "function_name": "current_function",
                        "file_name": "src/unsafe_ffi_tracker.rs",
                        "line_number": 42,
                        "is_unsafe": true
                    }
                ]
            }
        ],
        "safety_violations": [],
        "ffi_tracked": false,
        "memory_passport": null,
        "ownership_history": null
    }
];

// Utility functions
function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function updateStats(stats) {
    // Update statistics display
    const statsContainer = document.querySelector('.stats-grid');
    if (statsContainer && stats) {
        statsContainer.innerHTML = `
            <div class="stat-card">
                <span class="stat-value">${stats.active_allocations || 0}</span>
                <span class="stat-label">Active Allocations</span>
            </div>
            <div class="stat-card">
                <span class="stat-value">${formatBytes(stats.active_memory || 0)}</span>
                <span class="stat-label">Active Memory</span>
            </div>
            <div class="stat-card">
                <span class="stat-value">${formatBytes(stats.peak_memory || 0)}</span>
                <span class="stat-label">Peak Memory</span>
            </div>
            <div class="stat-card">
                <span class="stat-value">${stats.total_allocations || 0}</span>
                <span class="stat-label">Total Allocations</span>
            </div>
        `;
    }
}

// Dashboard initialization and data population
function initializeDashboard() {
    console.log('Initializing MemScope dashboard...');
    
    // Use embedded data if available, otherwise use fallback
    const complexTypesData = getComplexTypesData();
    const ffiSnapshotData = getFfiSnapshotData();

    // Populate summary data
    const totalComplexTypesEl = document.getElementById('total-complex-types');
    const totalAllocationsEl = document.getElementById('total-allocations');
    const genericTypeCountEl = document.getElementById('generic-type-count');
    const unsafeFfiCountEl = document.getElementById('unsafe-ffi-count');

    if (totalComplexTypesEl) totalComplexTypesEl.textContent = complexTypesData.summary.total_complex_types;
    if (totalAllocationsEl) totalAllocationsEl.textContent = complexTypesData.metadata.total_allocations_analyzed;
    if (genericTypeCountEl) genericTypeCountEl.textContent = complexTypesData.summary.generic_type_count;
    if (unsafeFfiCountEl) unsafeFfiCountEl.textContent = ffiSnapshotData.length;

    // Populate generic types table with lifetime information
    populateGenericTypesTable(complexTypesData);
    
    // Populate optimization recommendations
    populateOptimizationRecommendations(complexTypesData);
    
    // Create charts
    createComplexityChart(complexTypesData);
    createMemoryDistributionChart(complexTypesData);
    
    // Render FFI data
    renderFfiData(ffiSnapshotData, document.getElementById('ffi-data-render'));
    
    // Initialize modern variable graph
    initModernVariableGraph(complexTypesData);
    
    // Initialize lifetime visualization
    initLifetimeVisualization();
}

function populateGenericTypesTable(complexTypesData) {
    const tableBody = document.getElementById('generic-types-table-body');
    if (!tableBody) return;
    
    complexTypesData.categorized_types.generic_types.forEach(type => {
        let typeClass = '';
        if (type.type_name.includes('Vec')) typeClass = 'bg-blue-50';
        else if (type.type_name.includes('Arc')) typeClass = 'bg-green-50';
        else if (type.type_name.includes('Box')) typeClass = 'bg-purple-50';
        else if (type.type_name.includes('Rc')) typeClass = 'bg-yellow-50';

        const row = document.createElement('tr');
        row.className = `hover:bg-gray-50 transition-colors ${typeClass}`;
        row.innerHTML = `
            <td class="px-6 py-4 whitespace-nowrap">
                <div class="font-medium text-neutral">${type.var_name}</div>
            </td>
            <td class="px-6 py-4">
                <div class="text-sm text-gray-900 break-all max-w-xs">${type.type_name}</div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">${type.ptr}</td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">${type.size}</td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">${type.lifetime_ms}ms
                <div class="lifespan-indicator bg-gray-300 mt-1">
                    <div class="h-full" style="width: ${Math.min(100, type.lifetime_ms)}%; background-color: ${type.type_name.includes('Vec') ? '#3B82F6' :
                type.type_name.includes('Arc') ? '#10B981' :
                    type.type_name.includes('Box') ? '#8B5CF6' :
                        '#F59E0B'
            }"></div>
                </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
                <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full 
                    ${type.complexity_score > 10 ? 'bg-red-100 text-red-800' :
                type.complexity_score > 5 ? 'bg-yellow-100 text-yellow-800' :
                    'bg-green-100 text-green-800'}">
                    ${type.complexity_score}
                </span>
            </td>
        `;
        tableBody.appendChild(row);
    });
}

function populateOptimizationRecommendations(complexTypesData) {
    const memoryRecList = document.getElementById('memory-optimization-recommendations');
    if (!memoryRecList) return;
    
    complexTypesData.optimization_recommendations.forEach(rec => {
        const li = document.createElement('li');
        li.className = 'text-gray-700';
        li.textContent = rec;
        memoryRecList.appendChild(li);
    });
}

function createComplexityChart(complexTypesData) {
    const complexityCtx = document.getElementById('complexity-chart');
    if (!complexityCtx) return;
    
    new Chart(complexityCtx.getContext('2d'), {
        type: 'bar',
        data: {
            labels: ['Low', 'Medium', 'High', 'Very High'],
            datasets: [{
                label: 'Number of Types',
                data: [
                    complexTypesData.summary.complexity_distribution.low_complexity,
                    complexTypesData.summary.complexity_distribution.medium_complexity,
                    complexTypesData.summary.complexity_distribution.high_complexity,
                    complexTypesData.summary.complexity_distribution.very_high_complexity
                ],
                backgroundColor: [
                    'rgba(16, 185, 129, 0.7)',
                    'rgba(59, 130, 246, 0.7)',
                    'rgba(245, 158, 11, 0.7)',
                    'rgba(239, 68, 68, 0.7)'
                ],
                borderColor: [
                    'rgb(16, 185, 129)',
                    'rgb(59, 130, 246)',
                    'rgb(245, 158, 11)',
                    'rgb(239, 68, 68)'
                ],
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        precision: 0
                    }
                }
            }
        }
    });
}

function createMemoryDistributionChart(complexTypesData) {
    const memoryCtx = document.getElementById('memory-distribution-chart');
    if (!memoryCtx) return;
    
    new Chart(memoryCtx.getContext('2d'), {
        type: 'pie',
        data: {
            labels: complexTypesData.complex_type_analysis.map(a => a.type_name),
            datasets: [{
                data: complexTypesData.complex_type_analysis.map(a => a.total_size),
                backgroundColor: [
                    'rgba(245, 158, 11, 0.7)',  // Rc - yellow
                    'rgba(16, 185, 129, 0.7)',  // Arc - green
                    'rgba(139, 92, 246, 0.7)',  // Box - purple
                    'rgba(59, 130, 246, 0.7)',  // Vec - blue
                    'rgba(236, 72, 153, 0.7)'   // String - pink
                ],
                borderColor: [
                    'rgb(245, 158, 11)',
                    'rgb(16, 185, 129)',
                    'rgb(139, 92, 246)',
                    'rgb(59, 130, 246)',
                    'rgb(236, 72, 153)'
                ],
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            plugins: {
                tooltip: {
                    callbacks: {
                        label: function (context) {
                            const value = context.raw;
                            const total = context.dataset.data.reduce((a, b) => a + b, 0);
                            const percentage = Math.round((value / total) * 100);
                            return `${context.label}: ${value} bytes (${percentage}%)`;
                        }
                    }
                }
            }
        }
    });
}//
 FFI Data Rendering Functions
function renderFfiData(data, container) {
    if (!container) return;
    
    data.forEach((item, index) => {
        const entryDiv = document.createElement('div');
        entryDiv.className = 'bg-white rounded-lg card-shadow hover-lift border-l-4 ' +
            (item.source.FfiC ? 'border-blue-500' : 'border-yellow-500');

        const sourceType = item.source.FfiC ? 'FFI (C)' : 'Unsafe Rust';
        const sourceColor = item.source.FfiC ? 'text-blue-500' : 'text-yellow-500';

        // Create header with toggle button
        const headerDiv = document.createElement('div');
        headerDiv.className = 'p-5 cursor-pointer flex justify-between items-center';
        headerDiv.innerHTML = `
            <div class="font-semibold text-lg ${sourceColor}">Entry ${index + 1}: ${sourceType}</div>
            <i class="fa fa-chevron-down rotate-icon"></i>
        `;
        entryDiv.appendChild(headerDiv);

        // Create collapsible content
        const contentDiv = document.createElement('div');
        contentDiv.className = 'collapsible-content px-5 pb-5 border-t border-gray-100';
        entryDiv.appendChild(contentDiv);

        // Render the data inside collapsible content
        renderFfiObject(item, contentDiv);
        container.appendChild(entryDiv);

        // Add click handler for toggle
        headerDiv.addEventListener('click', () => {
            contentDiv.classList.toggle('active');
            headerDiv.querySelector('.rotate-icon').classList.toggle('active');
        });
    });
}

function renderFfiObject(obj, container, level = 0) {
    // Limit nesting depth to prevent excessive expansion
    if (level > 3) {
        const moreDiv = document.createElement('div');
        moreDiv.className = 'text-gray-500 text-sm italic mt-1';
        moreDiv.textContent = '(Content truncated for readability)';
        container.appendChild(moreDiv);
        return;
    }

    for (const [key, value] of Object.entries(obj)) {
        // Skip null values and empty objects
        if (value === null || (typeof value === 'object' && Object.keys(value).length === 0)) {
            continue;
        }

        const sectionDiv = document.createElement('div');
        sectionDiv.className = 'mb-4';

        const keyDiv = document.createElement('div');
        keyDiv.className = 'font-medium text-neutral mb-2';
        keyDiv.textContent = key.charAt(0).toUpperCase() + key.slice(1);
        sectionDiv.appendChild(keyDiv);

        const valueDiv = document.createElement('div');
        valueDiv.className = 'pl-4 border-l-2 border-gray-200';

        if (typeof value === 'object') {
            if (Array.isArray(value)) {
                // For arrays, show count and allow expansion
                const arrayHeader = document.createElement('div');
                arrayHeader.className = 'font-medium text-sm text-gray-600 cursor-pointer flex items-center';
                arrayHeader.innerHTML = `
                    Array (${value.length} items)
                    <i class="fa fa-chevron-down rotate-icon ml-2 text-xs"></i>
                `;
                valueDiv.appendChild(arrayHeader);

                const arrayContent = document.createElement('div');
                arrayContent.className = 'collapsible-content mt-2';
                valueDiv.appendChild(arrayContent);

                value.forEach((item, idx) => {
                    const arrayItemDiv = document.createElement('div');
                    arrayItemDiv.className = 'mb-3';
                    arrayItemDiv.innerHTML = `<div class="font-medium text-sm text-gray-600">Item ${idx + 1}:</div>`;
                    renderFfiObject(item, arrayItemDiv, level + 1);
                    arrayContent.appendChild(arrayItemDiv);
                });

                // Add click handler for array toggle
                arrayHeader.addEventListener('click', () => {
                    arrayContent.classList.toggle('active');
                    arrayHeader.querySelector('.rotate-icon').classList.toggle('active');
                });
            } else {
                // For objects, create a toggleable section
                const objectHeader = document.createElement('div');
                objectHeader.className = 'font-medium text-sm text-gray-600 cursor-pointer flex items-center';
                objectHeader.innerHTML = `
                    Object
                    <i class="fa fa-chevron-down rotate-icon ml-2 text-xs"></i>
                `;
                valueDiv.appendChild(objectHeader);

                const objectContent = document.createElement('div');
                objectContent.className = 'collapsible-content mt-2';
                valueDiv.appendChild(objectContent);

                renderFfiObject(value, objectContent, level + 1);

                // Add click handler for object toggle
                objectHeader.addEventListener('click', () => {
                    objectContent.classList.toggle('active');
                    objectHeader.querySelector('.rotate-icon').classList.toggle('active');
                });
            }
        } else {
            // For primitive values, just display them
            valueDiv.textContent = value;
        }

        sectionDiv.appendChild(valueDiv);
        container.appendChild(sectionDiv);
    }
}

// Modern Variable Relationship Graph using D3.js
function initModernVariableGraph(complexTypesData) {
    const container = d3.select("#variable-graph-container");
    if (container.empty()) return;
    
    const width = container.node().getBoundingClientRect().width;
    const height = container.node().getBoundingClientRect().height;

    // Clear any existing content
    container.selectAll("*").remove();

    // Create SVG
    const svg = container.append("svg")
        .attr("width", width)
        .attr("height", height)
        .style("background", "linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%)");

    // Create zoom behavior
    const zoom = d3.zoom()
        .scaleExtent([0.5, 3])
        .on("zoom", (event) => {
            g.attr("transform", event.transform);
        });

    svg.call(zoom);

    const g = svg.append("g");

    // Get data from complexTypesData
    const nodes = [];
    const links = [];

    if (complexTypesData && complexTypesData.categorized_types && complexTypesData.categorized_types.generic_types) {
        complexTypesData.categorized_types.generic_types.forEach((type) => {
            const category = getNodeCategory(type.type_name);
            nodes.push({
                id: type.var_name,
                name: type.var_name,
                type: type.type_name,
                size: type.size || 20,
                complexity: type.complexity_score || 1,
                lifetime: type.lifetime_ms || 0,
                ptr: type.ptr,
                category: category
            });
        });

        // Create links based on type relationships
        for (let i = 0; i < nodes.length; i++) {
            for (let j = i + 1; j < nodes.length; j++) {
                const node1 = nodes[i];
                const node2 = nodes[j];
                
                // Create links for similar types or clones
                if (node1.type === node2.type) {
                    links.push({
                        source: node1.id,
                        target: node2.id,
                        type: 'clone',
                        strength: 0.8
                    });
                } else if (getNodeCategory(node1.type) === getNodeCategory(node2.type)) {
                    links.push({
                        source: node1.id,
                        target: node2.id,
                        type: 'similar',
                        strength: 0.3
                    });
                }
            }
        }
    }

    // Create force simulation
    const simulation = d3.forceSimulation(nodes)
        .force("link", d3.forceLink(links).id(d => d.id).strength(d => d.strength))
        .force("charge", d3.forceManyBody().strength(-300))
        .force("center", d3.forceCenter(width / 2, height / 2))
        .force("collision", d3.forceCollide().radius(d => Math.sqrt(d.complexity) * 8 + 10));

    // Create links
    const link = g.append("g")
        .selectAll("line")
        .data(links)
        .enter().append("line")
        .attr("stroke", d => d.type === 'clone' ? '#666' : '#999')
        .attr("stroke-opacity", 0.6)
        .attr("stroke-width", d => d.type === 'clone' ? 2 : 1)
        .attr("stroke-dasharray", d => d.type === 'similar' ? "5,5" : null);

    // Create nodes
    const node = g.append("g")
        .selectAll("g")
        .data(nodes)
        .enter().append("g")
        .attr("class", "node")
        .call(d3.drag()
            .on("start", dragstarted)
            .on("drag", dragged)
            .on("end", dragended));

    // Add circles to nodes
    node.append("circle")
        .attr("r", d => Math.sqrt(d.complexity) * 4 + 8)
        .attr("fill", d => getNodeColor(d.category))
        .attr("stroke", "#fff")
        .attr("stroke-width", 2);

    // Add labels to nodes
    node.append("text")
        .attr("dy", ".35em")
        .attr("text-anchor", "middle")
        .style("font-size", "12px")
        .style("font-weight", "bold")
        .style("fill", "#333")
        .text(d => d.name);

    // Add tooltips
    node.append("title")
        .text(d => `${d.name}\nType: ${d.type}\nSize: ${d.size} bytes\nComplexity: ${d.complexity}\nLifetime: ${d.lifetime}ms`);

    // Update positions on simulation tick
    simulation.on("tick", () => {
        link
            .attr("x1", d => d.source.x)
            .attr("y1", d => d.source.y)
            .attr("x2", d => d.target.x)
            .attr("y2", d => d.target.y);

        node
            .attr("transform", d => `translate(${d.x},${d.y})`);
    });

    // Drag functions
    function dragstarted(event, d) {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
    }

    function dragged(event, d) {
        d.fx = event.x;
        d.fy = event.y;
    }

    function dragended(event, d) {
        if (!event.active) simulation.alphaTarget(0);
        d.fx = null;
        d.fy = null;
    }

    // Add reset zoom button functionality
    const resetZoomBtn = document.getElementById('reset-zoom');
    if (resetZoomBtn) {
        resetZoomBtn.addEventListener('click', () => {
            svg.transition().duration(750).call(
                zoom.transform,
                d3.zoomIdentity
            );
        });
    }

    // Add auto layout button functionality
    const autoLayoutBtn = document.getElementById('auto-layout');
    if (autoLayoutBtn) {
        autoLayoutBtn.addEventListener('click', () => {
            simulation.alpha(1).restart();
        });
    }
}

function getNodeCategory(typeName) {
    if (typeName.includes('Vec') || typeName.includes('HashMap') || typeName.includes('BTreeMap')) {
        return 'collections';
    } else if (typeName.includes('Box') || typeName.includes('Rc') || typeName.includes('Arc')) {
        return 'smart_pointers';
    } else {
        return 'primitives';
    }
}

function getNodeColor(category) {
    const colors = {
        'smart_pointers': '#3B82F6',  // Blue
        'collections': '#10B981',     // Green
        'primitives': '#F59E0B',      // Orange
        'default': '#6B7280'          // Gray
    };
    return colors[category] || colors.default;
}

// Initialize dashboard when DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
    console.log('MemScope dashboard loaded');
    initializeDashboard();
});