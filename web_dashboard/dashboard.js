// Global variables
let currentData = null;
let uploadedFiles = new Map(); // Store uploaded files
let chartInstances = {}; // Store chart instances for cleanup

// Initialize dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    initializeDashboard();
    setupEventListeners();
    loadDefaultData();
});

// Initialize dashboard components
function initializeDashboard() {
    // Setup tab navigation
    const navTabs = document.querySelectorAll('.nav-tab');
    navTabs.forEach(tab => {
        tab.addEventListener('click', function() {
            const targetTab = this.getAttribute('data-tab');
            switchTab(targetTab);
        });
    });

    // Setup data source selector
    const dataSourceSelect = document.getElementById('dataSource');
    if (dataSourceSelect) {
        dataSourceSelect.addEventListener('change', handleDataSourceChange);
    }
}

// Setup event listeners
function setupEventListeners() {
    // Create file input for uploads
    createFileUploadInput();
}

// Create hidden file input for uploads
function createFileUploadInput() {
    const fileInput = document.createElement('input');
    fileInput.type = 'file';
    fileInput.accept = '.json';
    fileInput.multiple = true;
    fileInput.style.display = 'none';
    fileInput.id = 'fileUploadInput';
    fileInput.addEventListener('change', handleFileUpload);
    document.body.appendChild(fileInput);
}

// Handle data source change
function handleDataSourceChange() {
    const dataSource = document.getElementById('dataSource').value;
    
    if (dataSource === 'upload') {
        // Trigger file upload dialog
        document.getElementById('fileUploadInput').click();
    } else if (dataSource === 'sample') {
        loadSampleData();
    } else if (dataSource === 'json') {
        autoDetectJSON();
    } else {
        // Load specific JSON file
        loadJSONFile(dataSource);
    }
}

// Handle file upload
function handleFileUpload(event) {
    const files = event.target.files;
    if (files.length === 0) {
        // User cancelled, reset to previous selection
        resetDataSourceSelector();
        return;
    }

    Array.from(files).forEach(file => {
        if (file.type === 'application/json' || file.name.endsWith('.json')) {
            const reader = new FileReader();
            reader.onload = function(e) {
                try {
                    const data = JSON.parse(e.target.result);
                    uploadedFiles.set(file.name, data);
                    addUploadedFileToSelector(file.name);
                    loadData(data);
                    showNotification(`Successfully loaded ${file.name}`, 'success');
                } catch (error) {
                    showNotification(`Error parsing ${file.name}: ${error.message}`, 'error');
                }
            };
            reader.readAsText(file);
        } else {
            showNotification(`Invalid file type: ${file.name}. Please upload JSON files only.`, 'error');
        }
    });
}

// Add uploaded file to selector
function addUploadedFileToSelector(fileName) {
    const dataSourceSelect = document.getElementById('dataSource');
    
    // Remove existing option if it exists
    const existingOption = dataSourceSelect.querySelector(`option[value="${fileName}"]`);
    if (existingOption) {
        existingOption.remove();
    }
    
    // Add new option
    const option = document.createElement('option');
    option.value = fileName;
    option.textContent = `[FILE] ${fileName}`;
    option.selected = true;
    dataSourceSelect.appendChild(option);
}

// Reset data source selector
function resetDataSourceSelector() {
    const dataSourceSelect = document.getElementById('dataSource');
    dataSourceSelect.value = 'sample';
}

// Load default data
function loadDefaultData() {
    loadSampleData();
}

// Load sample data
function loadSampleData() {
    const sampleData = generateSampleData();
    loadData(sampleData);
}

// Auto-detect JSON files
function autoDetectJSON() {
    // Try to load from common locations
    const commonFiles = [
        'images/lifecycle_snapshot.json',
        'lifecycle_snapshot.json',
        'memory_analysis.json',
        'unsafe_ffi_memory_snapshot.json',
        'complex_lifecycle_snapshot.json'
    ];
    
    // Try each file until one loads successfully
    let fileIndex = 0;
    function tryNextFile() {
        if (fileIndex >= commonFiles.length) {
            showNotification('No JSON files found. Using sample data.', 'warning');
            loadSampleData();
            return;
        }
        
        const fileName = commonFiles[fileIndex];
        fetch(fileName)
            .then(response => {
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                return response.json();
            })
            .then(data => {
                loadData(data);
                showNotification(`Successfully loaded ${fileName}`, 'success');
            })
            .catch(error => {
                fileIndex++;
                tryNextFile();
            });
    }
    
    tryNextFile();
}

// Load JSON file
function loadJSONFile(fileName) {
    // Check if it's an uploaded file first
    if (uploadedFiles.has(fileName)) {
        loadData(uploadedFiles.get(fileName));
        return;
    }
    
    // Try to load from server/local
    fetch(fileName)
        .then(response => {
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            return response.json();
        })
        .then(data => {
            loadData(data);
            showNotification(`Successfully loaded ${fileName}`, 'success');
        })
        .catch(error => {
            console.warn(`Could not load ${fileName}:`, error);
            showNotification(`Could not load ${fileName}. Using sample data instead.`, 'warning');
            loadSampleData();
        });
}

// Load data into dashboard
function loadData(data) {
    currentData = data;
    updateAllVisualizations();
}

// Update all visualizations
function updateAllVisualizations() {
    if (!currentData) return;
    
    updateMemoryAnalysis();
    updateLifecycleVisualization();
    updateUnsafeFFIAnalysis();
    updateMetrics();
}

// Update memory analysis
function updateMemoryAnalysis() {
    updateMemoryMetrics();
    updateMemoryCharts();
    updateMemoryHierarchy();
}

// Update memory metrics
function updateMemoryMetrics() {
    const memoryStats = extractMemoryStats(currentData);
    
    // Update metric cards
    updateMetricCard('totalMemory', formatBytes(memoryStats.total_allocated_bytes || 0));
    updateMetricCard('activeAllocations', memoryStats.active_allocations || 0);
    updateMetricCard('peakMemory', formatBytes(memoryStats.peak_memory_usage_bytes || 0));
    updateMetricCard('memoryLeaks', memoryStats.memory_leaks_detected || 0);
}

// Update metric card
function updateMetricCard(id, value) {
    const element = document.getElementById(id);
    if (element) {
        element.textContent = value;
    }
}

// Update memory charts
function updateMemoryCharts() {
    updateAllocationChart();
    updateMemoryUsageChart();
    updateAllocationSourceChart();
}

// Update allocation chart
function updateAllocationChart() {
    const ctx = document.getElementById('allocationChart');
    if (!ctx) return;
    
    // Destroy existing chart if it exists
    if (chartInstances.allocationChart) {
        chartInstances.allocationChart.destroy();
    }
    
    // Extract data from the actual JSON format
    const memoryStats = extractMemoryStats(currentData);
    
    chartInstances.allocationChart = new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: ['Active', 'Freed', 'Leaked'],
            datasets: [{
                data: [
                    memoryStats.active_allocations || 0,
                    memoryStats.freed_allocations || 0,
                    memoryStats.memory_leaks_detected || 0
                ],
                backgroundColor: ['#3498db', '#2ecc71', '#e74c3c'],
                borderWidth: 2,
                borderColor: '#2c3e50'
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: true,
            aspectRatio: 1.5,
            plugins: {
                legend: {
                    position: 'bottom',
                    labels: { 
                        color: '#ffffff',
                        padding: 20,
                        font: { size: 12 }
                    }
                }
            }
        }
    });
}

// Update memory usage chart
function updateMemoryUsageChart() {
    const ctx = document.getElementById('memoryUsageChart');
    if (!ctx) return;
    
    // Destroy existing chart if it exists
    if (chartInstances.memoryUsageChart) {
        chartInstances.memoryUsageChart.destroy();
    }
    
    const timeline = extractTimelineData(currentData);
    
    chartInstances.memoryUsageChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: timeline.labels,
            datasets: [{
                label: 'Memory Usage',
                data: timeline.data,
                borderColor: '#3498db',
                backgroundColor: 'rgba(52, 152, 219, 0.1)',
                fill: true,
                tension: 0.4
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: true,
            aspectRatio: 2,
            scales: {
                y: {
                    ticks: { 
                        color: '#ffffff',
                        callback: function(value) {
                            return formatBytes(value);
                        }
                    },
                    grid: { color: 'rgba(255, 255, 255, 0.1)' }
                },
                x: {
                    ticks: { color: '#ffffff' },
                    grid: { color: 'rgba(255, 255, 255, 0.1)' }
                }
            },
            plugins: {
                legend: {
                    labels: { color: '#ffffff' }
                }
            }
        }
    });
}

// Update allocation source chart
function updateAllocationSourceChart() {
    const ctx = document.getElementById('allocationSourceChart');
    if (!ctx) return;
    
    // Destroy existing chart if it exists
    if (chartInstances.allocationSourceChart) {
        chartInstances.allocationSourceChart.destroy();
    }
    
    const sources = extractAllocationSources(currentData);
    
    chartInstances.allocationSourceChart = new Chart(ctx, {
        type: 'bar',
        data: {
            labels: sources.labels,
            datasets: [{
                label: 'Allocations',
                data: sources.data,
                backgroundColor: '#3498db',
                borderColor: '#2980b9',
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: true,
            aspectRatio: 2,
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: { color: '#ffffff' },
                    grid: { color: 'rgba(255, 255, 255, 0.1)' }
                },
                x: {
                    ticks: { 
                        color: '#ffffff',
                        maxRotation: 45
                    },
                    grid: { color: 'rgba(255, 255, 255, 0.1)' }
                }
            },
            plugins: {
                legend: {
                    labels: { color: '#ffffff' }
                }
            }
        }
    });
}

// Update memory hierarchy
function updateMemoryHierarchy() {
    const container = document.getElementById('memoryHierarchy');
    if (!container) return;
    
    const hierarchy = currentData.memory_hierarchy || {};
    displayMemoryHierarchyTree(hierarchy, container);
}

// Display memory hierarchy tree
function displayMemoryHierarchyTree(memoryHierarchy, container) {
    let hierarchyHTML = "<div class=\"hierarchy-root\">";
    
    for (const [categoryName, categoryData] of Object.entries(memoryHierarchy)) {
        hierarchyHTML += `
            <div class="hierarchy-category">
                <div class="category-header" onclick="toggleCategory('${categoryName}')">
                    <span class="category-icon">[+]</span>
                    <span class="category-name">${categoryName}</span>
                    <span class="category-summary">${categoryData.summary?.total_size_bytes || 0} bytes</span>
                </div>
                <div class="category-content" id="category-${categoryName}" style="display: block;">
        `;
        
        if (categoryData.subcategories) {
            for (const [subName, subData] of Object.entries(categoryData.subcategories)) {
                hierarchyHTML += `
                    <div class="subcategory">
                        <div class="subcategory-header">
                            <span class="subcategory-icon">[-]</span>
                            <span class="subcategory-name">${subName}</span>
                            <span class="subcategory-summary">${subData.summary?.total_size_bytes || 0} bytes</span>
                        </div>
                `;
                
                if (subData.types) {
                    subData.types.forEach(type => {
                        hierarchyHTML += `
                            <div class="type-item">
                                <div class="type-header">
                                    <span class="type-name">${type.type_name || "Unknown"}</span>
                                    <span class="type-size">${type.size_bytes || 0} bytes</span>
                                    <span class="type-count">${type.allocation_count || 0} allocations</span>
                                </div>
                            </div>
                        `;
                    });
                }
                
                hierarchyHTML += "</div>";
            }
        }
        
        hierarchyHTML += "</div></div>";
    }
    
    hierarchyHTML += "</div>";
    container.innerHTML = hierarchyHTML;
}

// Toggle category visibility
function toggleCategory(categoryName) {
    const content = document.getElementById(`category-${categoryName}`);
    if (content) {
        content.style.display = content.style.display === "none" ? "block" : "none";
    }
}

// Update lifecycle visualization
function updateLifecycleVisualization() {
    const container = document.getElementById("lifecycleVisualization");
    if (!container) return;
    
    if (!currentData.lifecycle_stats) {
        container.innerHTML = "<div class=\"no-data\">No lifecycle data available.</div>";
        return;
    }
    
    const lifecycle = currentData.lifecycle_stats;
    
    let timelineHTML = `
        <div class="lifecycle-summary">
            <div class="lifecycle-stat">
                <span class="stat-label">Average Lifetime:</span>
                <span class="stat-value">${lifecycle.average_lifetime_ms || 0}ms</span>
            </div>
            <div class="lifecycle-stat">
                <span class="stat-label">Memory Leaks:</span>
                <span class="stat-value">${lifecycle.memory_leaks_detected || 0}</span>
            </div>
            <div class="lifecycle-stat">
                <span class="stat-label">Short-lived Objects:</span>
                <span class="stat-value">${lifecycle.short_lived_objects || 0}</span>
            </div>
            <div class="lifecycle-stat">
                <span class="stat-label">Long-lived Objects:</span>
                <span class="stat-value">${lifecycle.long_lived_objects || 0}</span>
            </div>
        </div>
    `;
    
    container.innerHTML = timelineHTML;
}

// Update unsafe FFI analysis
function updateUnsafeFFIAnalysis() {
    const container = document.getElementById("unsafeFFIAnalysis");
    if (!container) return;
    
    const unsafeStats = currentData.unsafe_ffi_stats || {};
    
    let analysisHTML = `
        <div class="unsafe-ffi-summary">
            <div class="unsafe-stat">
                <span class="stat-label">Unsafe Blocks:</span>
                <span class="stat-value">${unsafeStats.unsafe_blocks_count || 0}</span>
            </div>
            <div class="unsafe-stat">
                <span class="stat-label">FFI Calls:</span>
                <span class="stat-value">${unsafeStats.ffi_calls_count || 0}</span>
            </div>
            <div class="unsafe-stat">
                <span class="stat-label">Raw Pointer Operations:</span>
                <span class="stat-value">${unsafeStats.raw_pointer_ops || 0}</span>
            </div>
            <div class="unsafe-stat">
                <span class="stat-label">Memory Safety Issues:</span>
                <span class="stat-value">${unsafeStats.safety_issues || 0}</span>
            </div>
        </div>
    `;
    
    container.innerHTML = analysisHTML;
}

// Update metrics
function updateMetrics() {
    // This is called by updateMemoryAnalysis, but can be extended for other metrics
}

// Switch tab
function switchTab(tabName) {
    // Hide all tab contents
    const tabContents = document.querySelectorAll('.tab-content');
    tabContents.forEach(content => content.classList.remove('active'));
    
    // Remove active class from all nav tabs
    const navTabs = document.querySelectorAll('.nav-tab');
    navTabs.forEach(tab => tab.classList.remove('active'));
    
    // Show selected tab content
    const selectedContent = document.getElementById(tabName);
    if (selectedContent) {
        selectedContent.classList.add('active');
    }
    
    // Add active class to selected nav tab
    const selectedTab = document.querySelector(`[data-tab="${tabName}"]`);
    if (selectedTab) {
        selectedTab.classList.add('active');
    }
}

// Export functions
function exportMemoryAnalysisSVG() {
    showNotification('Exporting memory analysis SVG...', 'info');
    // Implementation for SVG export
}

function exportLifecycleTimelineSVG() {
    showNotification('Exporting lifecycle timeline SVG...', 'info');
    // Implementation for SVG export
}

function exportUnsafeFFIReport() {
    showNotification('Exporting unsafe FFI report SVG...', 'info');
    // Implementation for SVG export
}

function exportAllReports() {
    showNotification('Exporting all reports...', 'info');
    // Implementation for all reports export
}

function exportJSON() {
    if (!currentData) {
        showNotification('No data to export', 'error');
        return;
    }
    
    const dataStr = JSON.stringify(currentData, null, 2);
    const blob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    
    const a = document.createElement('a');
    a.href = url;
    a.download = 'memory_analysis_export.json';
    a.click();
    
    URL.revokeObjectURL(url);
    showNotification('JSON data exported successfully', 'success');
}

function exportCSV() {
    showNotification('Exporting CSV data...', 'info');
    // Implementation for CSV export
}

function refreshData() {
    const dataSource = document.getElementById('dataSource').value;
    handleDataSourceChange();
    showNotification('Data refreshed', 'success');
}

// Utility functions
function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function showNotification(message, type = 'info') {
    // Create notification element
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    notification.textContent = message;
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 1rem 1.5rem;
        border-radius: 8px;
        color: white;
        font-weight: 500;
        z-index: 1000;
        animation: slideIn 0.3s ease;
    `;
    
    // Set background color based on type
    const colors = {
        success: '#2ecc71',
        error: '#e74c3c',
        warning: '#f39c12',
        info: '#3498db'
    };
    notification.style.backgroundColor = colors[type] || colors.info;
    
    document.body.appendChild(notification);
    
    // Remove notification after 3 seconds
    setTimeout(() => {
        notification.style.animation = 'slideOut 0.3s ease';
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 300);
    }, 3000);
}

// Generate sample data
function generateSampleData() {
    return {
        memory_stats: {
            total_allocated_bytes: 1048576,
            active_allocations: 150,
            freed_allocations: 1200,
            peak_memory_usage_bytes: 2097152,
            memory_leaks_detected: 3
        },
        lifecycle_stats: {
            average_lifetime_ms: 250,
            memory_leaks_detected: 3,
            short_lived_objects: 800,
            long_lived_objects: 45
        },
        unsafe_ffi_stats: {
            unsafe_blocks_count: 12,
            ffi_calls_count: 8,
            raw_pointer_ops: 25,
            safety_issues: 2
        },
        memory_hierarchy: {
            "Heap Allocations": {
                summary: { total_size_bytes: 524288 },
                subcategories: {
                    "Vec<T>": {
                        summary: { total_size_bytes: 262144 },
                        types: [
                            { type_name: "Vec<u8>", size_bytes: 131072, allocation_count: 45 },
                            { type_name: "Vec<String>", size_bytes: 131072, allocation_count: 23 }
                        ]
                    }
                }
            }
        },
        memory_timeline: [
            { memory_usage_bytes: 100000 },
            { memory_usage_bytes: 250000 },
            { memory_usage_bytes: 400000 },
            { memory_usage_bytes: 300000 },
            { memory_usage_bytes: 200000 }
        ],
        allocation_sources: {
            "std::vec::Vec": { count: 45 },
            "std::collections::HashMap": { count: 23 },
            "Custom Allocator": { count: 12 }
        }
    };
}

// Data extraction functions to handle different JSON formats
function extractMemoryStats(data) {
    // Handle the actual program output format
    if (data.memory_hierarchy) {
        let totalBytes = 0;
        let totalAllocations = 0;
        
        // Calculate from memory hierarchy
        for (const [categoryName, categoryData] of Object.entries(data.memory_hierarchy)) {
            if (categoryData.subcategories) {
                for (const [subName, subData] of Object.entries(categoryData.subcategories)) {
                    if (subData.summary) {
                        totalBytes += subData.summary.total_size_bytes || 0;
                    }
                    if (subData.types) {
                        subData.types.forEach(type => {
                            totalAllocations += type.allocation_count || 0;
                        });
                    }
                }
            }
        }
        
        return {
            total_allocated_bytes: totalBytes,
            active_allocations: totalAllocations,
            freed_allocations: Math.floor(totalAllocations * 0.8), // Estimate
            peak_memory_usage_bytes: Math.floor(totalBytes * 1.2), // Estimate
            memory_leaks_detected: Math.floor(totalAllocations * 0.02) // Estimate
        };
    }
    
    // Handle sample data format
    return data.memory_stats || {};
}

function extractTimelineData(data) {
    // Handle actual program output - create timeline from allocation times
    if (data.memory_hierarchy) {
        const timePoints = [];
        
        for (const [categoryName, categoryData] of Object.entries(data.memory_hierarchy)) {
            if (categoryData.subcategories) {
                for (const [subName, subData] of Object.entries(categoryData.subcategories)) {
                    if (subData.types) {
                        subData.types.forEach(type => {
                            if (type.allocations) {
                                type.allocations.forEach(alloc => {
                                    timePoints.push({
                                        time: alloc.allocation_time,
                                        size: alloc.size_bytes
                                    });
                                });
                            }
                        });
                    }
                }
            }
        }
        
        // Sort by time and create cumulative data
        timePoints.sort((a, b) => a.time - b.time);
        let cumulativeSize = 0;
        const labels = [];
        const data = [];
        
        timePoints.forEach((point, index) => {
            cumulativeSize += point.size;
            labels.push(`T${index}`);
            data.push(cumulativeSize);
        });
        
        return { labels, data };
    }
    
    // Handle sample data format
    const timeline = data.memory_timeline || [];
    return {
        labels: timeline.map((_, i) => `T${i}`),
        data: timeline.map(t => t.memory_usage_bytes || 0)
    };
}

function extractAllocationSources(data) {
    // Handle actual program output
    if (data.memory_hierarchy) {
        const sources = {};
        
        for (const [categoryName, categoryData] of Object.entries(data.memory_hierarchy)) {
            if (categoryData.subcategories) {
                for (const [subName, subData] of Object.entries(categoryData.subcategories)) {
                    if (subData.types) {
                        subData.types.forEach(type => {
                            const typeName = type.type_name || 'Unknown';
                            const shortName = typeName.split('::').pop() || typeName;
                            sources[shortName] = (sources[shortName] || 0) + (type.allocation_count || 0);
                        });
                    }
                }
            }
        }
        
        return {
            labels: Object.keys(sources),
            data: Object.values(sources)
        };
    }
    
    // Handle sample data format
    const sources = data.allocation_sources || {};
    return {
        labels: Object.keys(sources),
        data: Object.values(sources).map(s => s.count || 0)
    };
}

// Add CSS for animations
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from { transform: translateX(100%); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
    }
    
    @keyframes slideOut {
        from { transform: translateX(0); opacity: 1; }
        to { transform: translateX(100%); opacity: 0; }
    }
    
    .chart-container {
        position: relative;
        height: 300px;
        margin-bottom: 2rem;
    }
    
    .chart-container canvas {
        max-height: 300px !important;
    }
`;
document.head.appendChild(style);