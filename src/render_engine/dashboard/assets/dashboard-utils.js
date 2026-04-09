// Dashboard Utility Functions
window.DashboardUtils = {
    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    },

    formatDuration(ms) {
        if (ms < 1000) return ms.toFixed(1) + 'ms';
        if (ms < 60000) return (ms / 1000).toFixed(2) + 's';
        return (ms / 60000).toFixed(2) + 'm';
    },

    formatPercentage(value) {
        return (value * 100).toFixed(1) + '%';
    },

    getChartColors() {
        const isDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
        return {
            primary: '#3b82f6',
            success: '#10b981',
            warning: '#f59e0b',
            danger: '#ef4444',
            text: isDark ? '#e5e7eb' : '#1f2937',
            text2: isDark ? '#9ca3af' : '#6b7280',
            grid: isDark ? '#374151' : '#e5e7eb',
            bg: isDark ? '#1f2937' : '#ffffff',
            bg2: isDark ? '#111827' : '#f9fafb',
            bg3: isDark ? '#374151' : '#f3f4f6'
        };
    },

    getEfficiencyClass(score) {
        if (score >= 0.8) return 'success';
        if (score >= 0.5) return 'warning';
        return 'danger';
    },

    getStatusColor(isCompleted) {
        return isCompleted ? '#10b981' : '#3b82f6';
    },

    debounce(func, wait) {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    },

    throttle(func, limit) {
        let inThrottle;
        return function(...args) {
            if (!inThrottle) {
                func.apply(this, args);
                inThrottle = true;
                setTimeout(() => inThrottle = false, limit);
            }
        };
    }
};

// Chart.js default configuration
if (typeof Chart !== 'undefined') {
    Chart.defaults.font.family = "'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";
    Chart.defaults.color = DashboardUtils.getChartColors().text;
}
