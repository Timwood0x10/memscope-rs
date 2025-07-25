/**
 * Data Loader for MemScope Dashboard
 * Loads and processes JSON data from MemoryAnalysis endpoints
 */

class MemScopeDataLoader {
    constructor() {
        this.cache = new Map();
        this.loadingPromises = new Map();
    }

    /**
     * Load data from API endpoint or JSON files with caching
     */
    async loadData(endpoint) {
        if (this.cache.has(endpoint)) {
            return this.cache.get(endpoint);
        }

        if (this.loadingPromises.has(endpoint)) {
            return this.loadingPromises.get(endpoint);
        }

        const promise = this._fetchData(endpoint);
        this.loadingPromises.set(endpoint, promise);

        try {
            const data = await promise;
            this.cache.set(endpoint, data);
            this.loadingPromises.delete(endpoint);
            return data;
        } catch (error) {
            this.loadingPromises.delete(endpoint);
            throw error;
        }
    }

    /**
     * Load memory analysis data from multiple sources
     */
    async loadMemoryAnalysisData() {
        try {
            console.log('🔍 Loading memory analysis data...');
            
            // 首先尝试从 API 端点加载
            try {
                const apiData = await this.loadFromAPI();
                if (apiData) {
                    console.log('✅ Successfully loaded data from API');
                    return apiData;
                }
            } catch (error) {
                console.log('⚠️ API loading failed, trying JSON files:', error.message);
            }
            
            // 如果 API 失败，尝试直接从 JSON 文件加载
            try {
                const jsonData = await this.loadFromJSONFiles();
                if (jsonData) {
                    console.log('✅ Successfully loaded data from JSON files');
                    return jsonData;
                }
            } catch (error) {
                console.log('⚠️ JSON file loading failed:', error.message);
            }
            
            // 如果都失败，使用模拟数据
            console.log('⚠️ Falling back to mock data');
            return this.getDefaultData();
        } catch (error) {
            console.error('❌ Failed to load memory data:', error);
            return this.getDefaultData();
        }
    }

    /**
     * 从 API 端点加载数据
     */
    async loadFromAPI() {
        const endpoints = [
            '/api/overview',
            '/api/variables',
            '/api/timeline',
            '/api/unsafe-ffi',
            '/api/performance'
        ];
        
        let apiData = {};
        let hasData = false;
        
        for (const endpoint of endpoints) {
            try {
                const data = await this.loadData(endpoint);
                apiData[endpoint] = data;
                hasData = true;
            } catch (error) {
                console.log(`⚠️ Failed to load ${endpoint}:`, error.message);
            }
        }
        
        if (hasData) {
            return this.processAPIData(apiData);
        }
        
        return null;
    }

    /**
     * 从 MemoryAnalysis JSON 文件加载数据
     */
    async loadFromJSONFiles() {
        const dataFiles = [
            'MemoryAnalysis/snapshot_memory_analysis_memory_analysis.json',
            'MemoryAnalysis/snapshot_memory_analysis_lifetime.json',
            'MemoryAnalysis/snapshot_memory_analysis_security_violations.json',
            'MemoryAnalysis/snapshot_unsafe_ffi.json',
            'MemoryAnalysis/snapshot_memory_analysis_complex_types.json',
            'MemoryAnalysis/snapshot_memory_analysis_performance.json'
        ];
        
        let loadedData = {};
        let hasData = false;
        let availableFiles = [];
        
        for (const file of dataFiles) {
            try {
                const response = await fetch(file);
                if (response.ok) {
                    const data = await response.json();
                    const dataType = this.extractDataType(file);
                    loadedData[dataType] = data;
                    availableFiles.push(file);
                    hasData = true;
                    console.log(`✅ Loaded ${file}`);
                } else {
                    console.log(`⚠️ Could not load ${file}: ${response.status}`);
                }
            } catch (error) {
                console.log(`⚠️ Error loading ${file}:`, error.message);
            }
        }
        
        if (hasData) {
            this.updateDataSourceInfo(availableFiles);
            return this.processJSONData(loadedData);
        }
        
        return null;
    }

    /**
     * 从文件名提取数据类型
     */
    extractDataType(filename) {
        if (filename.includes('memory_analysis.json')) return 'memory_analysis';
        if (filename.includes('lifetime')) return 'lifetime';
        if (filename.includes('security_violations')) return 'security_violations';
        if (filename.includes('unsafe_ffi')) return 'unsafe_ffi';
        if (filename.includes('complex_types')) return 'complex_types';
        if (filename.includes('performance')) return 'performance';
        return 'unknown';
    }

    /**
     * 处理 API 数据
     */
    processAPIData(apiData) {
        // 转换 API 数据为仪表板格式
        const dashboardData = this.getDefaultData();
        
        if (apiData['/api/overview']) {
            const overview = apiData['/api/overview'];
            dashboardData.summary = {
                totalAllocations: overview.total_allocations || 0,
                activeAllocations: overview.active_allocations || 0,
                totalMemory: overview.total_memory || 0,
                peakMemory: overview.peak_memory || 0
            };
        }
        
        if (apiData['/api/variables']) {
            dashboardData.allocations = apiData['/api/variables'].slice(0, 50);
        }
        
        if (apiData['/api/unsafe-ffi']) {
            dashboardData.unsafeFFI = apiData['/api/unsafe-ffi'];
        }
        
        return dashboardData;
    }

    /**
     * 处理 JSON 文件数据
     */
    processJSONData(loadedData) {
        const dashboardData = this.getDefaultData();
        
        // 处理内存分析数据
        if (loadedData.memory_analysis && loadedData.memory_analysis.allocations) {
            const allocations = loadedData.memory_analysis.allocations;
            
            dashboardData.summary = {
                totalAllocations: allocations.length,
                activeAllocations: allocations.filter(a => !a.timestamp_dealloc).length,
                totalMemory: allocations.reduce((sum, a) => sum + (a.size || 0), 0),
                peakMemory: Math.max(...allocations.map(a => a.size || 0))
            };
            
            dashboardData.allocations = allocations.slice(0, 50).map(alloc => ({
                id: alloc.ptr || 'unknown',
                size: alloc.size || 0,
                type: alloc.type_name || 'Unknown',
                timestamp: alloc.timestamp_alloc || Date.now(),
                status: alloc.timestamp_dealloc ? 'deallocated' : 'active',
                scope: alloc.scope_name || 'global'
            }));
        }
        
        // 处理其他数据类型
        if (loadedData.unsafe_ffi) {
            dashboardData.unsafeFFI = loadedData.unsafe_ffi.unsafe_operations || [];
        }
        
        if (loadedData.complex_types) {
            dashboardData.complexTypes = Array.isArray(loadedData.complex_types) 
                ? loadedData.complex_types 
                : [loadedData.complex_types];
        }
        
        if (loadedData.lifetime) {
            dashboardData.lifecycle = loadedData.lifetime.variables || [];
        }
        
        return dashboardData;
    }

    /**
     * 更新数据源信息显示
     */
    updateDataSourceInfo(availableFiles) {
        const header = document.querySelector('header .container');
        if (header && availableFiles.length > 0) {
            const dataInfo = document.createElement('div');
            dataInfo.className = 'mt-4 text-sm text-gray-600';
            dataInfo.innerHTML = `
                <div class="bg-blue-50 border border-blue-200 rounded-lg p-3">
                    📁 Real data loaded from: ${availableFiles.length} files
                    <details class="mt-2">
                        <summary class="cursor-pointer font-semibold text-blue-700">View loaded files</summary>
                        <ul class="mt-2 ml-4 text-xs">
                            ${availableFiles.map(file => `<li class="text-gray-700">• ${file}</li>`).join('')}
                        </ul>
                    </details>
                </div>
            `;
            header.appendChild(dataInfo);
        }
    }

    async _fetchData(endpoint) {
        const response = await fetch(endpoint);
        if (!response.ok) {
            throw new Error(`Failed to load data from ${endpoint}: ${response.statusText}`);
        }
        return response.json();
    }

    /**
     * Process raw JSON data into dashboard format
     */
    processMemoryAnalysisData(rawData) {
        if (!Array.isArray(rawData)) {
            console.warn('Expected array data, got:', typeof rawData);
            return this.getDefaultData();
        }

        const stats = this.calculateMemoryStatistics(rawData);
        const allocations = this.processAllocations(rawData);
        const ffiData = this.processFfiData(rawData);
        const lifecycle = this.processLifecycleData(rawData);
        const security = this.processSecurityData(rawData);

        return {
            stats,
            allocations,
            ffiData,
            lifecycle,
            security,
            metadata: {
                totalEntries: rawData.length,
                processedAt: new Date().toISOString(),
                dataSource: 'MemoryAnalysis'
            }
        };
    }

    calculateMemoryStatistics(data) {
        let totalSize = 0;
        let activeAllocations = 0;
        let ffiAllocations = 0;
        let unsafeAllocations = 0;
        let peakMemory = 0;

        data.forEach(entry => {
            if (entry.base && entry.base.size) {
                totalSize += entry.base.size;
                activeAllocations++;
                
                if (entry.base.size > peakMemory) {
                    peakMemory = entry.base.size;
                }
            }

            if (entry.ffi_tracked) {
                ffiAllocations++;
            }

            if (entry.source && (entry.source.UnsafeRust || entry.source.FfiC)) {
                unsafeAllocations++;
            }
        });

        return {
            activeMemory: totalSize,
            activeAllocations,
            peakMemory,
            totalAllocations: activeAllocations,
            totalAllocated: totalSize,
            memoryEfficiency: activeAllocations > 0 ? (totalSize / (activeAllocations * peakMemory)) * 100 : 0,
            ffiAllocations,
            unsafeAllocations
        };
    }

    processAllocations(data) {
        return data.map((entry, index) => {
            const base = entry.base || {};
            const source = entry.source || {};
            
            return {
                id: index,
                ptr: base.ptr || 0,
                size: base.size || 0,
                varName: base.var_name || `allocation_${index}`,
                typeName: base.type_name || 'unknown',
                scopeName: base.scope_name || 'global',
                timestampAlloc: base.timestamp_alloc,
                timestampDealloc: base.timestamp_dealloc,
                isLeaked: base.is_leaked || false,
                lifetimeMs: base.lifetime_ms,
                borrowCount: base.borrow_count || 0,
                sourceType: this.getSourceType(source),
                riskLevel: this.getRiskLevel(entry),
                ffiTracked: entry.ffi_tracked || false
            };
        });
    }

    processFfiData(data) {
        const ffiEntries = data.filter(entry => entry.ffi_tracked || entry.source?.FfiC || entry.source?.UnsafeRust);
        
        return {
            totalFfiCalls: ffiEntries.length,
            libraries: this.extractLibraries(ffiEntries),
            crossBoundaryEvents: this.extractCrossBoundaryEvents(ffiEntries),
            safetyViolations: this.extractSafetyViolations(ffiEntries),
            riskAssessment: this.calculateRiskAssessment(ffiEntries)
        };
    }

    processLifecycleData(data) {
        const lifecycleEvents = [];
        
        data.forEach((entry, index) => {
            if (entry.base?.timestamp_alloc) {
                lifecycleEvents.push({
                    id: index,
                    type: 'allocation',
                    timestamp: entry.base.timestamp_alloc,
                    size: entry.base.size,
                    varName: entry.base.var_name || `var_${index}`,
                    sourceType: this.getSourceType(entry.source)
                });
            }
            
            if (entry.base?.timestamp_dealloc) {
                lifecycleEvents.push({
                    id: index,
                    type: 'deallocation',
                    timestamp: entry.base.timestamp_dealloc,
                    size: entry.base.size,
                    varName: entry.base.var_name || `var_${index}`,
                    sourceType: this.getSourceType(entry.source)
                });
            }
        });

        return {
            events: lifecycleEvents.sort((a, b) => a.timestamp - b.timestamp),
            totalLifetime: this.calculateTotalLifetime(lifecycleEvents),
            averageLifetime: this.calculateAverageLifetime(data)
        };
    }

    processSecurityData(data) {
        const violations = [];
        let riskScore = 0;
        
        data.forEach((entry, index) => {
            if (entry.safety_violations && entry.safety_violations.length > 0) {
                violations.push(...entry.safety_violations.map(v => ({
                    ...v,
                    entryId: index,
                    source: this.getSourceType(entry.source)
                })));
            }

            // Calculate risk based on source type
            if (entry.source?.UnsafeRust) {
                riskScore += entry.source.UnsafeRust.risk_assessment?.confidence_score || 0.5;
            }
            if (entry.source?.FfiC) {
                riskScore += 0.7; // FFI calls are inherently risky
            }
        });

        return {
            violations,
            riskScore: data.length > 0 ? riskScore / data.length : 0,
            unsafeBlocks: data.filter(e => e.source?.UnsafeRust).length,
            ffiCalls: data.filter(e => e.source?.FfiC).length
        };
    }

    // Helper methods
    getSourceType(source) {
        if (!source) return 'unknown';
        if (source.FfiC) return 'ffi_c';
        if (source.UnsafeRust) return 'unsafe_rust';
        return 'safe_rust';
    }

    getRiskLevel(entry) {
        if (entry.source?.UnsafeRust?.risk_assessment) {
            return entry.source.UnsafeRust.risk_assessment.risk_level || 'Low';
        }
        if (entry.source?.FfiC) return 'Medium';
        return 'Low';
    }

    extractLibraries(ffiEntries) {
        const libraries = new Set();
        ffiEntries.forEach(entry => {
            if (entry.source?.FfiC?.library_name) {
                libraries.add(entry.source.FfiC.library_name);
            }
        });
        return Array.from(libraries);
    }

    extractCrossBoundaryEvents(ffiEntries) {
        const events = [];
        ffiEntries.forEach(entry => {
            if (entry.cross_boundary_events) {
                events.push(...entry.cross_boundary_events);
            }
        });
        return events;
    }

    extractSafetyViolations(ffiEntries) {
        const violations = [];
        ffiEntries.forEach(entry => {
            if (entry.safety_violations) {
                violations.push(...entry.safety_violations);
            }
        });
        return violations;
    }

    calculateRiskAssessment(ffiEntries) {
        let totalRisk = 0;
        let riskCount = 0;

        ffiEntries.forEach(entry => {
            if (entry.source?.UnsafeRust?.risk_assessment) {
                const assessment = entry.source.UnsafeRust.risk_assessment;
                const riskValue = this.riskLevelToNumber(assessment.risk_level);
                totalRisk += riskValue * (assessment.confidence_score || 0.5);
                riskCount++;
            }
            if (entry.source?.FfiC) {
                totalRisk += 2; // Medium risk for FFI
                riskCount++;
            }
        });

        return riskCount > 0 ? totalRisk / riskCount : 0;
    }

    riskLevelToNumber(level) {
        switch (level?.toLowerCase()) {
            case 'low': return 1;
            case 'medium': return 2;
            case 'high': return 3;
            case 'critical': return 4;
            default: return 1;
        }
    }

    calculateTotalLifetime(events) {
        // Calculate total lifetime from allocation/deallocation pairs
        const allocations = new Map();
        let totalLifetime = 0;

        events.forEach(event => {
            if (event.type === 'allocation') {
                allocations.set(event.id, event.timestamp);
            } else if (event.type === 'deallocation' && allocations.has(event.id)) {
                const allocTime = allocations.get(event.id);
                totalLifetime += event.timestamp - allocTime;
                allocations.delete(event.id);
            }
        });

        return totalLifetime;
    }

    calculateAverageLifetime(data) {
        const lifetimes = data
            .map(entry => entry.base?.lifetime_ms)
            .filter(lt => lt !== null && lt !== undefined);
        
        return lifetimes.length > 0 
            ? lifetimes.reduce((sum, lt) => sum + lt, 0) / lifetimes.length 
            : 0;
    }

    getDefaultData() {
        return {
            stats: {
                activeMemory: 0,
                activeAllocations: 0,
                peakMemory: 0,
                totalAllocations: 0,
                totalAllocated: 0,
                memoryEfficiency: 0,
                ffiAllocations: 0,
                unsafeAllocations: 0
            },
            allocations: [],
            ffiData: {
                totalFfiCalls: 0,
                libraries: [],
                crossBoundaryEvents: [],
                safetyViolations: [],
                riskAssessment: 0
            },
            lifecycle: {
                events: [],
                totalLifetime: 0,
                averageLifetime: 0
            },
            security: {
                violations: [],
                riskScore: 0,
                unsafeBlocks: 0,
                ffiCalls: 0
            },
            metadata: {
                totalEntries: 0,
                processedAt: new Date().toISOString(),
                dataSource: 'default'
            }
        };
    }

    /**
     * Load all available data sources
     */
    async loadAllData() {
        const endpoints = [
            '/api/data/unsafe_ffi',
            '/api/data/memory_analysis',
            '/api/data/security_violations',
            '/api/data/lifetime'
        ];

        const results = {};
        
        for (const endpoint of endpoints) {
            try {
                const data = await this.loadData(endpoint);
                const processedData = this.processMemoryAnalysisData(data);
                const key = endpoint.split('/').pop();
                results[key] = processedData;
            } catch (error) {
                console.warn(`Failed to load data from ${endpoint}:`, error);
                results[endpoint.split('/').pop()] = this.getDefaultData();
            }
        }

        return this.mergeDataSources(results);
    }

    /**
     * Merge data from multiple sources
     */
    mergeDataSources(sources) {
        const merged = this.getDefaultData();
        
        Object.values(sources).forEach(source => {
            // Merge statistics
            merged.stats.activeMemory += source.stats.activeMemory;
            merged.stats.activeAllocations += source.stats.activeAllocations;
            merged.stats.peakMemory = Math.max(merged.stats.peakMemory, source.stats.peakMemory);
            merged.stats.totalAllocations += source.stats.totalAllocations;
            merged.stats.totalAllocated += source.stats.totalAllocated;
            merged.stats.ffiAllocations += source.stats.ffiAllocations || 0;
            merged.stats.unsafeAllocations += source.stats.unsafeAllocations || 0;

            // Merge allocations
            merged.allocations.push(...source.allocations);

            // Merge FFI data
            merged.ffiData.totalFfiCalls += source.ffiData.totalFfiCalls;
            merged.ffiData.libraries.push(...source.ffiData.libraries);
            merged.ffiData.crossBoundaryEvents.push(...source.ffiData.crossBoundaryEvents);
            merged.ffiData.safetyViolations.push(...source.ffiData.safetyViolations);

            // Merge lifecycle events
            merged.lifecycle.events.push(...source.lifecycle.events);
            merged.lifecycle.totalLifetime += source.lifecycle.totalLifetime;

            // Merge security data
            merged.security.violations.push(...source.security.violations);
            merged.security.unsafeBlocks += source.security.unsafeBlocks;
            merged.security.ffiCalls += source.security.ffiCalls;
        });

        // Recalculate derived values
        merged.stats.memoryEfficiency = merged.stats.activeAllocations > 0 
            ? (merged.stats.activeMemory / (merged.stats.activeAllocations * merged.stats.peakMemory)) * 100 
            : 0;

        merged.lifecycle.averageLifetime = merged.lifecycle.events.length > 0
            ? merged.lifecycle.totalLifetime / merged.lifecycle.events.length
            : 0;

        merged.security.riskScore = (merged.security.unsafeBlocks + merged.security.ffiCalls) / 
            Math.max(merged.stats.totalAllocations, 1);

        // Remove duplicates
        merged.ffiData.libraries = [...new Set(merged.ffiData.libraries)];
        merged.lifecycle.events.sort((a, b) => a.timestamp - b.timestamp);

        return merged;
    }
}

// Global instance
window.memScopeDataLoader = new MemScopeDataLoader();