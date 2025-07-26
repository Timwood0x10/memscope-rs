// Basic JavaScript for MemScope dashboard
document.addEventListener('DOMContentLoaded', function() {
    console.log('MemScope dashboard loaded');
    
    // Initialize dashboard
    initializeDashboard();
});

function initializeDashboard() {
    // Basic dashboard initialization
    console.log('Initializing MemScope dashboard...');
    
    // Show loading message
    showLoading();
    
    // Simulate data loading
    setTimeout(() => {
        hideLoading();
        showSuccess('Dashboard initialized successfully');
    }, 1000);
}

function showLoading() {
    const container = document.querySelector('.container');
    if (container) {
        const loading = document.createElement('div');
        loading.className = 'loading';
        loading.id = 'loading';
        loading.innerHTML = '<p>Loading memory analysis data...</p>';
        container.appendChild(loading);
    }
}

function hideLoading() {
    const loading = document.getElementById('loading');
    if (loading) {
        loading.remove();
    }
}

function showSuccess(message) {
    const container = document.querySelector('.container');
    if (container) {
        const success = document.createElement('div');
        success.className = 'success';
        success.innerHTML = `<p>${message}</p>`;
        container.appendChild(success);
        
        // Auto-remove after 3 seconds
        setTimeout(() => {
            success.remove();
        }, 3000);
    }
}

function showError(message) {
    const container = document.querySelector('.container');
    if (container) {
        const error = document.createElement('div');
        error.className = 'error';
        error.innerHTML = `<p>Error: ${message}</p>`;
        container.appendChild(error);
    }
}

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