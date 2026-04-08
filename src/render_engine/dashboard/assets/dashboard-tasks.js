// Async Tasks Module
window.DashboardTasks = {
    filteredData: [],
    currentView: 'timeline',
    
    toggleView(view) {
        this.currentView = view;
        this.renderTimeline();
    },
    
    filter(filterBy, asyncTasks) {
        switch(filterBy) {
            case 'active':
                this.filteredData = asyncTasks.filter(t => !t.is_completed);
                break;
            case 'completed':
                this.filteredData = asyncTasks.filter(t => t.is_completed);
                break;
            case 'leak':
                this.filteredData = asyncTasks.filter(t => t.has_potential_leak);
                break;
            default:
                this.filteredData = [...asyncTasks];
        }
        
        this.render();
        this.renderTimeline();
    },
    
    sort(sortBy) {
        this.filteredData.sort((a, b) => {
            switch (sortBy) {
                case 'memory-desc': return (b.current_memory || 0) - (a.current_memory || 0);
                case 'memory-asc': return (a.current_memory || 0) - (b.current_memory || 0);
                case 'peak-desc': return (b.peak_memory || 0) - (a.peak_memory || 0);
                case 'efficiency-desc': return (b.efficiency_score || 0) - (a.efficiency_score || 0);
                case 'duration-desc': return (b.duration_ms || 0) - (a.duration_ms || 0);
                case 'allocations-desc': return (b.total_allocations || 0) - (a.total_allocations || 0);
                default: return 0;
            }
        });
        this.render();
    },
    
    render(containerId = 'taskCardsContainer') {
        const container = document.getElementById(containerId);
        if (!container) return;
        
        if (this.filteredData.length === 0) {
            container.innerHTML = '<div class="empty-state">No async task data</div>';
            return;
        }
        
        container.innerHTML = this.filteredData.map(t => {
            const statusClass = t.is_completed ? 'success' : 'info';
            const statusText = t.is_completed ? 'Completed' : 'Running';
            const leakBadge = t.has_potential_leak ? '<span class="badge badge-warning">⚠️ Leak?</span>' : '';
            const efficiencyClass = DashboardUtils.getEfficiencyClass(t.efficiency_score || 0);
            const taskType = t.task_type || 'Unknown';
            const statusColor = DashboardUtils.getStatusColor(t.is_completed);
            
            return `
                <div class="card" style="padding: 12px; margin: 0; border-left: 4px solid ${statusColor};">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                        <div>
                            <span style="font-weight: 600; color: var(--primary);">#${t.task_id}</span>
                            <span style="margin-left: 8px; font-size: 0.9rem;">${t.task_name || 'Unknown'}</span>
                        </div>
                        <div>
                            <span class="badge badge-${statusClass}">${statusText}</span>
                            ${leakBadge}
                        </div>
                    </div>
                    
                    <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; margin-bottom: 8px; font-size: 0.85rem;">
                        <div>
                            <div style="color: var(--text2); font-size: 0.75rem;">💾 Current Memory</div>
                            <div style="font-weight: 600;">${DashboardUtils.formatBytes(t.current_memory || 0)}</div>
                        </div>
                        <div>
                            <div style="color: var(--text2); font-size: 0.75rem;">🏔️ Peak Memory</div>
                            <div style="font-weight: 600;">${DashboardUtils.formatBytes(t.peak_memory || 0)}</div>
                        </div>
                        <div>
                            <div style="color: var(--text2); font-size: 0.75rem;">📊 Allocations</div>
                            <div style="font-weight: 600;">${t.total_allocations || 0}</div>
                        </div>
                    </div>
                    
                    <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; font-size: 0.85rem;">
                        <div>
                            <div style="color: var(--text2); font-size: 0.75rem;">⏱️ Duration</div>
                            <div style="font-weight: 600;">${DashboardUtils.formatDuration(t.duration_ms || 0)}</div>
                        </div>
                        <div>
                            <div style="color: var(--text2); font-size: 0.75rem;">📈 Efficiency</div>
                            <div class="badge badge-${efficiencyClass}">${((t.efficiency_score || 0) * 100).toFixed(0)}%</div>
                        </div>
                        <div>
                            <div style="color: var(--text2); font-size: 0.75rem;">🏷️ Type</div>
                            <div style="font-weight: 600; font-size: 0.75rem;">${taskType}</div>
                        </div>
                    </div>
                </div>
            `;
        }).join('');
    },
    
    renderTimeline(containerId = 'taskTimeline') {
        const container = document.getElementById(containerId);
        if (!container) return;
        
        const tasks = this.filteredData;
        if (tasks.length === 0) {
            container.innerHTML = '<div class="empty-state">No task data for timeline</div>';
            return;
        }
        
        if (this.currentView === 'gantt') {
            this.renderGanttChart(container, tasks);
        } else {
            this.renderTimelineView(container, tasks);
        }
    },
    
    renderTimelineView(container, tasks) {
        const maxDuration = Math.max(...tasks.map(t => t.duration_ms || 0));
        
        container.innerHTML = `
            <div style="display: flex; flex-direction: column; gap: 8px;">
                ${tasks.slice(0, 15).map((t, i) => {
                    const width = maxDuration > 0 ? ((t.duration_ms || 0) / maxDuration * 100) : 0;
                    const statusColor = DashboardUtils.getStatusColor(t.is_completed);
                    const leakIndicator = t.has_potential_leak ? '⚠️' : '';
                    
                    return `
                        <div style="display: flex; align-items: center; gap: 12px;">
                            <div style="width: 120px; font-size: 0.85rem; font-weight: 600; color: var(--primary);">
                                #${t.task_id} ${leakIndicator}
                            </div>
                            <div style="flex: 1; height: 24px; background: var(--bg3); border-radius: 4px; overflow: hidden; position: relative;">
                                <div style="height: 100%; width: ${width}%; background: ${statusColor}; border-radius: 4px; transition: width 0.3s;"></div>
                                <div style="position: absolute; top: 50%; left: 8px; transform: translateY(-50%); font-size: 0.75rem; color: white; font-weight: 600; text-shadow: 0 1px 2px rgba(0,0,0,0.5);">
                                    ${t.task_name || 'Unknown'} (${DashboardUtils.formatDuration(t.duration_ms || 0)})
                                </div>
                            </div>
                            <div style="width: 80px; text-align: right; font-size: 0.75rem; color: var(--text2);">
                                ${DashboardUtils.formatBytes(t.peak_memory || 0)}
                            </div>
                        </div>
                    `;
                }).join('')}
            </div>
        `;
    },
    
    renderGanttChart(container, tasks) {
        const sortedTasks = [...tasks].sort((a, b) => (a.created_at_ms || 0) - (b.created_at_ms || 0));
        const minTime = Math.min(...sortedTasks.map(t => t.created_at_ms || 0));
        const maxTime = Math.max(...sortedTasks.map(t => (t.created_at_ms || 0) + (t.duration_ms || 0)));
        const timeRange = maxTime - minTime;
        
        container.innerHTML = `
            <div style="display: flex; flex-direction: column; gap: 6px;">
                ${sortedTasks.slice(0, 15).map((t, i) => {
                    const start = ((t.created_at_ms || 0) - minTime) / timeRange * 100;
                    const width = (t.duration_ms || 0) / timeRange * 100;
                    const statusColor = DashboardUtils.getStatusColor(t.is_completed);
                    const leakIndicator = t.has_potential_leak ? 'border: 2px solid #f59e0b;' : '';
                    
                    return `
                        <div style="display: flex; align-items: center; gap: 12px; height: 28px;">
                            <div style="width: 100px; font-size: 0.8rem; font-weight: 600; color: var(--primary);">
                                #${t.task_id}
                            </div>
                            <div style="flex: 1; height: 20px; background: var(--bg3); border-radius: 4px; position: relative;">
                                <div style="position: absolute; left: ${start}%; width: ${Math.max(width, 2)}%; height: 100%; background: ${statusColor}; border-radius: 4px; ${leakIndicator} display: flex; align-items: center; padding: 0 8px;">
                                    <span style="font-size: 0.7rem; color: white; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">
                                        ${t.task_name || 'Task'}
                                    </span>
                                </div>
                            </div>
                        </div>
                    `;
                }).join('')}
            </div>
        `;
    },
    
    initCharts(tasks) {
        const colors = DashboardUtils.getChartColors();
        const taskData = tasks.slice(0, 10);
        
        // Duration Chart
        const durationCtx = document.getElementById('asyncDurationChart');
        if (durationCtx) {
            new Chart(durationCtx, {
                type: 'bar',
                data: {
                    labels: taskData.map(t => t.task_name || `Task ${t.task_id}`),
                    datasets: [{
                        label: 'Duration (ms)',
                        data: taskData.map(t => t.duration_ms || 0),
                        backgroundColor: colors.primary + '80',
                        borderColor: colors.primary,
                        borderWidth: 1
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {
                        y: { beginAtZero: true, grid: { color: colors.grid }, ticks: { color: colors.text2 } },
                        x: { grid: { display: false }, ticks: { color: colors.text2, maxRotation: 45 } }
                    },
                    plugins: { legend: { labels: { color: colors.text } } }
                }
            });
        }
        
        // Efficiency Chart
        const efficiencyCtx = document.getElementById('asyncEfficiencyChart');
        if (efficiencyCtx) {
            new Chart(efficiencyCtx, {
                type: 'doughnut',
                data: {
                    labels: taskData.map(t => `#${t.task_id}`),
                    datasets: [{
                        data: taskData.map(t => (t.efficiency_score || 0) * 100),
                        backgroundColor: taskData.map(t => 
                            (t.efficiency_score || 0) >= 0.8 ? '#10b981' : 
                            (t.efficiency_score || 0) >= 0.5 ? '#f59e0b' : '#ef4444'
                        )
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: { 
                        legend: { position: 'right', labels: { color: colors.text, font: { size: 10 } } }
                    }
                }
            });
        }
        
        // Heatmap Chart
        const heatmapCtx = document.getElementById('asyncHeatmapChart');
        if (heatmapCtx) {
            const taskLabels = taskData.map(t => t.task_name || `Task ${t.task_id}`);
            new Chart(heatmapCtx, {
                type: 'bar',
                data: {
                    labels: taskLabels,
                    datasets: [
                        {
                            label: 'Memory %',
                            data: taskData.map(t => Math.min(100, ((t.peak_memory || 0) / 1024 / 1024) * 10)),
                            backgroundColor: '#3b82f680',
                            borderColor: '#3b82f6',
                            borderWidth: 1
                        },
                        {
                            label: 'CPU %',
                            data: taskData.map(t => Math.min(100, (t.total_allocations || 0) * 5)),
                            backgroundColor: '#10b98180',
                            borderColor: '#10b981',
                            borderWidth: 1
                        },
                        {
                            label: 'IO %',
                            data: taskData.map(t => Math.min(100, ((t.total_bytes || 0) / 1024) * 0.5)),
                            backgroundColor: '#f59e0b80',
                            borderColor: '#f59e0b',
                            borderWidth: 1
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {
                        y: { 
                            beginAtZero: true,
                            max: 100,
                            grid: { color: colors.grid }, 
                            ticks: { color: colors.text2, callback: v => v + '%' }
                        },
                        x: { grid: { display: false }, ticks: { color: colors.text2, maxRotation: 45 } }
                    },
                    plugins: { legend: { labels: { color: colors.text } } }
                }
            });
        }
    }
};
