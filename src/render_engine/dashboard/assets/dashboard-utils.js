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
        const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
        // Morandi palette — low-saturation earth tones, no bright blue/purple/red
        return {
            primary: '#6B7280',   // slate gray (main neutral)
            success: '#7A8B6F',   // muted sage
            warning: '#C9A876',   // muted ochre
            danger: '#A4716C',    // muted clay red
            info: '#6B8B95',      // muted teal-gray
            accent: '#B08968',    // single clay accent
            text: isDark ? '#E8E2D5' : '#1F1B16',
            text2: isDark ? '#9A9080' : '#7A7268',
            grid: isDark ? 'rgba(232,226,213,0.05)' : 'rgba(31,27,22,0.05)',
            border: isDark ? 'rgba(232,226,213,0.08)' : 'rgba(31,27,22,0.08)',
            bg: isDark ? '#1A1814' : '#F5F2ED',
            bg2: isDark ? '#232019' : '#FAFAF7',
            bg3: isDark ? '#2E2A22' : '#EFEAE2'
        };
    },

    getEfficiencyClass(score) {
        if (score >= 0.8) return 'success';
        if (score >= 0.5) return 'warning';
        return 'danger';
    },

    getStatusColor(isCompleted) {
        return isCompleted ? '#7A8B6F' : '#6B8B95';
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
    Chart.defaults.font.family = "'JetBrains Mono', 'Fira Code', 'SF Mono', ui-monospace, monospace";
    Chart.defaults.font.size = 11;
    Chart.defaults.color = DashboardUtils.getChartColors().text2;
    Chart.defaults.borderColor = DashboardUtils.getChartColors().grid;
    Chart.defaults.plugins.legend.labels.boxWidth = 10;
    Chart.defaults.plugins.legend.labels.boxHeight = 10;
}
