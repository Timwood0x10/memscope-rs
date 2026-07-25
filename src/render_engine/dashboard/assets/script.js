// Shared utility functions
window.MemScopeUtils = {
    formatBytes: function(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    },

    formatTimestamp: function(timestamp) {
        const date = new Date(timestamp);
        return date.toLocaleString();
    },

    createTooltip: function(element, content) {
        const tooltip = document.createElement('div');
        tooltip.className = 'tooltip';
        tooltip.textContent = content;
        tooltip.style.cssText = `
            position: absolute;
            background: rgba(0, 0, 0, 0.8);
            color: white;
            padding: 8px 12px;
            border-radius: 4px;
            font-size: 12px;
            pointer-events: none;
            z-index: 1000;
        `;
        document.body.appendChild(tooltip);

        element.addEventListener('mouseenter', (e) => {
            tooltip.style.display = 'block';
            tooltip.style.left = e.pageX + 10 + 'px';
            tooltip.style.top = e.pageY + 10 + 'px';
        });

        element.addEventListener('mousemove', (e) => {
            tooltip.style.left = e.pageX + 10 + 'px';
            tooltip.style.top = e.pageY + 10 + 'px';
        });

        element.addEventListener('mouseleave', () => {
            tooltip.style.display = 'none';
        });
    }
};

// Chart.js default configuration
Chart.defaults.font.family = "'JetBrains Mono', 'Fira Code', 'SF Mono', ui-monospace, monospace";
Chart.defaults.font.size = 11;
Chart.defaults.color = '#7A7268';
Chart.defaults.borderColor = 'rgba(31,27,22,0.05)';

console.log('MemScope Dashboard utilities loaded');