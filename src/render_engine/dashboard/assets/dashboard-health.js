// Health Dashboard Module
window.DashboardHealth = {
    updateQuickHealth(data) {
        const asyncTasks = data.async_tasks || [];
        const activeCount = asyncTasks.filter(t => !t.is_completed).length;
        const leakCount = data.leak_count || 0;
        const unsafeCount = data.unsafe_count || 0;
        
        // Update active tasks
        const activeTaskEl = document.getElementById('quickActiveTasks');
        if (activeTaskEl) {
            activeTaskEl.textContent = activeCount;
        }
        
        // Update status badge
        const statusBadge = document.getElementById('quickStatusBadge');
        if (statusBadge) {
            if (leakCount > 0) {
                statusBadge.className = 'badge badge-danger';
                statusBadge.textContent = `⚠️ ${leakCount} Leak${leakCount > 1 ? 's' : ''} Detected`;
            } else if (unsafeCount > 0) {
                statusBadge.className = 'badge badge-warning';
                statusBadge.textContent = `⚡ ${unsafeCount} Unsafe Operation${unsafeCount > 1 ? 's' : ''}`;
            } else {
                statusBadge.className = 'badge badge-success';
                statusBadge.textContent = '✅ Healthy';
            }
        }
        
        // Update health trend
        const healthScore = parseInt(data.health_score) || 0;
        const healthTrend = document.getElementById('healthTrend');
        if (healthTrend) {
            if (healthScore >= 80) {
                healthTrend.textContent = '✅';
            } else if (healthScore >= 60) {
                healthTrend.textContent = '⚠️';
            } else {
                healthTrend.textContent = '🚨';
            }
        }
    },
    
    updateCriticalAlerts(data) {
        const alertsContainer = document.getElementById('criticalAlerts');
        if (!alertsContainer) return;
        
        const alerts = [];
        const leakCount = data.leak_count || 0;
        const unsafeCount = data.unsafe_count || 0;
        const ffiCount = data.ffi_count || 0;
        const asyncTasks = data.async_tasks || [];
        const leakyTasks = asyncTasks.filter(t => t.has_potential_leak).length;
        
        if (leakCount > 0) {
            alerts.push({
                type: 'danger',
                icon: '🔴',
                message: `<strong>${leakCount} Memory Leak${leakCount > 1 ? 's' : ''}</strong> detected - Immediate attention required`,
                action: `onclick="showMode('passport')" style="cursor: pointer;"`
            });
        }
        
        if (leakyTasks > 0) {
            alerts.push({
                type: 'warning',
                icon: '⚠️',
                message: `<strong>${leakyTasks} Async Task${leakyTasks > 1 ? 's' : ''}</strong> with potential memory leaks`,
                action: `onclick="showMode('task')" style="cursor: pointer;"`
            });
        }
        
        if (unsafeCount > 10) {
            alerts.push({
                type: 'warning',
                icon: '⚡',
                message: `<strong>${unsafeCount} Unsafe Operations</strong> - Consider reviewing for safety`,
                action: `onclick="showMode('unsafe')" style="cursor: pointer;"`
            });
        }
        
        if (ffiCount > 20) {
            alerts.push({
                type: 'info',
                icon: '🔗',
                message: `<strong>${ffiCount} FFI Calls</strong> - High cross-boundary activity`,
                action: `onclick="showMode('unsafe')" style="cursor: pointer;"`
            });
        }
        
        if (alerts.length === 0) {
            alertsContainer.innerHTML = `
                <div style="background: rgba(16, 185, 129, 0.1); border-left: 4px solid #10b981; padding: 12px; border-radius: 4px;">
                    <span style="font-size: 1.2rem;">✅</span>
                    <span style="margin-left: 8px; font-weight: 600; color: #10b981;">All systems healthy - No critical issues detected</span>
                </div>
            `;
        } else {
            alertsContainer.innerHTML = alerts.map(alert => `
                <div class="alert alert-${alert.type}" style="background: rgba(${alert.type === 'danger' ? '239, 68, 68' : alert.type === 'warning' ? '245, 158, 11' : '59, 130, 246'}, 0.1); border-left: 4px solid ${alert.type === 'danger' ? '#ef4444' : alert.type === 'warning' ? '#f59e0b' : '#3b82f6'}; padding: 12px; border-radius: 4px; margin-bottom: 8px;" ${alert.action}>
                    <span style="font-size: 1.2rem;">${alert.icon}</span>
                    <span style="margin-left: 8px;">${alert.message}</span>
                </div>
            `).join('');
        }
    },
    
    init(data) {
        this.updateQuickHealth(data);
        this.updateCriticalAlerts(data);
    }
};
