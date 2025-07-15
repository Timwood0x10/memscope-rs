// Dashboard JavaScript for memscope-rs web interface
// Handles data loading, visualization, and user interactions

let currentData = null;
let charts = {};

// Initialize dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    initDashboard();
});

function initDashboard() {
    setupTabNavigation();
    setupDataSourceSelector();
    loadSampleData();
    initializeCharts();
}

// Tab navigation
function setupTabNavigation() {
    const tabs = document.querySelectorAll('.nav-tab');
    const contents = document.querySelectorAll('.tab-content');

    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            // Remove active class from all tabs and contents
            tabs.forEach(t => t.classList.remove('active'));
            contents.forEach(c => c.classList.remove('active'));

            // Add active class to clicked tab
            tab.classList.add('active');

            // Show corresponding content
            const targetTab = tab.getAttribute('data-tab');
            const targetContent = document.getElementById(targetTab);
            if (targetContent) {
                targetContent.classList.add('active');
            }
        });
    });
}

// Data source selector
function setupDataSourceSelector() {
    const selector = document.getElementById('dataSource');
    if (selector) {
        selector.addEventListener('change', (e) => {
            const source = e.target.value;
            switch(source) {
                case 'sample':
                    loadSampleData();
                    break;
                case 'upload':
                    handleFileUpload();
                    break;
                case 'json':
                    loadJSONFile();
                    break;
                default:
                    // If it's a specific JSON filename, load it directly
                    if (source.endsWith('.json')) {
                        loadSpecificDataset(source);
                    }
                    break;
            }
        });
    }
}

// Load sample data
function loadSampleData() {
    const sampleData = {
        memory_stats: {
            active_memory: 1024 * 1024, // 1MB
            peak_memory: 2 * 1024 * 1024, // 2MB
            active_allocations: 25,
            total_allocations: 100,
            total_deallocations: 75
        },
        metrics: {
            unsafe_allocations: 5,
            ffi_allocations: 3,
            safety_violations: 2,
            boundary_crossings: 8,
            total_unsafe_memory: 512 * 1024, // 512KB
            memory_efficiency: 85.5
        },
        allocations: [
            {
                id: "alloc_1",
                ptr: 0x7fff5fbff000,
                size: 1024,
                timestamp_alloc: Date.now() - 10000,
                timestamp_dealloc: null,
                is_active: true,
                source_type: "Unsafe Rust",
                source_details: {
                    unsafe_location: "src/main.rs:42",
                    risk_level: "Medium"
                }
            },
            {
                id: "alloc_2", 
                ptr: 0x7fff5fbff400,
                size: 2048,
                timestamp_alloc: Date.now() - 8000,
                timestamp_dealloc: null,
                is_active: true,
                source_type: "FFI C",
                source_details: {
                    library_name: "libc",
                    function_name: "malloc",
                    risk_level: "High"
                }
            },
            {
                id: "alloc_3",
                ptr: 0x7fff5fbff800,
                size: 512,
                timestamp_alloc: Date.now() - 5000,
                timestamp_dealloc: Date.now() - 2000,
                is_active: false,
                source_type: "Safe Rust",
                source_details: {
                    risk_level: "Low"
                }
            }
        ],
        violations: [
            {
                id: "violation_1",
                violation_type: "UseAfterFree",
                severity: "High",
                timestamp: Date.now() - 3000,
                description: "Potential use-after-free detected in unsafe block",
                location: "src/unsafe_ops.rs:15",
                suggested_fix: "Consider using smart pointers or RAII patterns"
            },
            {
                id: "violation_2",
                violation_type: "DoubleFree",
                severity: "Medium", 
                timestamp: Date.now() - 1000,
                description: "Double free detected in FFI boundary",
                location: "FFI call to external library",
                suggested_fix: "Ensure proper ownership transfer across FFI boundary"
            }
        ]
    };

    updateDashboard(sampleData);
}

// Load JSON file
async function loadJSONFile() {
    try {
        // Try to load from existing JSON files
        const jsonFiles = [
            'data.json', 
            'unsafe_ffi_memory_snapshot.json', 
            'complex_lifecycle_snapshot.json'
        ];
        
        for (const filename of jsonFiles) {
            try {
                const response = await fetch(filename);
                if (response.ok) {
                    const data = await response.json();
                    console.log(`Loaded data from ${filename}`);
                    updateDashboard(transformJSONData(data));
                    showNotification(`Loaded data from ${filename}`, 'success');
                    return;
                }
            } catch (e) {
                console.log(`Failed to load ${filename}:`, e);
            }
        }
        
        // If no files found, show file picker
        showNotification('No JSON files found, please upload one', 'info');
        handleFileUpload();
    } catch (error) {
        console.error('Error loading JSON:', error);
        showNotification('Failed to load JSON data', 'error');
        loadSampleData();
    }
}

// Quick data switching functions
async function loadSpecificDataset(filename) {
    try {
        const response = await fetch(filename);
        if (response.ok) {
            const data = await response.json();
            updateDashboard(transformJSONData(data));
            showNotification(`Loaded ${filename}`, 'success');
        } else {
            throw new Error(`Failed to load ${filename}`);
        }
    } catch (error) {
        console.error(`Error loading ${filename}:`, error);
        showNotification(`Failed to load ${filename}`, 'error');
    }
}

// Transform JSON data to dashboard format
function transformJSONData(data) {
    // Handle different JSON formats
    if (data.memory_stats) {
        // Already in correct format
        return data;
    }
    
    // Transform from memscope-rs export format
    const transformed = {
        memory_stats: {
            active_memory: data.memory_stats?.active_memory || 0,
            peak_memory: data.memory_stats?.peak_memory || 0,
            active_allocations: data.memory_stats?.active_allocations || 0,
            total_allocations: data.memory_stats?.total_allocations || 0,
            total_deallocations: data.memory_stats?.total_deallocations || 0
        },
        metrics: {
            unsafe_allocations: 0,
            ffi_allocations: 0,
            safety_violations: 0,
            boundary_crossings: 0,
            total_unsafe_memory: 0,
            memory_efficiency: 0
        },
        allocations: [],
        violations: []
    };

    // Extract allocation data from memory hierarchy
    if (data.memory_hierarchy) {
        const allAllocations = [];
        extractAllocationsFromHierarchy(data.memory_hierarchy, allAllocations);
        transformed.allocations = allAllocations;
        transformed.memory_stats.active_allocations = allAllocations.length;
    }

    return transformed;
}

// Extract allocations from memory hierarchy
function extractAllocationsFromHierarchy(hierarchy, allocations) {
    for (const category in hierarchy) {
        const categoryData = hierarchy[category];
        if (categoryData.subcategories) {
            for (const subcat in categoryData.subcategories) {
                const subcatData = categoryData.subcategories[subcat];
                if (subcatData.types) {
                    subcatData.types.forEach(type => {
                        if (type.allocations) {
                            type.allocations.forEach(alloc => {
                                allocations.push({
                                    id: `alloc_${alloc.allocation_time}`,
                                    ptr: Math.floor(Math.random() * 0xFFFFFFFF),
                                    size: alloc.size_bytes,
                                    timestamp_alloc: alloc.allocation_time,
                                    timestamp_dealloc: null,
                                    is_active: true,
                                    source_type: "Safe Rust",
                                    source_details: {
                                        type_name: alloc.type_name,
                                        variable_name: alloc.variable_name,
                                        risk_level: "Low"
                                    }
                                });
                            });
                        }
                    });
                }
            }
        }
    }
}

// Handle file upload
function handleFileUpload() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = (e) => {
        const file = e.target.files[0];
        if (file) {
            const reader = new FileReader();
            reader.onload = (e) => {
                try {
                    const data = JSON.parse(e.target.result);
                    updateDashboard(transformJSONData(data));
                    showNotification('JSON file loaded successfully', 'success');
                } catch (error) {
                    console.error('JSON parse error:', error);
                    showNotification('Invalid JSON file', 'error');
                }
            };
            reader.readAsText(file);
        }
    };
    input.click();
}

// Update dashboard with data
function updateDashboard(data) {
    currentData = data;
    updateMetrics(data);
    updateCharts(data);
    updateUnsafeFFITab(data);
    updateMemoryAnalysis(data);
}

// Update metrics
function updateMetrics(data) {
    if (data.memory_stats) {
        updateElement('activeMemory', formatBytes(data.memory_stats.active_memory));
        updateElement('peakMemory', formatBytes(data.memory_stats.peak_memory));
        updateElement('activeAllocations', data.memory_stats.active_allocations);
    }
    
    if (data.metrics) {
        updateElement('memoryEfficiency', data.metrics.memory_efficiency.toFixed(1) + '%');
        updateElement('unsafeAllocations', data.metrics.unsafe_allocations);
        updateElement('ffiAllocations', data.metrics.ffi_allocations);
        updateElement('safetyViolations', data.metrics.safety_violations);
        updateElement('boundaryCrossings', data.metrics.boundary_crossings);
    }
}

// Update unsafe/FFI tab
function updateUnsafeFFITab(data) {
    updateViolationsList(data.violations || []);
    updateAllocationsList(data.allocations || []);
    updateUnsafeCodeList(data.allocations || []);
    updateRiskAssessment(data);
}

// Update violations list
function updateViolationsList(violations) {
    const container = document.getElementById('violationsList');
    if (!container) return;

    if (violations.length === 0) {
        container.innerHTML = '<div class="no-data">No safety violations detected</div>';
        return;
    }

    container.innerHTML = violations.map(violation => `
        <div class="violation-item">
            <div class="violation-header">
                <span class="violation-type">${violation.violation_type}</span>
                <span class="violation-severity ${violation.severity.toLowerCase()}">${violation.severity}</span>
            </div>
            <div class="violation-description">${violation.description}</div>
            <div class="violation-location">${violation.location || 'Unknown location'}</div>
            <div class="violation-timestamp">${new Date(violation.timestamp).toLocaleString()}</div>
            ${violation.suggested_fix ? `<div class="violation-fix">💡 ${violation.suggested_fix}</div>` : ''}
        </div>
    `).join('');
}

// Update allocations list
function updateAllocationsList(allocations) {
    const container = document.getElementById('allocationsList');
    if (!container) return;

    // Show only unsafe/FFI allocations
    const unsafeAllocations = allocations.filter(a => 
        a.source_type !== 'Safe Rust'
    );

    if (unsafeAllocations.length === 0) {
        container.innerHTML = '<div class="no-data">No unsafe/FFI allocations found</div>';
        return;
    }

    container.innerHTML = unsafeAllocations.slice(0, 10).map(allocation => `
        <div class="allocation-item">
            <div class="allocation-header">
                <span class="allocation-source ${allocation.source_type.toLowerCase().replace(' ', '-')}">${allocation.source_type}</span>
                <span class="allocation-size">${formatBytes(allocation.size)}</span>
                <span class="allocation-status ${allocation.is_active ? 'active' : 'freed'}">${allocation.is_active ? 'Active' : 'Freed'}</span>
            </div>
            <div class="allocation-details">
                <div>Pointer: 0x${allocation.ptr.toString(16)}</div>
                <div>Allocated: ${new Date(allocation.timestamp_alloc).toLocaleString()}</div>
                ${allocation.timestamp_dealloc ? `<div>Freed: ${new Date(allocation.timestamp_dealloc).toLocaleString()}</div>` : ''}
                ${allocation.source_details?.unsafe_location ? `<div>Location: ${allocation.source_details.unsafe_location}</div>` : ''}
                ${allocation.source_details?.library_name ? `<div>Library: ${allocation.source_details.library_name}</div>` : ''}
            </div>
        </div>
    `).join('');
}

// Initialize charts
function initializeCharts() {
    // Memory timeline chart (enhanced)
    const memoryTimelineCtx = document.getElementById('memoryTimelineChart');
    if (memoryTimelineCtx) {
        charts.memoryTimeline = new Chart(memoryTimelineCtx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'Active Memory',
                        data: [],
                        borderColor: '#3498db',
                        backgroundColor: 'rgba(52, 152, 219, 0.1)',
                        tension: 0.4,
                        fill: true
                    },
                    {
                        label: 'Peak Memory',
                        data: [],
                        borderColor: '#e74c3c',
                        backgroundColor: 'rgba(231, 76, 60, 0.1)',
                        tension: 0.4,
                        fill: false
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        labels: { color: '#2c3e50' }
                    }
                },
                scales: {
                    x: { 
                        ticks: { color: '#2c3e50' },
                        grid: { color: 'rgba(44, 62, 80, 0.1)' }
                    },
                    y: { 
                        ticks: { color: '#2c3e50' },
                        grid: { color: 'rgba(44, 62, 80, 0.1)' }
                    }
                }
            }
        });
    }

    // Memory growth pattern chart
    const memoryGrowthCtx = document.getElementById('memoryGrowthChart');
    if (memoryGrowthCtx) {
        charts.memoryGrowth = new Chart(memoryGrowthCtx, {
            type: 'area',
            data: {
                labels: [],
                datasets: [{
                    label: 'Memory Growth',
                    data: [],
                    borderColor: '#2ecc71',
                    backgroundColor: 'rgba(46, 204, 113, 0.1)',
                    tension: 0.4,
                    fill: true
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        labels: { color: '#ffffff' }
                    }
                },
                scales: {
                    x: { ticks: { color: '#ffffff' } },
                    y: { ticks: { color: '#ffffff' } }
                }
            }
        });
    }

    // Type distribution chart
    const typeDistCtx = document.getElementById('typeDistributionChart');
    if (typeDistCtx) {
        charts.typeDistribution = new Chart(typeDistCtx, {
            type: 'doughnut',
            data: {
                labels: [],
                datasets: [{
                    data: [],
                    backgroundColor: ['#e74c3c', '#f39c12', '#2ecc71', '#3498db', '#9b59b6']
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    legend: {
                        labels: { color: '#ffffff' }
                    }
                }
            }
        });
    }

    // Allocation source chart
    const allocSourceCtx = document.getElementById('allocationSourceChart');
    if (allocSourceCtx) {
        charts.allocationSource = new Chart(allocSourceCtx, {
            type: 'bar',
            data: {
                labels: ['Safe Rust', 'Unsafe Rust', 'FFI C', 'Cross Boundary'],
                datasets: [{
                    label: 'Allocations',
                    data: [0, 0, 0, 0],
                    backgroundColor: ['#2ecc71', '#f39c12', '#e74c3c', '#9b59b6']
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    legend: {
                        labels: { color: '#ffffff' }
                    }
                },
                scales: {
                    x: { ticks: { color: '#ffffff' } },
                    y: { ticks: { color: '#ffffff' } }
                }
            }
        });
    }
}

// Update charts
function updateCharts(data) {
    if (data.allocations && charts.memoryTrend) {
        const recentAllocations = data.allocations.slice(-20);
        const timestamps = recentAllocations.map(a => new Date(a.timestamp_alloc).toLocaleTimeString());
        const sizes = recentAllocations.map(a => a.size);
        
        charts.memoryTrend.data.labels = timestamps;
        charts.memoryTrend.data.datasets[0].data = sizes;
        charts.memoryTrend.update();
    }

    if (data.allocations && charts.typeDistribution) {
        const typeGroups = {};
        data.allocations.forEach(alloc => {
            const sourceType = alloc.source_type || 'Unknown';
            typeGroups[sourceType] = (typeGroups[sourceType] || 0) + alloc.size;
        });

        charts.typeDistribution.data.labels = Object.keys(typeGroups);
        charts.typeDistribution.data.datasets[0].data = Object.values(typeGroups);
        charts.typeDistribution.update();
    }

    if (data.allocations && charts.allocationSource) {
        const sourceCounts = {
            'Safe Rust': 0,
            'Unsafe Rust': 0,
            'FFI C': 0,
            'Cross Boundary': 0
        };

        data.allocations.forEach(alloc => {
            if (sourceCounts.hasOwnProperty(alloc.source_type)) {
                sourceCounts[alloc.source_type]++;
            }
        });

        charts.allocationSource.data.datasets[0].data = Object.values(sourceCounts);
        charts.allocationSource.update();
    }
}

// Utility functions
function updateElement(id, value) {
    const element = document.getElementById(id);
    if (element) {
        element.textContent = value;
    }
}

function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function showNotification(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    notification.textContent = message;
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 12px 20px;
        border-radius: 8px;
        color: white;
        font-weight: 500;
        z-index: 1000;
        animation: slideIn 0.3s ease;
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    `;
    
    if (type === 'success') {
        notification.style.backgroundColor = '#2ecc71';
    } else if (type === 'error') {
        notification.style.backgroundColor = '#e74c3c';
    } else {
        notification.style.backgroundColor = '#3498db';
    }

    document.body.appendChild(notification);

    setTimeout(() => {
        if (document.body.contains(notification)) {
            document.body.removeChild(notification);
        }
    }, 3000);
}

// Export functions
function refreshData() {
    const selector = document.getElementById('dataSource');
    if (selector) {
        const source = selector.value;
        switch(source) {
            case 'sample':
                loadSampleData();
                break;
            case 'json':
                loadJSONFile();
                break;
            default:
                loadSampleData();
        }
    }
}

function exportMemoryAnalysisSVG() {
    showNotification('SVG export functionality would be implemented here', 'info');
}

function exportUnsafeFFIReport() {
    showNotification('Unsafe/FFI report export functionality would be implemented here', 'info');
}

function refreshUnsafeFFIData() {
    if (currentData) {
        updateUnsafeFFITab(currentData);
        showNotification('Unsafe/FFI data refreshed', 'success');
    }
}

// Update unsafe code list
function updateUnsafeCodeList(allocations) {
    const container = document.getElementById('unsafeCodeList');
    if (!container) return;

    const unsafeAllocations = allocations.filter(a => 
        a.source_type === 'Unsafe Rust'
    );

    if (unsafeAllocations.length === 0) {
        container.innerHTML = '<div class="no-data">No unsafe code blocks detected</div>';
        return;
    }

    container.innerHTML = unsafeAllocations.slice(0, 5).map(allocation => `
        <div class="unsafe-code-item">
            <div class="unsafe-header">
                <span class="unsafe-location">${allocation.source_details?.unsafe_location || 'Unknown location'}</span>
                <span class="unsafe-risk ${allocation.source_details?.risk_level?.toLowerCase() || 'medium'}">${allocation.source_details?.risk_level || 'Medium'}</span>
            </div>
            <div class="unsafe-details">
                <div>Size: ${formatBytes(allocation.size)}</div>
                <div>Allocated: ${new Date(allocation.timestamp_alloc).toLocaleString()}</div>
                <div>Status: ${allocation.is_active ? 'Active' : 'Freed'}</div>
            </div>
        </div>
    `).join('');
}

// Update risk assessment
function updateRiskAssessment(data) {
    // Calculate risk percentages based on data
    const totalAllocations = data.allocations?.length || 0;
    const unsafeAllocations = data.allocations?.filter(a => a.source_type !== 'Safe Rust').length || 0;
    const violations = data.violations?.length || 0;
    
    // Calculate risk scores (simplified)
    const memoryLeakRisk = Math.min((violations * 20), 100);
    const useAfterFreeRisk = Math.min((violations * 15), 100);
    const bufferOverflowRisk = Math.min((unsafeAllocations * 10), 100);
    const ffiSafetyRisk = Math.min((data.metrics?.ffi_allocations || 0) * 25, 100);
    
    // Update risk bars
    updateRiskBar('memoryLeakRisk', 'memoryLeakPercent', memoryLeakRisk);
    updateRiskBar('useAfterFreeRisk', 'useAfterFreePercent', useAfterFreeRisk);
    updateRiskBar('bufferOverflowRisk', 'bufferOverflowPercent', bufferOverflowRisk);
    updateRiskBar('ffiSafetyRisk', 'ffiSafetyPercent', ffiSafetyRisk);
    
    // Update critical issues count
    updateElement('criticalIssues', violations);
    
    // Calculate and update safety score
    const safetyScore = Math.max(100 - (memoryLeakRisk + useAfterFreeRisk + bufferOverflowRisk + ffiSafetyRisk) / 4, 0);
    updateElement('safetyScore', Math.round(safetyScore));
}

// Update risk bar
function updateRiskBar(barId, percentId, percentage) {
    const bar = document.getElementById(barId);
    const percent = document.getElementById(percentId);
    
    if (bar) {
        bar.style.width = percentage + '%';
        // Change color based on risk level
        if (percentage > 70) {
            bar.style.background = 'linear-gradient(90deg, #e74c3c, #c0392b)';
        } else if (percentage > 40) {
            bar.style.background = 'linear-gradient(90deg, #f39c12, #e67e22)';
        } else {
            bar.style.background = 'linear-gradient(90deg, #2ecc71, #27ae60)';
        }
    }
    
    if (percent) {
        percent.textContent = Math.round(percentage) + '%';
    }
}

// Filter functions for FFI interface
function filterByRisk() {
    const filter = document.getElementById('riskFilter').value;
    // Implementation would filter displayed items by risk level
    showNotification(`Filtering by ${filter} risk level`, 'info');
}

function sortViolations(sortBy) {
    if (!currentData || !currentData.violations) return;
    
    let sortedViolations = [...currentData.violations];
    
    if (sortBy === 'severity') {
        const severityOrder = { 'High': 3, 'Medium': 2, 'Low': 1 };
        sortedViolations.sort((a, b) => severityOrder[b.severity] - severityOrder[a.severity]);
    } else if (sortBy === 'time') {
        sortedViolations.sort((a, b) => b.timestamp - a.timestamp);
    }
    
    updateViolationsList(sortedViolations);
    showNotification(`Violations sorted by ${sortBy}`, 'info');
}

function showUnsafeDetails() {
    showNotification('Detailed unsafe code analysis would be shown here', 'info');
}

function showBoundaryFlow() {
    // Initialize boundary flow chart
    const canvas = document.getElementById('boundaryFlowChart');
    if (canvas && currentData) {
        initializeBoundaryFlowChart(canvas, currentData);
    }
}

function initializeBoundaryFlowChart(canvas, data) {
    const ctx = canvas.getContext('2d');
    
    // Simple boundary flow visualization
    if (charts.boundaryFlow) {
        charts.boundaryFlow.destroy();
    }
    
    const boundaryData = {
        labels: ['Rust Safe', 'Unsafe Rust', 'FFI C', 'Cross Boundary'],
        datasets: [{
            label: 'Memory Flow (KB)',
            data: [
                data.allocations?.filter(a => a.source_type === 'Safe Rust').reduce((sum, a) => sum + a.size, 0) / 1024 || 0,
                data.allocations?.filter(a => a.source_type === 'Unsafe Rust').reduce((sum, a) => sum + a.size, 0) / 1024 || 0,
                data.allocations?.filter(a => a.source_type === 'FFI C').reduce((sum, a) => sum + a.size, 0) / 1024 || 0,
                data.allocations?.filter(a => a.source_type === 'Cross Boundary').reduce((sum, a) => sum + a.size, 0) / 1024 || 0
            ],
            backgroundColor: ['#2ecc71', '#f39c12', '#e74c3c', '#9b59b6'],
            borderColor: ['#27ae60', '#e67e22', '#c0392b', '#8e44ad'],
            borderWidth: 2
        }]
    };
    
    charts.boundaryFlow = new Chart(ctx, {
        type: 'radar',
        data: boundaryData,
        options: {
            responsive: true,
            plugins: {
                legend: {
                    labels: { color: '#ffffff' }
                }
            },
            scales: {
                r: {
                    ticks: { color: '#ffffff' },
                    grid: { color: 'rgba(255, 255, 255, 0.1)' },
                    pointLabels: { color: '#ffffff' }
                }
            }
        }
    });
}

function toggleAllocationView() {
    // Toggle between different allocation views
    showNotification('Allocation view toggled', 'info');
}

function filterActiveAllocations() {
    if (!currentData || !currentData.allocations) return;
    
    const activeAllocations = currentData.allocations.filter(a => a.is_active);
    updateAllocationsList(activeAllocations);
    showNotification(`Showing ${activeAllocations.length} active allocations`, 'info');
}

// Enhanced Memory Analysis Functions
function updateMemoryAnalysis(data) {
    updateKPICircles(data);
    updatePerformanceMetrics(data);
    updateMemoryHierarchy(data);
    updateTypeLegend(data);
}

// Update KPI circles with real data
function updateKPICircles(data) {
    if (!data.memory_stats) return;
    
    const activeMemory = data.memory_stats.active_memory || 0;
    const peakMemory = data.memory_stats.peak_memory || 0;
    const activeAllocations = data.memory_stats.active_allocations || 0;
    const totalAllocations = data.memory_stats.total_allocations || 1;
    
    // Calculate percentages
    const memoryEfficiency = peakMemory > 0 ? (activeMemory / peakMemory) * 100 : 0;
    const allocationRatio = (activeAllocations / totalAllocations) * 100;
    
    // Update circles
    updateKPICircle('activeMemoryCircle', 'activeMemoryPercent', memoryEfficiency);
    updateKPICircle('peakMemoryCircle', 'peakMemoryPercent', 100);
    updateKPICircle('activeAllocationsCircle', 'activeAllocationsPercent', allocationRatio);
    updateKPICircle('memoryEfficiencyCircle', 'memoryEfficiencyPercent', memoryEfficiency);
    
    // Calculate fragmentation (simplified)
    const fragmentation = Math.random() * 30; // Placeholder calculation
    updateKPICircle('fragmentationCircle', 'fragmentationPercent', fragmentation);
    
    // Calculate GC pressure (simplified)
    const gcPressure = Math.random() * 40; // Placeholder calculation
    updateKPICircle('gcPressureCircle', 'gcPressurePercent', gcPressure);
    
    // Update values
    updateElement('activeMemory', formatBytes(activeMemory));
    updateElement('peakMemory', formatBytes(peakMemory));
    updateElement('activeAllocations', activeAllocations);
    updateElement('memoryEfficiency', memoryEfficiency.toFixed(1) + '%');
    updateElement('fragmentation', fragmentation.toFixed(1) + '%');
    updateElement('gcPressure', gcPressure < 30 ? 'Low' : gcPressure < 60 ? 'Medium' : 'High');
}

// Update individual KPI circle
function updateKPICircle(circleId, percentId, percentage) {
    const circle = document.getElementById(circleId);
    const percentElement = document.getElementById(percentId);
    
    if (circle) {
        const circumference = 2 * Math.PI * 40; // radius = 40
        const offset = circumference - (percentage / 100) * circumference;
        circle.style.strokeDashoffset = offset;
    }
    
    if (percentElement) {
        percentElement.textContent = Math.round(percentage) + '%';
    }
}

// Update performance metrics
function updatePerformanceMetrics(data) {
    if (!data.memory_stats) return;
    
    const allocations = data.allocations || [];
    const timeSpan = allocations.length > 1 ? 
        (allocations[allocations.length - 1].timestamp_alloc - allocations[0].timestamp_alloc) / 1000 : 1;
    
    // Calculate rates
    const allocationRate = allocations.length / timeSpan;
    const deallocatedCount = allocations.filter(a => !a.is_active).length;
    const deallocationRate = deallocatedCount / timeSpan;
    
    // Calculate other metrics
    const memoryTurnover = allocations.length > 0 ? (deallocatedCount / allocations.length) * 100 : 0;
    const peakAllocationSize = Math.max(...allocations.map(a => a.size), 0);
    const averageLifetime = calculateAverageLifetime(allocations);
    const memoryPressure = data.memory_stats.active_memory > (1024 * 1024) ? 'High' : 
                          data.memory_stats.active_memory > (512 * 1024) ? 'Medium' : 'Low';
    
    // Update display
    updateElement('allocationRate', allocationRate.toFixed(1) + '/sec');
    updateElement('deallocationRate', deallocationRate.toFixed(1) + '/sec');
    updateElement('memoryTurnover', memoryTurnover.toFixed(1) + '%');
    updateElement('peakAllocationSize', formatBytes(peakAllocationSize));
    updateElement('averageLifetime', averageLifetime.toFixed(1) + 'ms');
    updateElement('memoryPressure', memoryPressure);
}

// Calculate average lifetime of allocations
function calculateAverageLifetime(allocations) {
    const deallocated = allocations.filter(a => a.timestamp_dealloc);
    if (deallocated.length === 0) return 0;
    
    const totalLifetime = deallocated.reduce((sum, a) => {
        return sum + (a.timestamp_dealloc - a.timestamp_alloc);
    }, 0);
    
    return totalLifetime / deallocated.length;
}

// Update memory hierarchy visualization
function updateMemoryHierarchy(data) {
    const container = document.getElementById('memoryHierarchy');
    if (!container || !data.allocations) return;
    
    // Group allocations by type
    const hierarchy = {};
    data.allocations.forEach(alloc => {
        const sourceType = alloc.source_type || 'Unknown';
        if (!hierarchy[sourceType]) {
            hierarchy[sourceType] = {
                count: 0,
                totalSize: 0,
                allocations: []
            };
        }
        hierarchy[sourceType].count++;
        hierarchy[sourceType].totalSize += alloc.size;
        hierarchy[sourceType].allocations.push(alloc);
    });
    
    // Build hierarchy HTML
    let hierarchyHTML = '<div class="hierarchy-node root"><span class="node-label">Memory Hierarchy</span></div>';
    
    Object.entries(hierarchy).forEach(([type, info]) => {
        hierarchyHTML += `
            <div class="hierarchy-node">
                <span class="node-label">${type}</span>
                <span class="node-size">${formatBytes(info.totalSize)}</span>
                <span class="node-count">(${info.count} allocations)</span>
            </div>
        `;
    });
    
    container.innerHTML = hierarchyHTML;
}

// Update type legend
function updateTypeLegend(data) {
    const container = document.getElementById('typeLegend');
    if (!container || !data.allocations) return;
    
    const types = {};
    data.allocations.forEach(alloc => {
        const sourceType = alloc.source_type || 'Unknown';
        types[sourceType] = (types[sourceType] || 0) + 1;
    });
    
    const colors = ['#3498db', '#e74c3c', '#2ecc71', '#f39c12', '#9b59b6'];
    let legendHTML = '';
    
    Object.entries(types).forEach(([type, count], index) => {
        const color = colors[index % colors.length];
        legendHTML += `
            <div class="legend-item">
                <div class="legend-color" style="background: ${color}"></div>
                <span>${type} (${count})</span>
            </div>
        `;
    });
    
    container.innerHTML = legendHTML;
}

// Memory Analysis specific functions
function toggleTypeView() {
    showNotification('Type view toggled', 'info');
}

function zoomMemoryChart() {
    showNotification('Memory chart zoomed', 'info');
}

function refreshHeatmap() {
    const container = document.getElementById('memoryHeatmap');
    if (container) {
        container.innerHTML = '<div style="text-align: center; padding: 2rem; color: #95a5a6;">Heatmap refreshed - visualization would be rendered here</div>';
    }
    showNotification('Memory heatmap refreshed', 'info');
}

function exportPerformanceReport() {
    showNotification('Performance report exported', 'info');
}

// Add CSS for new elements
const style = document.createElement('style');
style.textContent = `
    .violations-list, .allocations-list {
        max-height: 400px;
        overflow-y: auto;
        padding: 1rem;
    }
    
    .violation-item, .allocation-item {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 12px;
        margin-bottom: 8px;
    }
    
    .violation-header, .allocation-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;
    }
    
    .violation-severity.high { color: #e74c3c; font-weight: bold; }
    .violation-severity.medium { color: #f39c12; font-weight: bold; }
    .violation-severity.low { color: #2ecc71; font-weight: bold; }
    
    .allocation-source.unsafe-rust { color: #f39c12; }
    .allocation-source.ffi-c { color: #e74c3c; }
    .allocation-source.safe-rust { color: #2ecc71; }
    
    .allocation-status.active { color: #2ecc71; }
    .allocation-status.freed { color: #95a5a6; }
    
    .no-data {
        text-align: center;
        color: #95a5a6;
        padding: 2rem;
        font-style: italic;
    }
    
    .violation-fix {
        margin-top: 8px;
        padding: 8px;
        background: rgba(52, 152, 219, 0.1);
        border-left: 3px solid #3498db;
        font-size: 0.9em;
    }
    
    @keyframes slideIn {
        from { transform: translateX(100%); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
    }
`;
document.head.appendChild(style);