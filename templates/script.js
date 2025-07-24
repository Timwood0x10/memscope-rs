// MemScope-RS Dynamic Interactive Visualizations
// Enhanced JSON data loading and processing system

/**
 * 统一JSON数据读取器 - 支持5个数据源的并行加载
 */
class JSONDataLoader {
    constructor(basePath = './examples/MemoryAnalysis/') {
        this.basePath = basePath;
        this.dataSources = {
            memory_analysis: 'snapshot_memory_analysis.json',
            performance: 'snapshot_performance.json', 
            security_violations: 'snapshot_security_violations.json',
            unsafe_ffi: 'snapshot_unsafe_ffi.json',
            complex_types: 'snapshot_memory_analysis_complex_types.json'
        };
        this.loadedData = {};
        this.loadProgress = {};
        this.errorHandlers = [];
        this.progressHandlers = [];
        this.cacheManager = new BrowserCacheManager();
    }

    /**
     * 并行加载所有JSON数据文件
     */
    async loadAllData() {
        console.log('🔄 开始加载JSON数据文件...');
        const startTime = performance.now();
        
        try {
            // 并行加载所有数据源
            const loadPromises = Object.entries(this.dataSources).map(([key, fileName]) => 
                this.loadSingleDataSource(key, fileName)
            );
            
            const results = await Promise.allSettled(loadPromises);
            
            // 处理加载结果
            results.forEach((result, index) => {
                const [key] = Object.entries(this.dataSources)[index];
                if (result.status === 'fulfilled') {
                    this.loadedData[key] = result.value;
                    console.log(`✅ ${key} 数据加载成功`);
                } else {
                    console.warn(`⚠️ ${key} 数据加载失败:`, result.reason);
                    this.loadedData[key] = this.getDefaultData(key);
                }
            });
            
            const loadTime = performance.now() - startTime;
            console.log(`🎉 数据加载完成，耗时: ${loadTime.toFixed(2)}ms`);
            
            // 标准化和合并数据
            return this.normalizeAndMergeData();
            
        } catch (error) {
            console.error('❌ 数据加载失败:', error);
            throw new Error(`数据加载失败: ${error.message}`);
        }
    }

    /**
     * 加载单个数据源（带缓存支持）
     */
    async loadSingleDataSource(key, fileName) {
        const cacheKey = `${key}_${fileName}`;
        this.updateProgress(key, 0);
        
        try {
            // 1. 尝试从缓存获取
            const cachedData = this.cacheManager.get(cacheKey);
            if (cachedData) {
                console.log(`📦 使用缓存数据: ${key}`);
                this.updateProgress(key, 100);
                return cachedData;
            }
            
            // 2. 从网络加载
            console.log(`🌐 从网络加载: ${fileName}`);
            const url = `${this.basePath}${fileName}`;
            
            const response = await fetch(url);
            
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }
            
            this.updateProgress(key, 50);
            
            const data = await response.json();
            this.updateProgress(key, 75);
            
            // 3. 数据验证
            this.validateDataSource(key, data);
            
            // 4. 缓存数据
            this.cacheManager.set(cacheKey, data, true); // 持久化缓存
            
            this.updateProgress(key, 100);
            return data;
            
        } catch (error) {
            this.updateProgress(key, -1); // 错误状态
            console.error(`❌ 加载 ${fileName} 失败:`, error);
            
            // 尝试使用过期的缓存数据作为回退
            const expiredCache = this.tryGetExpiredCache(cacheKey);
            if (expiredCache) {
                console.warn(`⚠️ 使用过期缓存数据: ${key}`);
                this.updateProgress(key, 50); // 部分成功状态
                return expiredCache;
            }
            
            throw new Error(`加载 ${fileName} 失败: ${error.message}`);
        }
    }

    /**
     * 尝试获取过期的缓存数据
     */
    tryGetExpiredCache(cacheKey) {
        try {
            const localKey = `memscope_persistent_${this.cacheManager.cacheVersion}_${cacheKey}`;
            const cached = this.cacheManager.localStorage.getItem(localKey);
            if (cached) {
                const { data } = JSON.parse(cached);
                return data;
            }
        } catch (e) {
            console.warn('无法获取过期缓存:', e);
        }
        return null;
    }

    /**
     * 数据验证
     */
    validateDataSource(key, data) {
        if (!data || typeof data !== 'object') {
            throw new Error(`${key} 数据格式无效`);
        }
        
        // 根据数据源类型进行特定验证
        switch (key) {
            case 'memory_analysis':
                if (!Array.isArray(data.allocations)) {
                    throw new Error('memory_analysis 缺少 allocations 数组');
                }
                break;
            case 'performance':
                if (!data.performance_metrics) {
                    throw new Error('performance 缺少 performance_metrics');
                }
                break;
            case 'security_violations':
                if (!Array.isArray(data.security_violations)) {
                    throw new Error('security_violations 缺少 security_violations 数组');
                }
                break;
            case 'unsafe_ffi':
                if (!Array.isArray(data)) {
                    throw new Error('unsafe_ffi 应该是数组格式');
                }
                break;
            case 'complex_types':
                if (!data.categorized_types) {
                    throw new Error('complex_types 缺少 categorized_types');
                }
                break;
        }
    }

    /**
     * 获取默认数据（当文件加载失败时）
     */
    getDefaultData(key) {
        const defaults = {
            memory_analysis: { allocations: [] },
            performance: { performance_metrics: { active_allocations: 0, active_memory: 0, allocations: [] } },
            security_violations: { security_violations: [], timestamp: Date.now() },
            unsafe_ffi: [],
            complex_types: { 
                categorized_types: { collections: [], generic_types: [], smart_pointers: [], trait_objects: [] },
                summary: { total_complex_types: 0 }
            }
        };
        return defaults[key] || {};
    }

    /**
     * 标准化和合并数据
     */
    normalizeAndMergeData() {
        console.log('🔄 开始数据标准化和合并...');
        
        const normalizer = new DataNormalizer();
        
        // 标准化各个数据源
        const normalizedData = {
            allocations: normalizer.normalizeAllocations(this.loadedData),
            performance: normalizer.normalizePerformance(this.loadedData.performance),
            security: normalizer.normalizeSecurity(this.loadedData.security_violations),
            unsafeFFI: normalizer.normalizeUnsafeFFI(this.loadedData.unsafe_ffi),
            complexTypes: normalizer.normalizeComplexTypes(this.loadedData.complex_types),
            metadata: this.generateMetadata()
        };
        
        // 建立数据关联
        normalizedData.relationships = normalizer.buildDataRelationships(normalizedData);
        
        console.log('✅ 数据标准化完成');
        return normalizedData;
    }

    /**
     * 生成元数据
     */
    generateMetadata() {
        return {
            timestamp: Date.now(),
            version: '2.0',
            sources: Object.keys(this.dataSources),
            loadStatus: this.loadProgress,
            totalAllocations: this.getTotalAllocations()
        };
    }

    /**
     * 获取总分配数量
     */
    getTotalAllocations() {
        let total = 0;
        if (this.loadedData.memory_analysis?.allocations) {
            total += this.loadedData.memory_analysis.allocations.length;
        }
        if (this.loadedData.performance?.performance_metrics?.allocations) {
            total += this.loadedData.performance.performance_metrics.allocations.length;
        }
        return total;
    }

    /**
     * 更新加载进度
     */
    updateProgress(key, progress) {
        this.loadProgress[key] = progress;
        this.progressHandlers.forEach(handler => handler(key, progress));
    }

    /**
     * 注册进度回调
     */
    onProgress(callback) {
        this.progressHandlers.push(callback);
    }

    /**
     * 注册错误回调
     */
    onError(callback) {
        this.errorHandlers.push(callback);
    }
}

/**
 * 浏览器缓存管理器 - 优化数据加载性能
 */
class BrowserCacheManager {
    constructor() {
        this.memoryCache = new Map();
        this.maxMemorySize = 50; // 内存缓存最大条目数
        this.cacheVersion = '2.0';
        this.sessionStorage = window.sessionStorage;
        this.localStorage = window.localStorage;
    }

    /**
     * 获取缓存数据
     */
    get(key) {
        // 1. 优先从内存缓存获取
        if (this.memoryCache.has(key)) {
            console.log(`🎯 内存缓存命中: ${key}`);
            return this.memoryCache.get(key);
        }

        // 2. 从sessionStorage获取
        try {
            const sessionKey = `memscope_${this.cacheVersion}_${key}`;
            const cached = this.sessionStorage.getItem(sessionKey);
            if (cached) {
                const data = JSON.parse(cached);
                // 回填内存缓存
                this.setMemoryCache(key, data);
                console.log(`💾 会话缓存命中: ${key}`);
                return data;
            }
        } catch (e) {
            console.warn(`会话缓存读取失败 ${key}:`, e);
        }

        // 3. 从localStorage获取（持久化缓存）
        try {
            const localKey = `memscope_persistent_${this.cacheVersion}_${key}`;
            const cached = this.localStorage.getItem(localKey);
            if (cached) {
                const { data, timestamp } = JSON.parse(cached);
                // 检查是否过期（24小时）
                if (Date.now() - timestamp < 24 * 60 * 60 * 1000) {
                    this.setMemoryCache(key, data);
                    console.log(`💿 本地缓存命中: ${key}`);
                    return data;
                } else {
                    this.localStorage.removeItem(localKey);
                    console.log(`🗑️ 本地缓存已过期: ${key}`);
                }
            }
        } catch (e) {
            console.warn(`本地缓存读取失败 ${key}:`, e);
        }

        return null;
    }

    /**
     * 设置缓存数据
     */
    set(key, data, persistent = false) {
        // 1. 设置内存缓存
        this.setMemoryCache(key, data);

        // 2. 设置会话缓存
        try {
            const sessionKey = `memscope_${this.cacheVersion}_${key}`;
            this.sessionStorage.setItem(sessionKey, JSON.stringify(data));
        } catch (e) {
            console.warn(`会话缓存设置失败 ${key}:`, e);
        }

        // 3. 设置持久化缓存（可选）
        if (persistent) {
            try {
                const localKey = `memscope_persistent_${this.cacheVersion}_${key}`;
                const cacheData = {
                    data,
                    timestamp: Date.now()
                };
                this.localStorage.setItem(localKey, JSON.stringify(cacheData));
            } catch (e) {
                console.warn(`本地缓存设置失败 ${key}:`, e);
            }
        }
    }

    /**
     * 设置内存缓存
     */
    setMemoryCache(key, data) {
        // LRU淘汰策略
        if (this.memoryCache.size >= this.maxMemorySize) {
            const firstKey = this.memoryCache.keys().next().value;
            this.memoryCache.delete(firstKey);
        }
        this.memoryCache.set(key, data);
    }

    /**
     * 清除所有缓存
     */
    clear() {
        this.memoryCache.clear();
        
        // 清除会话缓存
        Object.keys(this.sessionStorage).forEach(key => {
            if (key.startsWith(`memscope_${this.cacheVersion}_`)) {
                this.sessionStorage.removeItem(key);
            }
        });

        // 清除本地缓存
        Object.keys(this.localStorage).forEach(key => {
            if (key.startsWith(`memscope_persistent_${this.cacheVersion}_`)) {
                this.localStorage.removeItem(key);
            }
        });

        console.log('🧹 所有缓存已清除');
    }

    /**
     * 获取缓存统计信息
     */
    getStats() {
        return {
            memoryCache: this.memoryCache.size,
            sessionStorage: Object.keys(this.sessionStorage).filter(k => 
                k.startsWith(`memscope_${this.cacheVersion}_`)).length,
            localStorage: Object.keys(this.localStorage).filter(k => 
                k.startsWith(`memscope_persistent_${this.cacheVersion}_`)).length
        };
    }
}

/**
 * 数据标准化器 - 统一不同JSON文件的数据格式
 */
class DataNormalizer {
    constructor() {
        this.typeInferenceCache = new Map();
    }

    /**
     * 标准化分配数据
     */
    normalizeAllocations(loadedData) {
        const allAllocations = [];
        
        // 从memory_analysis获取主要分配数据
        if (loadedData.memory_analysis?.allocations) {
            const memoryAllocs = loadedData.memory_analysis.allocations.map(alloc => 
                this.normalizeAllocation(alloc, 'memory_analysis')
            );
            allAllocations.push(...memoryAllocs);
        }
        
        // 从performance获取性能相关分配数据
        if (loadedData.performance?.performance_metrics?.allocations) {
            const perfAllocs = loadedData.performance.performance_metrics.allocations.map(alloc => 
                this.normalizeAllocation(alloc, 'performance')
            );
            allAllocations.push(...perfAllocs);
        }
        
        // 去重和排序
        return this.deduplicateAndSort(allAllocations);
    }

    /**
     * 标准化单个分配记录
     */
    normalizeAllocation(alloc, source) {
        // 统一指针格式
        const ptr = typeof alloc.ptr === 'string' ? 
            parseInt(alloc.ptr.replace('0x', ''), 16) : alloc.ptr;
        
        // 智能类型推断
        const inferredType = this.inferType(alloc);
        
        return {
            id: `${source}_${ptr}`,
            ptr: ptr,
            size: alloc.size || 0,
            type_name: alloc.type_name || inferredType,
            var_name: alloc.var_name || null,
            timestamp: alloc.timestamp_alloc || alloc.timestamp || Date.now(),
            timestamp_dealloc: alloc.timestamp_dealloc || null,
            scope_name: alloc.scope_name || null,
            call_stack: alloc.stack_trace || alloc.call_stack || [],
            borrow_count: alloc.borrow_count || 0,
            is_leaked: alloc.is_leaked || false,
            lifetime_ms: alloc.lifetime_ms || null,
            source: source,
            metadata: {
                inferred_type: !alloc.type_name,
                has_var_name: !!alloc.var_name,
                has_call_stack: !!(alloc.stack_trace || alloc.call_stack),
                risk_level: this.assessRiskLevel(alloc)
            }
        };
    }

    /**
     * 智能类型推断
     */
    inferType(alloc) {
        // 缓存推断结果
        const cacheKey = `${alloc.size}_${alloc.var_name || 'unknown'}`;
        if (this.typeInferenceCache.has(cacheKey)) {
            return this.typeInferenceCache.get(cacheKey);
        }
        
        let inferredType = 'Unknown';
        
        // 基于变量名推断
        if (alloc.var_name) {
            const varName = alloc.var_name.toLowerCase();
            if (varName.includes('vec') || varName.includes('vector')) {
                inferredType = 'Vec<T>';
            } else if (varName.includes('string') || varName.includes('str')) {
                inferredType = 'String';
            } else if (varName.includes('map') || varName.includes('hash')) {
                inferredType = 'HashMap<K,V>';
            } else if (varName.includes('box')) {
                inferredType = 'Box<T>';
            } else if (varName.includes('rc')) {
                inferredType = 'Rc<T>';
            } else if (varName.includes('arc')) {
                inferredType = 'Arc<T>';
            }
        }
        
        // 基于大小推断（如果变量名推断失败）
        if (inferredType === 'Unknown') {
            const size = alloc.size || 0;
            if (size <= 8) {
                inferredType = 'Primitive';
            } else if (size <= 32) {
                inferredType = 'Small Struct';
            } else if (size <= 1024) {
                inferredType = 'Medium Struct';
            } else if (size <= 1048576) {
                inferredType = 'Large Buffer';
            } else {
                inferredType = 'Huge Object';
            }
        }
        
        this.typeInferenceCache.set(cacheKey, inferredType);
        return inferredType;
    }

    /**
     * 评估风险级别
     */
    assessRiskLevel(alloc) {
        let riskScore = 0;
        
        // 大分配增加风险
        if (alloc.size > 1024 * 1024) riskScore += 3;
        else if (alloc.size > 1024) riskScore += 1;
        
        // 无变量名增加风险
        if (!alloc.var_name) riskScore += 1;
        
        // 无调用栈增加风险
        if (!alloc.stack_trace && !alloc.call_stack) riskScore += 1;
        
        if (riskScore >= 4) return 'HIGH';
        if (riskScore >= 2) return 'MEDIUM';
        return 'LOW';
    }

    /**
     * 去重和排序
     */
    deduplicateAndSort(allocations) {
        // 基于指针去重
        const uniqueAllocs = new Map();
        allocations.forEach(alloc => {
            const key = alloc.ptr;
            if (!uniqueAllocs.has(key) || uniqueAllocs.get(key).source === 'performance') {
                uniqueAllocs.set(key, alloc);
            }
        });
        
        // 按时间戳排序
        return Array.from(uniqueAllocs.values()).sort((a, b) => b.timestamp - a.timestamp);
    }

    /**
     * 标准化性能数据
     */
    normalizePerformance(performanceData) {
        if (!performanceData?.performance_metrics) {
            return {
                active_allocations: 0,
                active_memory: 0,
                peak_memory: 0,
                metrics: {}
            };
        }
        
        const metrics = performanceData.performance_metrics;
        return {
            active_allocations: metrics.active_allocations || 0,
            active_memory: metrics.active_memory || 0,
            peak_memory: metrics.peak_memory || metrics.active_memory || 0,
            metrics: {
                allocation_rate: this.calculateAllocationRate(metrics),
                memory_efficiency: this.calculateMemoryEfficiency(metrics),
                fragmentation_score: this.calculateFragmentation(metrics)
            }
        };
    }

    /**
     * 标准化安全数据
     */
    normalizeSecurity(securityData) {
        if (!securityData?.security_violations) {
            return {
                violations: [],
                risk_level: 'LOW',
                summary: { total_violations: 0 }
            };
        }
        
        const violations = securityData.security_violations.map(violation => ({
            type: Object.keys(violation)[0],
            details: violation[Object.keys(violation)[0]],
            severity: this.assessViolationSeverity(violation),
            timestamp: violation.timestamp || securityData.timestamp
        }));
        
        return {
            violations,
            risk_level: this.calculateOverallRiskLevel(violations),
            summary: {
                total_violations: violations.length,
                by_severity: this.groupViolationsBySeverity(violations)
            }
        };
    }

    /**
     * 标准化不安全FFI数据
     */
    normalizeUnsafeFFI(unsafeFFIData) {
        if (!Array.isArray(unsafeFFIData)) {
            return {
                allocations: [],
                boundary_events: [],
                safety_score: 100
            };
        }
        
        const allocations = unsafeFFIData.map(item => ({
            ...item.base,
            source_info: item.source,
            call_stack: item.call_stack || [],
            cross_boundary_events: item.cross_boundary_events || [],
            safety_violations: item.safety_violations || [],
            ffi_tracked: item.ffi_tracked || false
        }));
        
        return {
            allocations,
            boundary_events: this.extractBoundaryEvents(unsafeFFIData),
            safety_score: this.calculateSafetyScore(allocations)
        };
    }

    /**
     * 标准化复杂类型数据
     */
    normalizeComplexTypes(complexTypesData) {
        if (!complexTypesData?.categorized_types) {
            return {
                categories: {},
                analysis: [],
                summary: { total_types: 0 }
            };
        }
        
        return {
            categories: complexTypesData.categorized_types,
            analysis: complexTypesData.complex_type_analysis || [],
            summary: complexTypesData.summary || { total_types: 0 },
            optimization_recommendations: complexTypesData.optimization_recommendations || []
        };
    }

    /**
     * 建立数据关联
     */
    buildDataRelationships(normalizedData) {
        const relationships = {
            pointer_cross_references: new Map(),
            type_groupings: new Map(),
            temporal_clusters: [],
            call_stack_patterns: []
        };
        
        // 建立指针交叉引用
        normalizedData.allocations.forEach(alloc => {
            const ptr = alloc.ptr;
            if (!relationships.pointer_cross_references.has(ptr)) {
                relationships.pointer_cross_references.set(ptr, []);
            }
            relationships.pointer_cross_references.get(ptr).push({
                source: alloc.source,
                allocation: alloc
            });
        });
        
        // 建立类型分组
        normalizedData.allocations.forEach(alloc => {
            const type = alloc.type_name;
            if (!relationships.type_groupings.has(type)) {
                relationships.type_groupings.set(type, []);
            }
            relationships.type_groupings.get(type).push(alloc);
        });
        
        return relationships;
    }

    // 辅助方法
    calculateAllocationRate(metrics) {
        return metrics.allocations ? metrics.allocations.length / 1000 : 0;
    }

    calculateMemoryEfficiency(metrics) {
        const active = metrics.active_memory || 0;
        const peak = metrics.peak_memory || active;
        return peak > 0 ? Math.round((active / peak) * 100) : 100;
    }

    calculateFragmentation(metrics) {
        return metrics.allocations ? Math.min(100, metrics.allocations.length / 10) : 0;
    }

    assessViolationSeverity(violation) {
        const type = Object.keys(violation)[0];
        const severityMap = {
            'DoubleFree': 'CRITICAL',
            'UseAfterFree': 'CRITICAL',
            'BufferOverflow': 'HIGH',
            'MemoryLeak': 'MEDIUM',
            'InvalidPointer': 'HIGH'
        };
        return severityMap[type] || 'LOW';
    }

    calculateOverallRiskLevel(violations) {
        if (violations.some(v => v.severity === 'CRITICAL')) return 'CRITICAL';
        if (violations.some(v => v.severity === 'HIGH')) return 'HIGH';
        if (violations.some(v => v.severity === 'MEDIUM')) return 'MEDIUM';
        return 'LOW';
    }

    groupViolationsBySeverity(violations) {
        return violations.reduce((acc, v) => {
            acc[v.severity] = (acc[v.severity] || 0) + 1;
            return acc;
        }, {});
    }

    extractBoundaryEvents(unsafeFFIData) {
        return unsafeFFIData.flatMap(item => item.cross_boundary_events || []);
    }

    calculateSafetyScore(allocations) {
        if (allocations.length === 0) return 100;
        const violationCount = allocations.reduce((sum, alloc) => 
            sum + (alloc.safety_violations?.length || 0), 0);
        return Math.max(0, 100 - (violationCount * 10));
    }
}

/**
 * 安全/FFI 仪表板管理器 - 任务 7.1: 创建安全违规卡片
 */
class SecurityDashboard {
    constructor(visualizer) {
        this.visualizer = visualizer;
        this.securityData = null;
        this.updateInterval = null;
        this.init();
    }

    /**
     * 初始化安全仪表板
     */
    async init() {
        console.log('🔒 初始化安全仪表板');
        try {
            await this.loadSecurityData();
            this.createDashboardUI();
            this.renderSecurityCards();
            this.renderRiskAnalysis();
            this.renderFFITracking();
            this.startAutoUpdate();
            console.log('✅ 安全仪表板初始化成功');
        } catch (error) {
            console.error('❌ 安全仪表板初始化失败:', error);
        }
    }

    /**
     * 加载安全数据
     */
    async loadSecurityData() {
        try {
            const response = await fetch('/api/unsafe-ffi');
            const result = await response.json();
            if (result.success) {
                this.securityData = result.data;
                console.log('🔒 安全数据加载成功:', this.securityData);
            } else {
                throw new Error(result.error || '加载安全数据失败');
            }
        } catch (error) {
            console.error('❌ 加载安全数据失败:', error);
            // 使用模拟数据作为后备
            this.securityData = this.getMockSecurityData();
        }
    }

    /**
     * 创建安全仪表板UI结构
     */
    createDashboardUI() {
        const dashboardContainer = document.createElement('div');
        dashboardContainer.className = 'security-dashboard';
        dashboardContainer.innerHTML = `
            <div class="dashboard-header">
                <h2>🔒 安全 & FFI 分析</h2>
                <div class="dashboard-controls">
                    <button class="refresh-security-btn">🔄 刷新</button>
                    <button class="toggle-security-auto-update" data-enabled="true">⏱️ 自动更新</button>
                </div>
            </div>
            
            <div class="security-overview">
                <!-- 任务 7.1: 安全违规卡片 -->
                <div id="security-cards-container" class="security-cards"></div>
            </div>
            
            <div class="security-analysis">
                <!-- 任务 7.3: 安全风险评估 -->
                <div id="risk-analysis-container" class="risk-analysis"></div>
            </div>
            
            <div class="ffi-tracking">
                <!-- 任务 7.2: FFI 调用跟踪 -->
                <div id="ffi-tracking-container" class="ffi-tracking"></div>
            </div>
        `;

        // 插入到性能仪表板之后
        const performanceDashboard = document.querySelector('.performance-dashboard');
        if (performanceDashboard && performanceDashboard.nextSibling) {
            performanceDashboard.parentNode.insertBefore(dashboardContainer, performanceDashboard.nextSibling);
        } else {
            const content = document.querySelector('.content');
            if (content) {
                content.appendChild(dashboardContainer);
            }
        }

        this.bindDashboardEvents();
    }

    /**
     * 绑定仪表板事件
     */
    bindDashboardEvents() {
        const refreshBtn = document.querySelector('.refresh-security-btn');
        const autoUpdateBtn = document.querySelector('.toggle-security-auto-update');

        if (refreshBtn) {
            refreshBtn.addEventListener('click', () => this.refreshData());
        }

        if (autoUpdateBtn) {
            autoUpdateBtn.addEventListener('click', () => this.toggleAutoUpdate());
        }
    }

    /**
     * 任务 7.1: 渲染安全违规卡片
     */
    renderSecurityCards() {
        const container = document.getElementById('security-cards-container');
        if (!container || !this.securityData) return;

        const overview = this.securityData.overview;
        const metrics = this.securityData.security_metrics;

        container.innerHTML = `
            <!-- 总体安全状态卡片 -->
            <div class="security-card overall-security">
                <div class="card-header">
                    <h3>🛡️ 总体安全状态</h3>
                    <span class="security-badge" style="background: ${overview.security_level_color}">
                        ${overview.security_level.toUpperCase()}
                    </span>
                </div>
                <div class="card-content">
                    <div class="security-score-container">
                        <div class="security-score">
                            <span class="score-value">${overview.security_score}</span>
                            <span class="score-label">安全评分</span>
                        </div>
                        <div class="score-ring">
                            <svg width="120" height="120" viewBox="0 0 120 120">
                                <circle cx="60" cy="60" r="50" fill="none" stroke="#e5e7eb" stroke-width="8"/>
                                <circle cx="60" cy="60" r="50" fill="none" stroke="${overview.security_level_color}" 
                                        stroke-width="8" stroke-dasharray="${2 * Math.PI * 50}" 
                                        stroke-dashoffset="${2 * Math.PI * 50 * (1 - overview.security_score / 100)}"
                                        transform="rotate(-90 60 60)"/>
                            </svg>
                        </div>
                    </div>
                    <div class="security-summary">
                        <div class="summary-item">
                            <span class="label">风险评估</span>
                            <span class="value">${overview.risk_assessment}</span>
                        </div>
                        <div class="summary-item">
                            <span class="label">安全违规</span>
                            <span class="value">${overview.violations_formatted}</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 安全违规卡片 -->
            <div class="security-card violations">
                <div class="card-header">
                    <h3>⚠️ 安全违规</h3>
                    <span class="metric-badge" style="background: ${metrics.violations.color_hint}">
                        ${overview.total_violations}
                    </span>
                </div>
                <div class="card-content">
                    <div class="primary-metric">
                        <span class="value">${overview.violations_formatted}</span>
                        <span class="label">检测到的违规</span>
                    </div>
                    <div class="violation-breakdown">
                        <div class="breakdown-item critical">
                            <span class="severity-icon">🔴</span>
                            <span class="severity-label">严重</span>
                            <span class="severity-count">${metrics.violations.severity_breakdown.critical}</span>
                        </div>
                        <div class="breakdown-item high">
                            <span class="severity-icon">🟠</span>
                            <span class="severity-label">高</span>
                            <span class="severity-count">${metrics.violations.severity_breakdown.high}</span>
                        </div>
                        <div class="breakdown-item medium">
                            <span class="severity-icon">🟡</span>
                            <span class="severity-label">中</span>
                            <span class="severity-count">${metrics.violations.severity_breakdown.medium}</span>
                        </div>
                        <div class="breakdown-item low">
                            <span class="severity-icon">🟢</span>
                            <span class="severity-label">低</span>
                            <span class="severity-count">${metrics.violations.severity_breakdown.low}</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Unsafe 操作卡片 -->
            <div class="security-card unsafe-operations">
                <div class="card-header">
                    <h3>⚡ Unsafe 操作</h3>
                    <span class="metric-badge" style="background: ${metrics.unsafe_operations.color_hint}">
                        ${metrics.unsafe_operations.risk_level}
                    </span>
                </div>
                <div class="card-content">
                    <div class="primary-metric">
                        <span class="value">${overview.unsafe_count_formatted}</span>
                        <span class="label">Unsafe 操作</span>
                    </div>
                    <div class="secondary-metrics">
                        <div class="metric-item">
                            <span class="label">占比</span>
                            <span class="value">${overview.unsafe_percentage}%</span>
                        </div>
                        <div class="metric-item">
                            <span class="label">风险等级</span>
                            <span class="value">${metrics.unsafe_operations.risk_level}</span>
                        </div>
                    </div>
                    <div class="progress-bar">
                        <div class="progress-fill" style="width: ${overview.unsafe_percentage}%; background: ${metrics.unsafe_operations.color_hint}"></div>
                    </div>
                </div>
            </div>

            <!-- FFI 交互卡片 -->
            <div class="security-card ffi-interactions">
                <div class="card-header">
                    <h3>🔗 FFI 交互</h3>
                    <span class="metric-badge" style="background: ${metrics.ffi_interactions.color_hint}">
                        ${metrics.ffi_interactions.risk_level}
                    </span>
                </div>
                <div class="card-content">
                    <div class="primary-metric">
                        <span class="value">${overview.ffi_count_formatted}</span>
                        <span class="label">FFI 调用</span>
                    </div>
                    <div class="secondary-metrics">
                        <div class="metric-item">
                            <span class="label">占比</span>
                            <span class="value">${overview.ffi_percentage}%</span>
                        </div>
                        <div class="metric-item">
                            <span class="label">边界事件</span>
                            <span class="value">${overview.boundary_events}</span>
                        </div>
                        <div class="metric-item">
                            <span class="label">风险等级</span>
                            <span class="value">${metrics.ffi_interactions.risk_level}</span>
                        </div>
                    </div>
                </div>
            </div>
        `;
    }

    /**
     * 任务 7.3: 渲染安全风险评估
     */
    renderRiskAnalysis() {
        const container = document.getElementById('risk-analysis-container');
        if (!container || !this.securityData) return;

        const riskAnalysis = this.securityData.risk_analysis;
        const recommendations = this.securityData.recommendations;

        container.innerHTML = `
            <div class="risk-analysis-section">
                <h3>📊 风险分析</h3>
                <div class="risk-factors">
                    ${riskAnalysis.risk_factors.map(factor => `
                        <div class="risk-factor ${factor.level}">
                            <div class="factor-header">
                                <span class="factor-name">${factor.factor}</span>
                                <span class="factor-level ${factor.level}">${factor.level.toUpperCase()}</span>
                            </div>
                            <div class="factor-details">
                                <span class="factor-count">${factor.count}</span>
                                <span class="factor-description">${factor.description}</span>
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
            
            <div class="recommendations-section">
                <h3>💡 安全建议</h3>
                <div class="recommendations-grid">
                    <div class="priority-actions">
                        <h4>🚨 优先行动</h4>
                        <ul class="action-list">
                            ${recommendations.priority_actions.map(action => `
                                <li class="action-item">${action}</li>
                            `).join('')}
                        </ul>
                    </div>
                    <div class="security-improvements">
                        <h4>🔧 安全改进</h4>
                        <ul class="improvement-list">
                            ${recommendations.security_improvements.map(improvement => `
                                <li class="improvement-item">${improvement}</li>
                            `).join('')}
                        </ul>
                    </div>
                </div>
            </div>
        `;
    }

    /**
     * 任务 7.2: 实现 FFI 调用跟踪（简化版）
     */
    renderFFITracking() {
        const container = document.getElementById('ffi-tracking-container');
        if (!container || !this.securityData) return;

        const rawData = this.securityData.raw_data;
        const ffiData = rawData.enhanced_ffi_data || [];
        const boundaryEvents = rawData.boundary_events || [];

        container.innerHTML = `
            <div class="ffi-tracking-section">
                <h3>🔗 FFI 调用跟踪</h3>
                <div class="ffi-summary">
                    <div class="ffi-stat">
                        <span class="stat-label">FFI 调用总数</span>
                        <span class="stat-value">${ffiData.length}</span>
                    </div>
                    <div class="ffi-stat">
                        <span class="stat-label">边界事件</span>
                        <span class="stat-value">${boundaryEvents.length}</span>
                    </div>
                </div>
                
                ${ffiData.length > 0 ? `
                    <div class="ffi-calls-list">
                        <h4>最近的 FFI 调用</h4>
                        <div class="ffi-calls">
                            ${ffiData.slice(0, 5).map((call, index) => `
                                <div class="ffi-call-item">
                                    <span class="call-index">#${index + 1}</span>
                                    <span class="call-info">FFI 调用</span>
                                    <span class="call-status">已跟踪</span>
                                </div>
                            `).join('')}
                        </div>
                    </div>
                ` : `
                    <div class="no-ffi-calls">
                        <p>🎉 未检测到 FFI 调用</p>
                        <p class="no-calls-description">这是一个好兆头！您的代码没有使用外部函数接口。</p>
                    </div>
                `}
            </div>
        `;
    }

    /**
     * 刷新安全数据
     */
    async refreshData() {
        console.log('🔄 刷新安全数据');
        try {
            await this.loadSecurityData();
            this.renderSecurityCards();
            this.renderRiskAnalysis();
            this.renderFFITracking();
            console.log('✅ 安全数据刷新成功');
        } catch (error) {
            console.error('❌ 刷新安全数据失败:', error);
        }
    }

    /**
     * 切换自动更新
     */
    toggleAutoUpdate() {
        const btn = document.querySelector('.toggle-security-auto-update');
        const isEnabled = btn.dataset.enabled === 'true';
        
        if (isEnabled) {
            this.stopAutoUpdate();
            btn.dataset.enabled = 'false';
            btn.textContent = '⏸️ 已暂停';
            btn.style.background = '#6b7280';
        } else {
            this.startAutoUpdate();
            btn.dataset.enabled = 'true';
            btn.textContent = '⏱️ 自动更新';
            btn.style.background = '#10b981';
        }
    }

    /**
     * 开始自动更新
     */
    startAutoUpdate() {
        this.stopAutoUpdate(); // 清除现有的定时器
        this.updateInterval = setInterval(() => {
            this.refreshData();
        }, 60000); // 每60秒更新一次（安全数据更新频率较低）
    }

    /**
     * 停止自动更新
     */
    stopAutoUpdate() {
        if (this.updateInterval) {
            clearInterval(this.updateInterval);
            this.updateInterval = null;
        }
    }

    /**
     * 获取模拟安全数据（后备方案）
     */
    getMockSecurityData() {
        return {
            overview: {
                security_level: "low",
                security_score: 95,
                risk_assessment: "low",
                total_violations: 0,
                unsafe_count: 0,
                ffi_count: 0,
                boundary_events: 0,
                unsafe_percentage: 0,
                ffi_percentage: 0,
                violations_formatted: "0",
                unsafe_count_formatted: "0",
                ffi_count_formatted: "0",
                security_level_color: "#16a34a"
            },
            security_metrics: {
                violations: {
                    count: 0,
                    severity_breakdown: { critical: 0, high: 0, medium: 0, low: 0 },
                    color_hint: "#16a34a"
                },
                unsafe_operations: {
                    count: 0,
                    percentage: 0,
                    risk_level: "low",
                    color_hint: "#16a34a"
                },
                ffi_interactions: {
                    count: 0,
                    percentage: 0,
                    boundary_events: 0,
                    risk_level: "low",
                    color_hint: "#16a34a"
                }
            },
            risk_analysis: {
                overall_risk: "low",
                security_score: 95,
                risk_factors: [
                    {
                        factor: "Safety Violations",
                        level: "low",
                        count: 0,
                        description: "0 safety violations detected"
                    },
                    {
                        factor: "Unsafe Operations",
                        level: "low",
                        count: 0,
                        description: "0% of operations are unsafe"
                    },
                    {
                        factor: "FFI Interactions",
                        level: "low",
                        count: 0,
                        description: "0 FFI interactions detected"
                    }
                ]
            },
            recommendations: {
                priority_actions: [
                    "Maintain current security practices",
                    "Regular security audits recommended"
                ],
                security_improvements: [
                    "Enable additional compiler warnings",
                    "Use static analysis tools",
                    "Implement memory sanitizers in testing",
                    "Regular dependency security audits"
                ]
            },
            raw_data: {
                enhanced_ffi_data: [],
                boundary_events: []
            }
        };
    }

    /**
     * 销毁安全仪表板
     */
    destroy() {
        this.stopAutoUpdate();
        const dashboard = document.querySelector('.security-dashboard');
        if (dashboard) {
            dashboard.remove();
        }
    }
}

/**
 * 性能仪表板管理器 - 任务 6.1: 创建性能仪表板组件
 */
class PerformanceDashboard {
    constructor(visualizer) {
        this.visualizer = visualizer;
        this.performanceData = null;
        this.charts = {};
        this.updateInterval = null;
        this.init();
    }

    /**
     * 初始化性能仪表板
     */
    async init() {
        console.log('📊 初始化性能仪表板');
        try {
            await this.loadPerformanceData();
            this.createDashboardUI();
            this.renderMetricCards();
            this.renderTrendCharts();
            this.startAutoUpdate();
            console.log('✅ 性能仪表板初始化成功');
        } catch (error) {
            console.error('❌ 性能仪表板初始化失败:', error);
        }
    }

    /**
     * 加载性能数据
     */
    async loadPerformanceData() {
        try {
            const response = await fetch('/api/performance');
            const result = await response.json();
            if (result.success) {
                this.performanceData = result.data;
                console.log('📈 性能数据加载成功:', this.performanceData);
            } else {
                throw new Error(result.error || '加载性能数据失败');
            }
        } catch (error) {
            console.error('❌ 加载性能数据失败:', error);
            // 使用模拟数据作为后备
            this.performanceData = this.getMockPerformanceData();
        }
    }

    /**
     * 创建仪表板UI结构
     */
    createDashboardUI() {
        const dashboardContainer = document.createElement('div');
        dashboardContainer.className = 'performance-dashboard';
        dashboardContainer.innerHTML = `
            <div class="dashboard-header">
                <h2>📊 性能仪表板</h2>
                <div class="dashboard-controls">
                    <button class="refresh-btn" onclick="this.refreshData()">🔄 刷新</button>
                    <button class="toggle-auto-update" data-enabled="true">⏱️ 自动更新</button>
                </div>
            </div>
            
            <div class="performance-metrics-grid">
                <!-- 任务 6.2: 性能指标卡片将在这里渲染 -->
                <div id="performance-cards-container" class="metrics-cards"></div>
            </div>
            
            <div class="performance-charts-grid">
                <!-- 任务 6.3: 性能趋势图表将在这里渲染 -->
                <div id="performance-charts-container" class="charts-container"></div>
            </div>
        `;

        // 插入到过滤控件之后
        const filterControls = document.querySelector('.filter-controls');
        if (filterControls && filterControls.nextSibling) {
            filterControls.parentNode.insertBefore(dashboardContainer, filterControls.nextSibling);
        } else {
            const content = document.querySelector('.content');
            if (content) {
                content.appendChild(dashboardContainer);
            }
        }

        this.bindDashboardEvents();
    }

    /**
     * 绑定仪表板事件
     */
    bindDashboardEvents() {
        const refreshBtn = document.querySelector('.refresh-btn');
        const autoUpdateBtn = document.querySelector('.toggle-auto-update');

        if (refreshBtn) {
            refreshBtn.addEventListener('click', () => this.refreshData());
        }

        if (autoUpdateBtn) {
            autoUpdateBtn.addEventListener('click', () => this.toggleAutoUpdate());
        }
    }

    /**
     * 任务 6.2: 渲染性能指标卡片
     */
    renderMetricCards() {
        const container = document.getElementById('performance-cards-container');
        if (!container || !this.performanceData) return;

        const overview = this.performanceData.overview;
        const metrics = this.performanceData.metrics;

        container.innerHTML = `
            <!-- 内存利用率卡片 -->
            <div class="metric-card memory-utilization">
                <div class="card-header">
                    <h3>💾 内存利用率</h3>
                    <span class="metric-badge" style="background: ${metrics.memory_utilization.color_hint}">
                        ${overview.memory_efficiency}%
                    </span>
                </div>
                <div class="card-content">
                    <div class="primary-metric">
                        <span class="value">${overview.active_memory_formatted}</span>
                        <span class="label">当前使用</span>
                    </div>
                    <div class="secondary-metrics">
                        <div class="metric-item">
                            <span class="label">峰值内存</span>
                            <span class="value">${overview.peak_memory_formatted}</span>
                        </div>
                        <div class="metric-item">
                            <span class="label">内存浪费</span>
                            <span class="value">${metrics.memory_utilization.waste_formatted}</span>
                        </div>
                    </div>
                    <div class="progress-bar">
                        <div class="progress-fill" style="width: ${overview.memory_efficiency}%; background: ${metrics.memory_utilization.color_hint}"></div>
                    </div>
                </div>
            </div>

            <!-- 分配性能卡片 -->
            <div class="metric-card allocation-performance">
                <div class="card-header">
                    <h3>⚡ 分配性能</h3>
                    <span class="metric-badge" style="background: ${metrics.allocation_performance.color_hint}">
                        ${overview.performance_class}
                    </span>
                </div>
                <div class="card-content">
                    <div class="primary-metric">
                        <span class="value">${overview.allocation_rate_formatted}</span>
                        <span class="label">分配速率</span>
                    </div>
                    <div class="secondary-metrics">
                        <div class="metric-item">
                            <span class="label">总分配</span>
                            <span class="value">${overview.total_allocations_formatted}</span>
                        </div>
                        <div class="metric-item">
                            <span class="label">活跃分配</span>
                            <span class="value">${overview.active_allocations_formatted}</span>
                        </div>
                        <div class="metric-item">
                            <span class="label">释放率</span>
                            <span class="value">${metrics.allocation_performance.deallocation_rate}%</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 内存碎片化卡片 -->
            <div class="metric-card fragmentation">
                <div class="card-header">
                    <h3>🧩 内存碎片化</h3>
                    <span class="metric-badge" style="background: ${metrics.fragmentation.color_hint}">
                        ${metrics.fragmentation.score}
                    </span>
                </div>
                <div class="card-content">
                    <div class="primary-metric">
                        <span class="value">${metrics.fragmentation.avg_allocation_size_formatted}</span>
                        <span class="label">平均分配大小</span>
                    </div>
                    <div class="secondary-metrics">
                        <div class="metric-item">
                            <span class="label">小分配 (≤64B)</span>
                            <span class="value">${metrics.fragmentation.small_allocations}</span>
                        </div>
                        <div class="metric-item">
                            <span class="label">大分配 (>1MB)</span>
                            <span class="value">${metrics.fragmentation.large_allocations}</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 系统健康度卡片 -->
            <div class="metric-card system-health">
                <div class="card-header">
                    <h3>🏥 系统健康度</h3>
                    <span class="metric-badge" style="background: ${this.getHealthColor()}">
                        ${this.getHealthScore()}
                    </span>
                </div>
                <div class="card-content">
                    <div class="health-indicators">
                        <div class="indicator ${overview.memory_efficiency > 80 ? 'good' : overview.memory_efficiency > 60 ? 'warning' : 'critical'}">
                            <span class="indicator-icon">💾</span>
                            <span class="indicator-label">内存效率</span>
                            <span class="indicator-value">${overview.memory_efficiency}%</span>
                        </div>
                        <div class="indicator ${overview.performance_class === 'excellent' || overview.performance_class === 'good' ? 'good' : overview.performance_class === 'fair' ? 'warning' : 'critical'}">
                            <span class="indicator-icon">⚡</span>
                            <span class="indicator-label">性能等级</span>
                            <span class="indicator-value">${overview.performance_class}</span>
                        </div>
                        <div class="indicator ${metrics.fragmentation.score === 'low' ? 'good' : metrics.fragmentation.score === 'medium' ? 'warning' : 'critical'}">
                            <span class="indicator-icon">🧩</span>
                            <span class="indicator-label">碎片化</span>
                            <span class="indicator-value">${metrics.fragmentation.score}</span>
                        </div>
                    </div>
                </div>
            </div>
        `;
    }

    /**
     * 任务 6.3: 渲染性能趋势图表
     */
    renderTrendCharts() {
        const container = document.getElementById('performance-charts-container');
        if (!container || !this.performanceData) return;

        container.innerHTML = `
            <div class="chart-section">
                <h3>📈 内存使用趋势</h3>
                <div id="memory-trend-chart" class="chart-container">
                    <canvas id="memoryTrendCanvas" width="800" height="300"></canvas>
                </div>
            </div>
            
            <div class="chart-section">
                <h3>📊 分配大小分布</h3>
                <div id="size-distribution-chart" class="chart-container">
                    <canvas id="sizeDistributionCanvas" width="400" height="300"></canvas>
                </div>
            </div>
        `;

        // 渲染内存趋势图
        this.renderMemoryTrendChart();
        
        // 渲染分配大小分布图
        this.renderSizeDistributionChart();
    }

    /**
     * 渲染内存趋势图表
     */
    renderMemoryTrendChart() {
        const canvas = document.getElementById('memoryTrendCanvas');
        if (!canvas || !this.performanceData.trends) return;

        const ctx = canvas.getContext('2d');
        const timeline = this.performanceData.trends.memory_timeline;
        
        // 简单的折线图实现
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.strokeStyle = '#667eea';
        ctx.lineWidth = 2;
        ctx.beginPath();

        if (timeline.length > 0) {
            const maxSize = Math.max(...timeline.map(t => t.size));
            const step = canvas.width / timeline.length;
            
            timeline.forEach((point, index) => {
                const x = index * step;
                const y = canvas.height - (point.size / maxSize * canvas.height * 0.8);
                
                if (index === 0) {
                    ctx.moveTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
            });
        }
        
        ctx.stroke();
        
        // 添加标签
        ctx.fillStyle = '#374151';
        ctx.font = '12px sans-serif';
        ctx.fillText('内存使用量随时间变化', 10, 20);
    }

    /**
     * 渲染分配大小分布图表
     */
    renderSizeDistributionChart() {
        const canvas = document.getElementById('sizeDistributionCanvas');
        if (!canvas || !this.performanceData.trends) return;

        const ctx = canvas.getContext('2d');
        const distribution = this.performanceData.trends.size_distribution;
        
        // 简单的饼图实现
        const centerX = canvas.width / 2;
        const centerY = canvas.height / 2;
        const radius = Math.min(centerX, centerY) - 20;
        
        const total = Object.values(distribution).reduce((sum, val) => sum + val, 0);
        const colors = ['#10b981', '#3b82f6', '#f59e0b', '#ef4444', '#8b5cf6'];
        const labels = ['tiny', 'small', 'medium', 'large', 'massive'];
        
        let currentAngle = 0;
        
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        
        labels.forEach((label, index) => {
            const value = distribution[label] || 0;
            const sliceAngle = (value / total) * 2 * Math.PI;
            
            ctx.beginPath();
            ctx.moveTo(centerX, centerY);
            ctx.arc(centerX, centerY, radius, currentAngle, currentAngle + sliceAngle);
            ctx.closePath();
            ctx.fillStyle = colors[index];
            ctx.fill();
            
            // 添加标签
            const labelAngle = currentAngle + sliceAngle / 2;
            const labelX = centerX + Math.cos(labelAngle) * (radius + 15);
            const labelY = centerY + Math.sin(labelAngle) * (radius + 15);
            
            ctx.fillStyle = '#374151';
            ctx.font = '10px sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText(`${label}: ${value}`, labelX, labelY);
            
            currentAngle += sliceAngle;
        });
    }

    /**
     * 获取系统健康度分数
     */
    getHealthScore() {
        if (!this.performanceData) return 'Unknown';
        
        const overview = this.performanceData.overview;
        const metrics = this.performanceData.metrics;
        
        let score = 0;
        
        // 内存效率评分 (40%)
        if (overview.memory_efficiency > 80) score += 40;
        else if (overview.memory_efficiency > 60) score += 25;
        else if (overview.memory_efficiency > 40) score += 15;
        
        // 性能等级评分 (35%)
        switch (overview.performance_class) {
            case 'excellent': score += 35; break;
            case 'good': score += 25; break;
            case 'fair': score += 15; break;
            default: score += 5;
        }
        
        // 碎片化评分 (25%)
        switch (metrics.fragmentation.score) {
            case 'low': score += 25; break;
            case 'medium': score += 15; break;
            default: score += 5;
        }
        
        if (score >= 85) return 'Excellent';
        if (score >= 70) return 'Good';
        if (score >= 50) return 'Fair';
        return 'Poor';
    }

    /**
     * 获取健康度颜色
     */
    getHealthColor() {
        const score = this.getHealthScore();
        switch (score) {
            case 'Excellent': return '#10b981';
            case 'Good': return '#3b82f6';
            case 'Fair': return '#f59e0b';
            default: return '#ef4444';
        }
    }

    /**
     * 刷新性能数据
     */
    async refreshData() {
        console.log('🔄 刷新性能数据');
        try {
            await this.loadPerformanceData();
            this.renderMetricCards();
            this.renderTrendCharts();
            console.log('✅ 性能数据刷新成功');
        } catch (error) {
            console.error('❌ 刷新性能数据失败:', error);
        }
    }

    /**
     * 切换自动更新
     */
    toggleAutoUpdate() {
        const btn = document.querySelector('.toggle-auto-update');
        const isEnabled = btn.dataset.enabled === 'true';
        
        if (isEnabled) {
            this.stopAutoUpdate();
            btn.dataset.enabled = 'false';
            btn.textContent = '⏸️ 已暂停';
            btn.style.background = '#6b7280';
        } else {
            this.startAutoUpdate();
            btn.dataset.enabled = 'true';
            btn.textContent = '⏱️ 自动更新';
            btn.style.background = '#10b981';
        }
    }

    /**
     * 开始自动更新
     */
    startAutoUpdate() {
        this.stopAutoUpdate(); // 清除现有的定时器
        this.updateInterval = setInterval(() => {
            this.refreshData();
        }, 30000); // 每30秒更新一次
    }

    /**
     * 停止自动更新
     */
    stopAutoUpdate() {
        if (this.updateInterval) {
            clearInterval(this.updateInterval);
            this.updateInterval = null;
        }
    }

    /**
     * 获取模拟性能数据（后备方案）
     */
    getMockPerformanceData() {
        return {
            overview: {
                total_allocations: 639,
                active_allocations: 425,
                deallocated_allocations: 214,
                peak_memory: 551142,
                active_memory: 217161,
                memory_efficiency: 39,
                allocation_rate: 146,
                performance_class: "needs_optimization",
                fragmentation_score: "medium",
                peak_memory_formatted: "538.2 KB",
                active_memory_formatted: "212.1 KB",
                total_allocations_formatted: "639",
                active_allocations_formatted: "425",
                allocation_rate_formatted: "146/s"
            },
            metrics: {
                memory_utilization: {
                    current: 217161,
                    peak: 551142,
                    efficiency_percentage: 39,
                    waste: 333981,
                    waste_formatted: "326.2 KB",
                    color_hint: "#ef4444"
                },
                allocation_performance: {
                    rate: 146,
                    total_count: 639,
                    active_count: 425,
                    deallocation_rate: 33,
                    performance_class: "needs_optimization",
                    color_hint: "#ef4444"
                },
                fragmentation: {
                    score: "medium",
                    avg_allocation_size: 862,
                    avg_allocation_size_formatted: "862 B",
                    small_allocations: 569,
                    large_allocations: 4,
                    color_hint: "#f59e0b"
                }
            },
            trends: {
                memory_timeline: [],
                size_distribution: {
                    tiny: 569,
                    small: 46,
                    medium: 15,
                    large: 5,
                    massive: 4
                }
            }
        };
    }

    /**
     * 销毁仪表板
     */
    destroy() {
        this.stopAutoUpdate();
        const dashboard = document.querySelector('.performance-dashboard');
        if (dashboard) {
            dashboard.remove();
        }
    }
}

/**
 * 过滤控件管理器 - 任务 5.1: 构建 FilterControls 类
 */
class FilterControls {
    constructor(visualizer) {
        this.visualizer = visualizer;
        this.filters = {
            sizeRange: { min: 0, max: Infinity },
            typeFilter: '',
            statusFilter: 'all', // 'all', 'active', 'deallocated'
            timeRange: { start: null, end: null },
            variableFilter: '',
            sortBy: 'timestamp',
            sortOrder: 'desc'
        };
        this.originalData = null;
        this.filteredData = null;
        this.debounceTimer = null;
        this.init();
    }

    /**
     * 初始化过滤控件
     */
    init() {
        console.log('🎛️ 初始化过滤控件');
        this.createFilterUI();
        this.bindEvents();
        this.originalData = [...(this.visualizer.data.allocations || [])];
        this.applyFilters();
    }

    /**
     * 创建过滤器UI界面
     */
    createFilterUI() {
        const filterContainer = document.createElement('div');
        filterContainer.className = 'filter-controls';
        filterContainer.innerHTML = `
            <div class="filter-header">
                <h3>🎛️ 数据过滤器</h3>
                <button class="filter-toggle" onclick="this.parentElement.parentElement.classList.toggle('collapsed')">
                    <span class="toggle-icon">▼</span>
                </button>
            </div>
            <div class="filter-content">
                <div class="filter-row">
                    <div class="filter-group">
                        <label>📏 大小范围</label>
                        <div class="range-inputs">
                            <input type="number" id="minSize" placeholder="最小" min="0">
                            <span>-</span>
                            <input type="number" id="maxSize" placeholder="最大" min="0">
                            <span class="unit">bytes</span>
                        </div>
                    </div>
                    <div class="filter-group">
                        <label>📊 状态</label>
                        <select id="statusFilter">
                            <option value="all">全部</option>
                            <option value="active">活跃</option>
                            <option value="deallocated">已释放</option>
                        </select>
                    </div>
                </div>
                <div class="filter-row">
                    <div class="filter-group">
                        <label>🏷️ 类型过滤</label>
                        <input type="text" id="typeFilter" placeholder="输入类型名称...">
                    </div>
                    <div class="filter-group">
                        <label>🔤 变量过滤</label>
                        <input type="text" id="variableFilter" placeholder="输入变量名称...">
                    </div>
                </div>
                <div class="filter-row">
                    <div class="filter-group">
                        <label>📅 时间范围</label>
                        <div class="time-range">
                            <input type="datetime-local" id="startTime">
                            <span>至</span>
                            <input type="datetime-local" id="endTime">
                        </div>
                    </div>
                    <div class="filter-group">
                        <label>🔄 排序</label>
                        <div class="sort-controls">
                            <select id="sortBy">
                                <option value="timestamp">时间</option>
                                <option value="size">大小</option>
                                <option value="type_name">类型</option>
                                <option value="var_name">变量名</option>
                            </select>
                            <button id="sortOrder" class="sort-order-btn" data-order="desc">
                                <span class="sort-icon">↓</span>
                            </button>
                        </div>
                    </div>
                </div>
                <div class="filter-actions">
                    <button class="apply-filters-btn">🔍 应用过滤器</button>
                    <button class="reset-filters-btn">🔄 重置</button>
                    <span class="filter-results">显示 <span id="filteredCount">0</span> / <span id="totalCount">0</span> 项</span>
                </div>
            </div>
        `;

        // 插入到内容区域的顶部
        const content = document.querySelector('.content');
        if (content) {
            content.insertBefore(filterContainer, content.firstChild);
        }
    }

    /**
     * 绑定事件监听器
     */
    bindEvents() {
        // 实时过滤事件（带防抖）
        const inputs = ['minSize', 'maxSize', 'typeFilter', 'variableFilter', 'startTime', 'endTime'];
        inputs.forEach(id => {
            const element = document.getElementById(id);
            if (element) {
                element.addEventListener('input', () => this.debouncedFilter());
            }
        });

        // 下拉选择事件
        const selects = ['statusFilter', 'sortBy'];
        selects.forEach(id => {
            const element = document.getElementById(id);
            if (element) {
                element.addEventListener('change', () => this.applyFilters());
            }
        });

        // 排序顺序切换
        const sortOrderBtn = document.getElementById('sortOrder');
        if (sortOrderBtn) {
            sortOrderBtn.addEventListener('click', () => {
                const currentOrder = sortOrderBtn.dataset.order;
                const newOrder = currentOrder === 'desc' ? 'asc' : 'desc';
                sortOrderBtn.dataset.order = newOrder;
                sortOrderBtn.querySelector('.sort-icon').textContent = newOrder === 'desc' ? '↓' : '↑';
                this.filters.sortOrder = newOrder;
                this.applyFilters();
            });
        }

        // 应用和重置按钮
        const applyBtn = document.querySelector('.apply-filters-btn');
        const resetBtn = document.querySelector('.reset-filters-btn');
        
        if (applyBtn) {
            applyBtn.addEventListener('click', () => this.applyFilters());
        }
        
        if (resetBtn) {
            resetBtn.addEventListener('click', () => this.resetFilters());
        }
    }

    /**
     * 防抖过滤器应用
     */
    debouncedFilter() {
        clearTimeout(this.debounceTimer);
        this.debounceTimer = setTimeout(() => {
            this.applyFilters();
        }, 300);
    }

    /**
     * 应用所有过滤器
     */
    applyFilters() {
        console.log('🔍 应用过滤器');
        
        // 更新过滤器状态
        this.updateFilterState();
        
        // 应用过滤逻辑
        let filtered = [...this.originalData];
        
        // 大小过滤
        if (this.filters.sizeRange.min > 0 || this.filters.sizeRange.max < Infinity) {
            filtered = filtered.filter(alloc => 
                alloc.size >= this.filters.sizeRange.min && 
                alloc.size <= this.filters.sizeRange.max
            );
        }
        
        // 状态过滤
        if (this.filters.statusFilter !== 'all') {
            filtered = filtered.filter(alloc => {
                const isActive = !alloc.timestamp_dealloc;
                return this.filters.statusFilter === 'active' ? isActive : !isActive;
            });
        }
        
        // 类型过滤
        if (this.filters.typeFilter) {
            const typeRegex = new RegExp(this.filters.typeFilter, 'i');
            filtered = filtered.filter(alloc => 
                typeRegex.test(alloc.type_name || '')
            );
        }
        
        // 变量过滤
        if (this.filters.variableFilter) {
            const varRegex = new RegExp(this.filters.variableFilter, 'i');
            filtered = filtered.filter(alloc => 
                varRegex.test(alloc.var_name || '')
            );
        }
        
        // 时间范围过滤
        if (this.filters.timeRange.start || this.filters.timeRange.end) {
            filtered = filtered.filter(alloc => {
                const allocTime = new Date(alloc.timestamp_alloc / 1000000); // 纳秒转毫秒
                if (this.filters.timeRange.start && allocTime < this.filters.timeRange.start) {
                    return false;
                }
                if (this.filters.timeRange.end && allocTime > this.filters.timeRange.end) {
                    return false;
                }
                return true;
            });
        }
        
        // 排序
        filtered.sort((a, b) => {
            let aVal, bVal;
            switch (this.filters.sortBy) {
                case 'size':
                    aVal = a.size;
                    bVal = b.size;
                    break;
                case 'type_name':
                    aVal = a.type_name || '';
                    bVal = b.type_name || '';
                    break;
                case 'var_name':
                    aVal = a.var_name || '';
                    bVal = b.var_name || '';
                    break;
                default:
                    aVal = a.timestamp_alloc;
                    bVal = b.timestamp_alloc;
            }
            
            if (typeof aVal === 'string') {
                return this.filters.sortOrder === 'desc' ? 
                    bVal.localeCompare(aVal) : aVal.localeCompare(bVal);
            } else {
                return this.filters.sortOrder === 'desc' ? bVal - aVal : aVal - bVal;
            }
        });
        
        this.filteredData = filtered;
        
        // 更新显示
        this.updateFilterResults();
        this.updateVisualization();
        
        console.log(`✅ 过滤完成: ${filtered.length}/${this.originalData.length} 项`);
    }

    /**
     * 更新过滤器状态
     */
    updateFilterState() {
        // 大小范围
        const minSize = document.getElementById('minSize');
        const maxSize = document.getElementById('maxSize');
        this.filters.sizeRange.min = minSize ? (parseInt(minSize.value) || 0) : 0;
        this.filters.sizeRange.max = maxSize ? (parseInt(maxSize.value) || Infinity) : Infinity;
        
        // 状态
        const statusFilter = document.getElementById('statusFilter');
        this.filters.statusFilter = statusFilter ? statusFilter.value : 'all';
        
        // 类型和变量
        const typeFilter = document.getElementById('typeFilter');
        const variableFilter = document.getElementById('variableFilter');
        this.filters.typeFilter = typeFilter ? typeFilter.value.trim() : '';
        this.filters.variableFilter = variableFilter ? variableFilter.value.trim() : '';
        
        // 时间范围
        const startTime = document.getElementById('startTime');
        const endTime = document.getElementById('endTime');
        this.filters.timeRange.start = startTime && startTime.value ? new Date(startTime.value) : null;
        this.filters.timeRange.end = endTime && endTime.value ? new Date(endTime.value) : null;
        
        // 排序
        const sortBy = document.getElementById('sortBy');
        this.filters.sortBy = sortBy ? sortBy.value : 'timestamp';
    }

    /**
     * 更新过滤结果显示
     */
    updateFilterResults() {
        const filteredCount = document.getElementById('filteredCount');
        const totalCount = document.getElementById('totalCount');
        
        if (filteredCount) {
            filteredCount.textContent = this.filteredData.length;
        }
        if (totalCount) {
            totalCount.textContent = this.originalData.length;
        }
    }

    /**
     * 更新可视化显示
     */
    updateVisualization() {
        // 更新可视化器的数据
        this.visualizer.filteredAllocations = this.filteredData;
        
        // 重新渲染相关组件
        if (typeof this.visualizer.populateRecentAllocations === 'function') {
            this.visualizer.populateRecentAllocations();
        }
        if (typeof this.visualizer.populateTypeDistribution === 'function') {
            this.visualizer.populateTypeDistribution();
        }
        if (typeof this.visualizer.updateMemoryStats === 'function') {
            this.visualizer.updateMemoryStats();
        }
    }

    /**
     * 重置所有过滤器
     */
    resetFilters() {
        console.log('🔄 重置过滤器');
        
        // 重置过滤器状态
        this.filters = {
            sizeRange: { min: 0, max: Infinity },
            typeFilter: '',
            statusFilter: 'all',
            timeRange: { start: null, end: null },
            variableFilter: '',
            sortBy: 'timestamp',
            sortOrder: 'desc'
        };
        
        // 重置UI控件
        const inputs = ['minSize', 'maxSize', 'typeFilter', 'variableFilter', 'startTime', 'endTime'];
        inputs.forEach(id => {
            const element = document.getElementById(id);
            if (element) {
                element.value = '';
            }
        });
        
        const statusFilter = document.getElementById('statusFilter');
        if (statusFilter) statusFilter.value = 'all';
        
        const sortBy = document.getElementById('sortBy');
        if (sortBy) sortBy.value = 'timestamp';
        
        const sortOrderBtn = document.getElementById('sortOrder');
        if (sortOrderBtn) {
            sortOrderBtn.dataset.order = 'desc';
            sortOrderBtn.querySelector('.sort-icon').textContent = '↓';
        }
        
        // 重新应用过滤器（实际上是显示所有数据）
        this.applyFilters();
    }

    /**
     * 获取当前过滤后的数据
     */
    getFilteredData() {
        return this.filteredData || this.originalData;
    }

    /**
     * 获取过滤器状态
     */
    getFilterState() {
        return { ...this.filters };
    }

    /**
     * 设置过滤器状态
     */
    setFilterState(newFilters) {
        this.filters = { ...this.filters, ...newFilters };
        this.applyFilters();
    }
}

class MemScopeVisualizer {
    constructor(data) {
        this.data = data;
        this.filteredAllocations = [...(data.allocations || [])];
        this.filterControls = null; // 将在 init 后初始化
        this.performanceDashboard = null; // 任务 6.1: 性能仪表板实例
        this.securityDashboard = null; // 任务 7.1: 安全仪表板实例
        this.init();
    }

    init() {
        console.log('🎯 初始化MemScopeVisualizer');
        
        // 验证数据完整性
        if (!this.validateData()) {
            console.warn('⚠️ 数据验证失败，使用默认值');
            this.data = this.getDefaultData();
        }
        
        // 立即显示基础信息，避免长时间Loading
        this.updateHeaderStats();
        this.setupTabNavigation();
        
        // 使用渐进式加载，避免阻塞UI
        this.progressiveLoad();
    }

    /**
     * 验证数据完整性
     */
    validateData() {
        if (!this.data || typeof this.data !== 'object') {
            return false;
        }
        
        // 检查必要的数据结构
        if (!Array.isArray(this.data.allocations)) {
            console.warn('缺少allocations数组');
            this.data.allocations = [];
        }
        
        if (!this.data.performance) {
            console.warn('缺少performance数据');
            this.data.performance = { active_allocations: 0, active_memory: 0 };
        }
        
        if (!this.data.metadata) {
            console.warn('缺少metadata');
            this.data.metadata = { timestamp: Date.now(), sources: [] };
        }
        
        return true;
    }

    /**
     * 获取默认数据
     */
    getDefaultData() {
        return {
            allocations: [],
            performance: {
                active_allocations: 0,
                active_memory: 0,
                peak_memory: 0,
                metrics: {}
            },
            security: {
                violations: [],
                risk_level: 'LOW'
            },
            unsafeFFI: {
                allocations: [],
                safety_score: 100
            },
            complexTypes: {
                categories: {},
                summary: { total_types: 0 }
            },
            metadata: {
                timestamp: Date.now(),
                sources: [],
                loadStatus: {}
            }
        };
    }

    progressiveLoad() {
        // 分步骤加载，每步之间给UI时间响应
        const steps = [
            () => this.populateMemoryStats(),
            () => this.populateTypeDistribution(), 
            () => this.populateRecentAllocations(),
            () => this.populatePerformanceInsights(),
            () => this.setupInteractiveExplorer(),
            () => this.initializeFilterControls(), // 任务 5.1: 初始化过滤控件
            () => this.initializePerformanceDashboard(), // 任务 6.1: 初始化性能仪表板
            () => this.initializeSecurityDashboard() // 任务 7.1: 初始化安全仪表板
        ];
        
        let currentStep = 0;
        const executeStep = () => {
            if (currentStep < steps.length) {
                try {
                    steps[currentStep]();
                } catch (error) {
                    console.warn(`Step ${currentStep} failed:`, error);
                }
                currentStep++;
                
                // 使用requestAnimationFrame确保UI响应
                requestAnimationFrame(() => {
                    setTimeout(executeStep, 10); // 10ms间隔，让UI有时间更新
                });
            }
        };
        
        executeStep();
    }

    // Tab Navigation System
    setupTabNavigation() {
        const tabButtons = document.querySelectorAll('.tab-btn');
        const tabContents = document.querySelectorAll('.tab-content');

        tabButtons.forEach(button => {
            button.addEventListener('click', () => {
                const targetTab = button.getAttribute('data-tab');
                
                // Update active tab button
                tabButtons.forEach(btn => btn.classList.remove('active'));
                button.classList.add('active');
                
                // Update active tab content
                tabContents.forEach(content => content.classList.remove('active'));
                document.getElementById(targetTab).classList.add('active');
                
                // Trigger tab-specific updates
                this.onTabChange(targetTab);
            });
        });
    }

    onTabChange(tabName) {
        switch(tabName) {
            case 'overview':
                this.populateOverview();
                break;
            case 'memory-analysis':
                this.renderMemoryAnalysisDashboard();
                break;
            case 'lifecycle':
                this.renderLifecycleTimeline();
                break;
            case 'complex-types':
                this.renderComplexTypesAnalysis();
                break;
            case 'variable-relationships':
                this.renderVariableRelationships();
                break;
            case 'unsafe-ffi':
                this.renderUnsafeFFIDashboard();
                break;
            case 'interactive':
                this.updateInteractiveExplorer();
                break;
        }
    }

    // Header Statistics
    updateHeaderStats() {
        const performance = this.data.performance || {};
        
        const activeMemory = performance.active_memory || 0;
        const activeAllocs = performance.active_allocations || this.data.allocations?.length || 0;
        const peakMemory = performance.peak_memory || activeMemory;
        
        // 安全地更新DOM元素
        const totalMemoryEl = document.getElementById('totalMemory');
        const activeAllocsEl = document.getElementById('activeAllocs');
        const peakMemoryEl = document.getElementById('peakMemory');
        
        if (totalMemoryEl) {
            totalMemoryEl.textContent = `📊 ${this.formatBytes(activeMemory)}`;
        }
        
        if (activeAllocsEl) {
            activeAllocsEl.textContent = `🔢 ${activeAllocs.toLocaleString()} allocs`;
        }
        
        if (peakMemoryEl) {
            peakMemoryEl.textContent = `📈 Peak: ${this.formatBytes(peakMemory)}`;
        }
        
        console.log(`📊 统计信息更新: 内存=${this.formatBytes(activeMemory)}, 分配=${activeAllocs}, 峰值=${this.formatBytes(peakMemory)}`);
    }

    // Overview Tab Population
    populateOverview() {
        this.populateMemoryStats();
        this.populateTypeDistribution();
        this.populateRecentAllocations();
        this.populatePerformanceInsights();
    }

    populateMemoryStats() {
        const stats = this.data.stats;
        const container = document.getElementById('memoryStats');
        
        // 安全的数值计算
        const currentMemory = stats.active_memory || 0;
        const peakMemory = stats.peak_memory || 0;
        const activeAllocations = stats.active_allocations || 0;
        const totalAllocations = stats.total_allocations || this.data.allocations.length || 0;
        
        const memoryUtilization = peakMemory > 0 ? (currentMemory / peakMemory * 100).toFixed(1) : '0.0';
        
        container.innerHTML = `
            <div class="memory-stat">
                <span class="stat-label">Current Memory</span>
                <span class="stat-value">${this.formatBytes(currentMemory)}</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Peak Memory</span>
                <span class="stat-value">${this.formatBytes(peakMemory)}</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Memory Utilization</span>
                <span class="stat-value">${memoryUtilization}%</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Active Allocations</span>
                <span class="stat-value">${activeAllocations.toLocaleString()}</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Total Allocations</span>
                <span class="stat-value">${totalAllocations.toLocaleString()}</span>
            </div>
        `;
    }

    populateTypeDistribution() {
        const container = document.getElementById('typeDistribution');
        
        // 优先使用预处理的数据，避免重复计算
        if (this.data.precomputed && this.data.precomputed.type_distribution) {
            this.renderPrecomputedTypeDistribution(container, this.data.precomputed.type_distribution);
            return;
        }
        
        // 回退到原始计算方式（仅在没有预处理数据时）
        const typeMap = new Map();
        const maxAllocations = Math.min(this.data.allocations.length, 500); // 进一步减少
        const allocationsToProcess = this.data.allocations.slice(0, maxAllocations);
        
        allocationsToProcess.forEach(alloc => {
            let typeName = alloc.type_name;
            
            // 智能类型推断 - 充分利用JSON中的完整数据
            if (!typeName || typeName === 'Unknown' || typeName === null || typeName === '') {
                // 优先基于变量名推断（JSON中有完整的变量名）
                if (alloc.var_name && alloc.var_name !== 'unknown') {
                    const varName = alloc.var_name.toLowerCase();
                    if (varName.includes('vec') || varName.includes('vector')) {
                        typeName = 'Vec<T>';
                    } else if (varName.includes('string') || varName.includes('str')) {
                        typeName = 'String';
                    } else if (varName.includes('map') || varName.includes('hash')) {
                        typeName = 'HashMap<K,V>';
                    } else if (varName.includes('box')) {
                        typeName = 'Box<T>';
                    } else if (varName.includes('rc')) {
                        typeName = 'Rc<T>';
                    } else if (varName.includes('arc')) {
                        typeName = 'Arc<T>';
                    } else if (varName.includes('buffer') || varName.includes('buf')) {
                        typeName = 'Buffer';
                    } else if (varName.includes('data') || varName.includes('value')) {
                        // 基于大小进一步细化
                        if (alloc.size <= 8) {
                            typeName = 'Primitive';
                        } else if (alloc.size <= 64) {
                            typeName = 'Struct';
                        } else {
                            typeName = 'Complex Data';
                        }
                    } else {
                        // 基于大小推断
                        if (alloc.size <= 8) {
                            typeName = 'Small Value';
                        } else if (alloc.size <= 32) {
                            typeName = 'Medium Object';
                        } else if (alloc.size <= 1024) {
                            typeName = 'Large Structure';
                        } else {
                            typeName = 'Buffer/Collection';
                        }
                    }
                } else {
                    // 没有变量名时，基于大小和调用栈推断
                    if (alloc.call_stack && alloc.call_stack.length > 0) {
                        const topFrame = alloc.call_stack[0];
                        if (topFrame.function_name) {
                            const funcName = topFrame.function_name.toLowerCase();
                            if (funcName.includes('vec') || funcName.includes('vector')) {
                                typeName = 'Vec<T>';
                            } else if (funcName.includes('string')) {
                                typeName = 'String';
                            } else if (funcName.includes('alloc')) {
                                typeName = 'Raw Allocation';
                            } else {
                                typeName = 'Inferred Type';
                            }
                        }
                    } else {
                        // 最后基于大小推断
                        if (alloc.size <= 8) {
                            typeName = 'Primitive';
                        } else if (alloc.size <= 32) {
                            typeName = 'Small Struct';
                        } else if (alloc.size <= 1024) {
                            typeName = 'Medium Struct';
                        } else if (alloc.size <= 1048576) {
                            typeName = 'Large Buffer';
                        } else {
                            typeName = 'Huge Object';
                        }
                    }
                }
            }
            
            if (!typeMap.has(typeName)) {
                typeMap.set(typeName, { size: 0, count: 0 });
            }
            const current = typeMap.get(typeName);
            current.size += alloc.size;
            current.count += 1;
        });
        
        // Sort by size and take top 10
        const sortedTypes = Array.from(typeMap.entries())
            .sort((a, b) => b[1].size - a[1].size)
            .slice(0, 10);
        
        container.innerHTML = sortedTypes.map(([typeName, data]) => `
            <div class="type-item">
                <span class="type-name">${this.truncateText(typeName, 25)}</span>
                <div class="type-stats">
                    <span class="type-size">${this.formatBytes(data.size)}</span>
                    <span class="type-count">${data.count} allocs</span>
                </div>
            </div>
        `).join('');
    }

    populateRecentAllocations() {
        const container = document.getElementById('recentAllocations');
        
        // Sort by timestamp and take most recent 8
        const recentAllocs = [...this.data.allocations]
            .filter(alloc => alloc.var_name) // Only show named variables
            .sort((a, b) => b.timestamp - a.timestamp)
            .slice(0, 8);
        
        if (recentAllocs.length === 0) {
            container.innerHTML = '<p style="color: #64748b; font-style: italic;">No named variables found</p>';
            return;
        }
        
        container.innerHTML = recentAllocs.map(alloc => `
            <div class="type-item">
                <span class="type-name">${alloc.var_name}</span>
                <div class="type-stats">
                    <span class="type-size">${this.formatBytes(alloc.size)}</span>
                    <span class="type-count">${this.getDisplayTypeName(alloc)}</span>
                </div>
            </div>
        `).join('');
    }

    populatePerformanceInsights() {
        const container = document.getElementById('performanceInsights');
        
        // 优先使用预处理的性能指标
        if (this.data.precomputed && this.data.precomputed.performance_metrics) {
            this.renderPrecomputedInsights(container, this.data.precomputed.performance_metrics);
            return;
        }
        
        // 回退到动态生成
        const insights = this.generateInsights();
        container.innerHTML = insights.map(insight => `
            <div class="insight-item">
                <div class="insight-title">${insight.title}</div>
                <div class="insight-description">${insight.description}</div>
            </div>
        `).join('');
    }

    generateInsights() {
        const insights = [];
        const stats = this.data.stats;
        const allocations = this.data.allocations;
        
        // Memory utilization insight
        const utilization = (stats.active_memory / stats.peak_memory * 100);
        if (utilization > 80) {
            insights.push({
                title: "🔴 High Memory Utilization",
                description: `Current memory usage is ${utilization.toFixed(1)}% of peak. Consider optimizing memory usage.`
            });
        } else if (utilization < 30) {
            insights.push({
                title: "🟢 Efficient Memory Usage",
                description: `Memory utilization is low at ${utilization.toFixed(1)}%. Good memory management!`
            });
        }
        
        // Large allocations insight
        const largeAllocs = allocations.filter(a => a.size > 1024 * 1024); // > 1MB
        if (largeAllocs.length > 0) {
            insights.push({
                title: "📊 Large Allocations Detected",
                description: `Found ${largeAllocs.length} allocation(s) larger than 1MB. Review if necessary.`
            });
        }
        
        // Type diversity insight
        const uniqueTypes = new Set(allocations.map(a => a.type_name).filter(Boolean));
        insights.push({
            title: "🏷️ Type Diversity",
            description: `Using ${uniqueTypes.size} different types across ${allocations.length} allocations.`
        });
        
        // Unsafe/FFI insight
        if (this.data.unsafeFFI && this.data.unsafeFFI.violations.length > 0) {
            insights.push({
                title: "⚠️ Safety Violations",
                description: `Detected ${this.data.unsafeFFI.violations.length} safety violation(s). Review unsafe code carefully.`
            });
        } else if (this.data.unsafeFFI) {
            insights.push({
                title: "✅ No Safety Issues",
                description: "No memory safety violations detected in unsafe/FFI code."
            });
        }
        
        return insights;
    }

    // Interactive Explorer Setup
    setupInteractiveExplorer() {
        this.populateTypeFilter();
        this.setupEventListeners();
        this.updateInteractiveExplorer();
    }

    populateTypeFilter() {
        const select = document.getElementById('filterType');
        const types = new Set(this.data.allocations.map(a => a.type_name).filter(Boolean));
        
        select.innerHTML = '<option value="">All Types</option>' +
            Array.from(types).sort().map(type => 
                `<option value="${type}">${this.truncateText(type, 30)}</option>`
            ).join('');
    }

    setupEventListeners() {
        document.getElementById('filterType').addEventListener('change', () => this.updateFilters());
        document.getElementById('sizeRange').addEventListener('input', () => this.updateFilters());
        document.getElementById('sortBy').addEventListener('change', () => this.updateInteractiveExplorer());
    }

    updateFilters() {
        const typeFilter = document.getElementById('filterType').value;
        const sizeRange = document.getElementById('sizeRange').value;
        const maxSize = Math.max(...this.data.allocations.map(a => a.size));
        const sizeThreshold = (maxSize * sizeRange) / 100;
        
        // Update size range display
        document.getElementById('sizeRangeValue').textContent = 
            sizeRange == 100 ? 'All sizes' : `≤ ${this.formatBytes(sizeThreshold)}`;
        
        // Apply filters
        this.filteredAllocations = this.data.allocations.filter(alloc => {
            const typeMatch = !typeFilter || alloc.type_name === typeFilter;
            const sizeMatch = alloc.size <= sizeThreshold;
            return typeMatch && sizeMatch;
        });
        
        this.updateInteractiveExplorer();
    }

    updateInteractiveExplorer() {
        const sortBy = document.getElementById('sortBy').value;
        
        // Sort allocations
        const sorted = [...this.filteredAllocations].sort((a, b) => {
            switch(sortBy) {
                case 'size':
                    return b.size - a.size;
                case 'timestamp':
                    return b.timestamp - a.timestamp;
                case 'type':
                    return (a.type_name || '').localeCompare(b.type_name || '');
                default:
                    return 0;
            }
        });
        
        this.renderAllocationGrid(sorted);
    }

    renderAllocationGrid(allocations) {
        const container = document.getElementById('allocationGrid');
        
        if (allocations.length === 0) {
            container.innerHTML = `
                <div style="grid-column: 1 / -1; text-align: center; padding: 40px; color: #64748b;">
                    <h3>No allocations match the current filters</h3>
                    <p>Try adjusting the filters to see more results.</p>
                </div>
            `;
            return;
        }
        
        // 智能采样：大数据集时使用采样，小数据集时全部显示
        const maxDisplay = 50; // 减少显示数量提升性能
        const displayAllocations = allocations.length > maxDisplay ? 
            this.sampleAllocations(allocations, maxDisplay) : 
            allocations.slice(0, maxDisplay);
        
        container.innerHTML = displayAllocations.map(alloc => `
            <div class="allocation-card" onclick="memscope.showAllocationDetails(${alloc.ptr})">
                <div class="allocation-header">
                    <span class="allocation-name">${alloc.var_name || `Ptr ${alloc.ptr.toString(16)}`}</span>
                    <span class="allocation-size">${this.formatBytes(alloc.size)}</span>
                </div>
                <div class="allocation-type">${this.getDisplayTypeName(alloc)}</div>
                <div class="allocation-details">
                    <div>Address: 0x${alloc.ptr.toString(16)}</div>
                    <div>Timestamp: ${new Date(alloc.timestamp / 1000000).toLocaleString()}</div>
                    ${alloc.call_stack && alloc.call_stack.length > 0 ? 
                        `<div>Stack depth: ${alloc.call_stack.length} frames</div>` : ''}
                </div>
            </div>
        `).join('');
        
        // Show count info
        if (allocations.length > maxDisplay) {
            const samplingInfo = allocations.length > maxDisplay ? 
                `Showing ${maxDisplay} sampled from ${allocations.length} allocations` :
                `Showing first ${maxDisplay} of ${allocations.length} allocations`;
            
            container.innerHTML += `
                <div style="grid-column: 1 / -1; text-align: center; padding: 20px; color: #64748b; font-style: italic;">
                    ${samplingInfo}
                    <button onclick="memscope.loadMoreAllocations()" style="margin-left: 10px; padding: 5px 10px; background: #3498db; color: white; border: none; border-radius: 4px; cursor: pointer;">
                        Load More
                    </button>
                </div>
            `;
        }
    }

    showAllocationDetails(ptr) {
        const alloc = this.data.allocations.find(a => a.ptr === ptr);
        if (!alloc) return;
        
        const details = `
            Variable: ${alloc.var_name || 'Unnamed'}
            Type: ${this.getDisplayTypeName(alloc)}
            Size: ${this.formatBytes(alloc.size)}
            Address: 0x${alloc.ptr.toString(16)}
            Timestamp: ${new Date(alloc.timestamp / 1000000).toLocaleString()}
            
            Call Stack:
            ${alloc.call_stack ? alloc.call_stack.map((frame, i) => 
                `  ${i + 1}. ${frame.function_name || 'unknown'} (${frame.file_name || 'unknown'}:${frame.line_number || '?'})`
            ).join('\n') : 'No call stack available'}
        `;
        
        alert(details); // Simple popup for now, could be enhanced with a modal
    }

    // ===========================================
    // DYNAMIC VISUALIZATION RENDERERS
    // ===========================================

    // Memory Analysis Dashboard 
    renderMemoryAnalysisDashboard() {
        const container = document.getElementById('memory-analysis');
        container.innerHTML = '';
        
        const dashboard = document.createElement('div');
        dashboard.className = 'memory-dashboard';
        dashboard.innerHTML = `
            <div class="dashboard-header">
                <h2>🧠 Dynamic Memory Analysis Dashboard</h2>
                <p>Interactive visualization of memory usage patterns</p>
            </div>
            <div class="dashboard-grid">
                <div class="metric-cards" id="metricCards"></div>
                <div class="memory-heatmap" id="memoryHeatmap"></div>
                <div class="type-distribution" id="typeDistribution"></div>
                <div class="fragmentation-analysis" id="fragmentationAnalysis"></div>
                <div class="categorized-allocations" id="categorizedAllocations"></div>
                <div class="callstack-analysis" id="callstackAnalysis"></div>
                <div class="memory-growth-trends" id="memoryGrowthTrends"></div>
                <div class="variable-timeline" id="variableTimeline"></div>
                <div class="interactive-legend" id="interactiveLegend"></div>
                <div class="comprehensive-summary" id="comprehensiveSummary"></div>
            </div>
        `;
        container.appendChild(dashboard);
        
        this.renderPerformanceMetrics();           
        this.renderMemoryHeatmap();               
        this.renderDynamicTypeDistribution();     
        this.renderFragmentationAnalysis();       
        this.renderCategorizedAllocations();      
        this.renderCallStackAnalysis();           
        this.renderMemoryGrowthTrends();          
        this.renderVariableTimeline();            
        this.renderInteractiveLegend();           
        this.renderComprehensiveSummary();        
    }

    renderPerformanceMetrics() {
        const container = document.getElementById('metricCards');
        const stats = this.data.stats;
        
        const currentMemory = stats.active_memory || 0;
        const peakMemory = stats.peak_memory || 0;
        const activeAllocations = stats.active_allocations || 0;
        
        const utilizationPercent = peakMemory > 0 ? Math.round((currentMemory / peakMemory) * 100) : 0;
        
        const totalAllocations = this.data.allocations.length;
        const memoryEfficiency = peakMemory > 0 ? Math.round((currentMemory / peakMemory) * 100) : 0;
        const avgAllocationSize = totalAllocations > 0 ? currentMemory / totalAllocations : 0;
        const fragmentation = peakMemory > 0 ? Math.round((1 - (currentMemory / peakMemory)) * 100) : 0;
        
        const allMetrics = [
            {
                label: 'Active Memory',
                value: this.formatBytes(currentMemory),
                percent: utilizationPercent,
                color: '#3498db',
                status: utilizationPercent > 80 ? 'HIGH' : utilizationPercent > 50 ? 'MEDIUM' : 'LOW',
                icon: '💾',
                showProgress: true
            },
            {
                label: 'Peak Memory',
                value: this.formatBytes(peakMemory),
                percent: 100,
                color: '#e74c3c',
                status: 'PEAK',
                icon: '📊',
                showProgress: false
            },
            {
                label: 'Memory Efficiency',
                value: `${memoryEfficiency}%`,
                percent: memoryEfficiency,
                color: '#f39c12',
                status: memoryEfficiency > 70 ? 'GOOD' : memoryEfficiency > 40 ? 'MEDIUM' : 'LOW',
                icon: '⚡',
                showProgress: true
            },
            {
                label: 'Active Allocs',
                value: activeAllocations.toLocaleString(),
                percent: Math.min(100, (activeAllocations / Math.max(1, totalAllocations)) * 100),
                color: '#2ecc71',
                status: 'ACTIVE',
                icon: '🔢',
                showProgress: false
            },
            {
                label: 'Fragmentation',
                value: `${fragmentation}%`,
                percent: fragmentation,
                color: '#95a5a6',
                status: fragmentation < 30 ? 'LOW' : fragmentation < 60 ? 'MEDIUM' : 'HIGH',
                icon: '🧩',
                showProgress: true
            },
            {
                label: 'Avg Alloc Size',
                value: this.formatBytes(avgAllocationSize),
                percent: Math.min(100, (avgAllocationSize / 1024) * 10),
                color: '#9b59b6',
                status: avgAllocationSize > 10240 ? 'LARGE' : avgAllocationSize > 1024 ? 'MEDIUM' : 'SMALL',
                icon: '📏',
                showProgress: false
            }
        ];
        
        container.innerHTML = `
            <div class="performance-dashboard">
                <div class="metrics-grid-unified">
                    ${allMetrics.map((metric, index) => `
                        <div class="metric-card unified" style="animation-delay: ${index * 0.1}s">
                            <div class="metric-header">
                                <div class="metric-icon" style="color: ${metric.color}">
                                    ${metric.icon}
                                </div>
                                <div class="metric-title">
                                    <h4>${metric.label}</h4>
                                    <div class="metric-status ${metric.status.toLowerCase()}">${metric.status}</div>
                                </div>
                            </div>
                            
                            <div class="metric-content">
                                <div class="metric-value-large" style="color: ${metric.color}">
                                    ${metric.value}
                                </div>
                                
                                ${metric.showProgress ? `
                                    <div class="progress-bar-container">
                                        <div class="progress-bar">
                                            <div class="progress-fill" 
                                                 style="width: ${metric.percent}%; background-color: ${metric.color};">
                                            </div>
                                        </div>
                                        <span class="progress-percent" style="color: ${metric.color}">
                                            ${Math.round(metric.percent)}%
                                        </span>
                                    </div>
                                ` : `
                                    <div class="metric-description">
                                        ${metric.label === 'Peak Memory' ? 'Maximum memory used' : 
                                          metric.label === 'Active Allocs' ? 'Current allocations' : 
                                          'Average allocation size'}
                                    </div>
                                `}
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
        
        // 触发动画
        setTimeout(() => {
            document.querySelectorAll('.progress-circle').forEach((circle, index) => {
                circle.style.strokeDashoffset = `${188.5 - (metrics[index].percent / 100) * 188.5}`;
            });
        }, 100);
    }

    // 交互式内存热力图
    renderMemoryHeatmap() {
        const container = document.getElementById('memoryHeatmap');
        container.innerHTML = `
            <div class="heatmap-header">
                <h3>📊 Memory Allocation Heatmap</h3>
                <div class="heatmap-description">
                    <p>Interactive visualization showing memory allocation patterns. Each block represents an allocation, colored by the selected criteria.</p>
                </div>
                <div class="heatmap-controls">
                    <button class="heatmap-btn active" data-view="size">By Size</button>
                    <button class="heatmap-btn" data-view="type">By Type</button>
                    <button class="heatmap-btn" data-view="time">By Time</button>
                </div>
            </div>
            <div class="heatmap-container">
                <div class="heatmap-canvas" id="heatmapCanvas"></div>
                <div class="heatmap-legend" id="heatmapLegend"></div>
            </div>
        `;
        
        // 创建热力图数据
        const allocations = this.data.allocations;
        const maxSize = Math.max(...allocations.map(a => a.size));
        
        const heatmapData = allocations.map((alloc, index) => ({
            x: (index % 20) * 25 + 10,
            y: Math.floor(index / 20) * 25 + 10,
            size: alloc.size,
            intensity: alloc.size / maxSize,
            color: this.getHeatmapColor(alloc.size / maxSize),
            allocation: alloc
        }));
        
        this.renderHeatmapCanvas(heatmapData);
        this.setupHeatmapControls();
    }

    // 渲染热力图画布
    renderHeatmapCanvas(data) {
        const canvas = document.getElementById('heatmapCanvas');
        
        // 计算更合适的网格尺寸
        const maxItems = Math.min(data.length, 200); // 限制显示数量避免过于密集
        const itemsPerRow = Math.ceil(Math.sqrt(maxItems * 1.5)); // 稍微宽一些的布局
        const rows = Math.ceil(maxItems / itemsPerRow);
        
        const cellSize = 18;
        const gap = 2;
        const svgWidth = itemsPerRow * (cellSize + gap) + gap;
        const svgHeight = rows * (cellSize + gap) + gap;
        
        const displayData = data.slice(0, maxItems);
        
        canvas.innerHTML = `
            <div class="heatmap-info">
                <span>Showing ${displayData.length} of ${data.length} allocations</span>
                <span>Total Memory: ${this.formatBytes(data.reduce((sum, d) => sum + d.size, 0))}</span>
            </div>
            <svg width="100%" height="${svgHeight + 40}" viewBox="0 0 ${svgWidth} ${svgHeight + 40}" class="heatmap-svg">
                <defs>
                    <filter id="cellShadow">
                        <feDropShadow dx="1" dy="1" stdDeviation="1" flood-opacity="0.3"/>
                    </filter>
                </defs>
                ${displayData.map((point, index) => {
                    const row = Math.floor(index / itemsPerRow);
                    const col = index % itemsPerRow;
                    const x = col * (cellSize + gap) + gap;
                    const y = row * (cellSize + gap) + gap;
                    
                    return `
                        <rect 
                            x="${x}" y="${y}" 
                            width="${cellSize}" height="${cellSize}" 
                            fill="${point.color}" 
                            opacity="${0.4 + point.intensity * 0.6}"
                            class="heatmap-cell"
                            data-index="${index}"
                            filter="url(#cellShadow)"
                            rx="2"
                            style="cursor: pointer;"
                        />
                    `;
                }).join('')}
                
                <!-- 添加标题 -->
                <text x="${svgWidth/2}" y="${svgHeight + 25}" text-anchor="middle" font-size="12" fill="#7f8c8d">
                    Hover over blocks to see allocation details
                </text>
            </svg>
        `;
        
        // 添加悬停交互 - 修复闪烁问题
        document.querySelectorAll('.heatmap-cell').forEach((cell, index) => {
            const allocation = data[index].allocation;
            const originalOpacity = 0.3 + data[index].intensity * 0.7;
            
            // 使用更稳定的悬停效果
            cell.addEventListener('mouseenter', (e) => {
                // 移除过渡效果避免闪烁
                cell.style.transition = 'none';
                cell.style.opacity = '0.95';
                cell.style.stroke = '#2c3e50';
                cell.style.strokeWidth = '2';
                
                this.showTooltip(e, {
                    title: allocation.var_name || `Allocation ${allocation.ptr.toString(16)}`,
                    size: this.formatBytes(allocation.size),
                    type: this.getDisplayTypeName(allocation),
                    timestamp: new Date(allocation.timestamp / 1000000).toLocaleString()
                });
            });
            
            cell.addEventListener('mouseleave', () => {
                // 恢复原始状态
                cell.style.transition = 'all 0.2s ease';
                cell.style.opacity = originalOpacity;
                cell.style.stroke = 'none';
                cell.style.strokeWidth = '0';
                this.hideTooltip();
            });
        });
    }

    // 动态类型分布图
    renderDynamicTypeDistribution() {
        const container = document.getElementById('typeDistribution');
        const typeMap = new Map();
        
        // 聚合类型数据
        this.data.allocations.forEach(alloc => {
            const typeName = this.getDisplayTypeName(alloc);
            if (!typeMap.has(typeName)) {
                typeMap.set(typeName, { size: 0, count: 0, color: this.getTypeColor(typeName) });
            }
            const current = typeMap.get(typeName);
            current.size += alloc.size;
            current.count += 1;
        });
        
        const sortedTypes = Array.from(typeMap.entries())
            .sort((a, b) => b[1].size - a[1].size)
            .slice(0, 8);
        
        const maxSize = sortedTypes[0]?.[1].size || 1;
        
        container.innerHTML = `
            <div class="type-dist-header">
                <h3>🏷️ Dynamic Type Distribution</h3>
                <div class="view-toggle">
                    <button class="toggle-btn active" data-view="bar">Bar Chart</button>
                    <button class="toggle-btn" data-view="pie">Pie Chart</button>
                </div>
            </div>
            <div class="type-chart" id="typeChart">
                <svg width="400" height="250" viewBox="0 0 400 250" class="type-svg">
                    ${sortedTypes.map((type, index) => {
                        const [typeName, data] = type;
                        const barHeight = (data.size / maxSize) * 180;
                        const x = 40 + index * 45;
                        const y = 200 - barHeight;
                        
                        return `
                            <g class="type-bar-group" data-type="${typeName}">
                                <rect 
                                    x="${x}" y="${y}" 
                                    width="35" height="${barHeight}"
                                    fill="${data.color}" 
                                    class="type-bar"
                                    style="transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1); cursor: pointer;"
                                />
                                <text x="${x + 17.5}" y="220" text-anchor="middle" font-size="10" fill="#2c3e50">
                                    ${this.truncateText(typeName, 8)}
                                </text>
                                <text x="${x + 17.5}" y="${y - 5}" text-anchor="middle" font-size="9" fill="${data.color}" font-weight="bold">
                                    ${data.count}
                                </text>
                            </g>
                        `;
                    }).join('')}
                </svg>
            </div>
        `;
        
        // 添加交互效果
        this.setupTypeDistributionInteractions(sortedTypes);
    }

    // Lifecycle Timeline (替换静态SVG)
    renderLifecycleTimeline() {
        const container = document.getElementById('lifecycle');
        container.innerHTML = '';
        
        const timeline = document.createElement('div');
        timeline.className = 'lifecycle-timeline';
        timeline.innerHTML = `
            <div class="timeline-header">
                <h2>⏱️ Dynamic Scope Matrix & Lifecycle</h2>
                <div class="timeline-controls">
                    <button class="timeline-btn" id="playBtn">▶️ Play</button>
                    <button class="timeline-btn" id="pauseBtn">⏸️ Pause</button>
                    <button class="timeline-btn" id="resetBtn">🔄 Reset</button>
                    <input type="range" id="timelineSlider" min="0" max="100" value="0" class="timeline-slider">
                </div>
            </div>
            <div class="scope-matrix" id="scopeMatrix"></div>
            <div class="variable-relationships" id="variableRelationships"></div>
        `;
        container.appendChild(timeline);
        
        this.renderScopeMatrix();
        this.renderVariableRelationships();
        this.setupTimelineControls();
    }

    // Unsafe FFI Dashboard (替换静态SVG)
    renderUnsafeFFIDashboard() {
        const container = document.getElementById('unsafe-ffi');
        
        if (!this.data.unsafeFFI || !this.data.unsafeFFI.allocations || this.data.unsafeFFI.allocations.length === 0) {
            container.innerHTML = `
                <div class="empty-state enhanced">
                    <div class="empty-icon">🛡️</div>
                    <h3>No Unsafe/FFI Data Available</h3>
                    <p>This analysis did not detect any unsafe Rust code or FFI operations.</p>
                    <p>This is generally a good sign for memory safety! 🎉</p>
                    <div class="safety-score">
                        <div class="score-circle">
                            <span class="score">100</span>
                            <span class="score-label">Safety Score</span>
                        </div>
                    </div>
                </div>
            `;
            return;
        }
        
        container.innerHTML = '';
        const dashboard = document.createElement('div');
        dashboard.className = 'unsafe-ffi-dashboard';
        dashboard.innerHTML = `
            <div class="ffi-header">
                <h2>⚠️ Dynamic Unsafe/FFI Analysis</h2>
                <div class="safety-alert ${this.data.unsafeFFI.violations.length > 0 ? 'danger' : 'safe'}">
                    ${this.data.unsafeFFI.violations.length > 0 ? '🚨 Safety Issues Detected' : '✅ No Safety Issues'}
                </div>
            </div>
            <div class="ffi-metrics" id="ffiMetrics"></div>
            <div class="ffi-flow" id="ffiFlow"></div>
            <div class="ffi-hotspots" id="ffiHotspots"></div>
        `;
        container.appendChild(dashboard);
        
        this.renderFFIMetrics();
        this.renderFFIFlow();
        this.renderFFIHotspots();
    }

    // Utility Functions
    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }

    truncateText(text, maxLength) {
        if (!text) return 'Unknown';
        return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
    }

    // 智能采样算法 - 保持数据代表性
    sampleAllocations(allocations, maxCount) {
        if (allocations.length <= maxCount) return allocations;
        
        // 分层采样：确保大小、类型、时间的代表性
        const step = Math.floor(allocations.length / maxCount);
        const sampled = [];
        
        for (let i = 0; i < allocations.length && sampled.length < maxCount; i += step) {
            sampled.push(allocations[i]);
        }
        
        // 确保包含最大和最小的分配
        const sortedBySize = [...allocations].sort((a, b) => b.size - a.size);
        if (!sampled.includes(sortedBySize[0])) {
            sampled[0] = sortedBySize[0]; // 最大的
        }
        if (!sampled.includes(sortedBySize[sortedBySize.length - 1])) {
            sampled[sampled.length - 1] = sortedBySize[sortedBySize.length - 1]; // 最小的
        }
        
        return sampled;
    }

    // 渲染预处理的类型分布数据
    renderPrecomputedTypeDistribution(container, typeDistribution) {
        const sortedTypes = typeDistribution.slice(0, 10);
        
        container.innerHTML = sortedTypes.map(([typeName, data]) => `
            <div class="type-item">
                <span class="type-name">${this.truncateText(typeName, 25)}</span>
                <div class="type-stats">
                    <span class="type-size">${this.formatBytes(data[0])}</span>
                    <span class="type-count">${data[1]} allocs</span>
                </div>
            </div>
        `).join('');
    }

    // 渲染预处理的性能洞察
    renderPrecomputedInsights(container, metrics) {
        const insights = [
            {
                title: `📊 Memory Utilization: ${metrics.utilization_percent}%`,
                description: `Efficiency level: ${metrics.efficiency_score}`
            },
            {
                title: `📏 Average Allocation: ${this.formatBytes(metrics.avg_allocation_size)}`,
                description: `Fragmentation: ${metrics.fragmentation_score}`
            }
        ];
        
        if (metrics.large_allocations_count > 0) {
            insights.push({
                title: `🔍 Large Allocations: ${metrics.large_allocations_count}`,
                description: 'Consider reviewing allocations > 1MB'
            });
        }
        
        // 显示优化信息
        if (this.data.precomputed && this.data.precomputed.is_sampled) {
            insights.push({
                title: `⚡ Data Optimized`,
                description: `Showing ${this.data.precomputed.optimization_info.sampling_ratio} of data for faster loading`
            });
        }
        
        container.innerHTML = insights.map(insight => `
            <div class="insight-item">
                <div class="insight-title">${insight.title}</div>
                <div class="insight-description">${insight.description}</div>
            </div>
        `).join('');
    }

    // 获取显示用的类型名称 - 智能推断，避免显示Unknown
    getDisplayTypeName(alloc) {
        let typeName = alloc.type_name;
        
        // 如果类型名为空或Unknown，进行智能推断
        if (!typeName || typeName === 'Unknown' || typeName === null || typeName === '') {
            // 优先基于变量名推断
            if (alloc.var_name && alloc.var_name !== 'unknown') {
                const varName = alloc.var_name.toLowerCase();
                if (varName.includes('vec') || varName.includes('vector')) {
                    return 'Vec<T>';
                } else if (varName.includes('string') || varName.includes('str')) {
                    return 'String';
                } else if (varName.includes('map') || varName.includes('hash')) {
                    return 'HashMap<K,V>';
                } else if (varName.includes('box')) {
                    return 'Box<T>';
                } else if (varName.includes('rc')) {
                    return 'Rc<T>';
                } else if (varName.includes('arc')) {
                    return 'Arc<T>';
                } else if (varName.includes('buffer') || varName.includes('buf')) {
                    return 'Buffer';
                } else {
                    // 基于大小推断
                    if (alloc.size <= 8) {
                        return 'Primitive';
                    } else if (alloc.size <= 64) {
                        return 'Small Struct';
                    } else if (alloc.size <= 1024) {
                        return 'Medium Struct';
                    } else {
                        return 'Large Object';
                    }
                }
            } else {
                // 基于调用栈推断
                if (alloc.call_stack && alloc.call_stack.length > 0) {
                    const topFrame = alloc.call_stack[0];
                    if (topFrame.function_name) {
                        const funcName = topFrame.function_name.toLowerCase();
                        if (funcName.includes('vec')) return 'Vec<T>';
                        if (funcName.includes('string')) return 'String';
                        if (funcName.includes('alloc')) return 'Raw Allocation';
                    }
                }
                
                // 最后基于大小推断
                if (alloc.size <= 8) {
                    return 'Primitive';
                } else if (alloc.size <= 32) {
                    return 'Small Object';
                } else if (alloc.size <= 1024) {
                    return 'Medium Object';
                } else if (alloc.size <= 1048576) {
                    return 'Large Buffer';
                } else {
                    return 'Huge Object';
                }
            }
        }
        
        return typeName;
    }

    loadMoreAllocations() {
        // 实现加载更多功能
        console.log('Loading more allocations...');
        // 这里可以实现分页加载
    }

    // ===========================================
    // UTILITY FUNCTIONS FOR DYNAMIC VISUALIZATIONS
    // ===========================================

    getHeatmapColor(intensity) {
        // 从蓝色到红色的渐变
        const colors = [
            '#3498db', '#2ecc71', '#f1c40f', '#e67e22', '#e74c3c'
        ];
        const index = Math.floor(intensity * (colors.length - 1));
        return colors[Math.min(index, colors.length - 1)];
    }

    getTypeColor(typeName) {
        // 处理空值情况
        if (!typeName || typeName === 'Unknown' || typeName === null || typeName === '') {
            return '#95a5a6'; // 灰色表示未知
        }
        
        // 扩展的颜色映射 - 覆盖更多Rust类型
        const colors = {
            // 集合类型
            'Vec<T>': '#3498db',
            'Vec': '#3498db',
            'vector': '#3498db',
            'Array': '#2980b9',
            'HashMap<K,V>': '#9b59b6',
            'HashMap': '#9b59b6',
            'BTreeMap': '#8e44ad',
            'HashSet': '#e67e22',
            
            // 字符串类型
            'String': '#2ecc71',
            'str': '#27ae60',
            '&str': '#27ae60',
            
            // 智能指针
            'Box<T>': '#e74c3c',
            'Box': '#e74c3c',
            'Rc<T>': '#f39c12',
            'Rc': '#f39c12',
            'Arc<T>': '#d35400',
            'Arc': '#d35400',
            'RefCell': '#e67e22',
            
            // 基础类型
            'Primitive': '#1abc9c',
            'Small Value': '#16a085',
            'i32': '#1abc9c',
            'i64': '#1abc9c',
            'u32': '#16a085',
            'u64': '#16a085',
            'f32': '#17a2b8',
            'f64': '#17a2b8',
            'bool': '#6f42c1',
            
            // 结构体类型
            'Struct': '#34495e',
            'Small Struct': '#2c3e50',
            'Medium Struct': '#34495e',
            'Complex Data': '#5d6d7e',
            
            // 缓冲区类型
            'Buffer': '#f1c40f',
            'Large Buffer': '#f39c12',
            'Huge Object': '#e74c3c',
            'Raw Allocation': '#95a5a6',
            
            // 推断类型
            'Inferred Type': '#7f8c8d',
            'Medium Object': '#85929e',
            'Large Structure': '#566573',
            'Buffer/Collection': '#f4d03f'
        };
        
        // 精确匹配
        if (colors[typeName]) {
            return colors[typeName];
        }
        
        // 部分匹配 - 按优先级排序
        const partialMatches = [
            ['Vec', '#3498db'],
            ['String', '#2ecc71'],
            ['Box', '#e74c3c'],
            ['Rc', '#f39c12'],
            ['Arc', '#d35400'],
            ['HashMap', '#9b59b6'],
            ['Map', '#8e44ad'],
            ['Set', '#e67e22'],
            ['Buffer', '#f1c40f'],
            ['Primitive', '#1abc9c'],
            ['Struct', '#34495e'],
            ['Data', '#5d6d7e']
        ];
        
        for (const [pattern, color] of partialMatches) {
            if (typeName.includes(pattern)) {
                return color;
            }
        }
        
        // 为其他类型生成一致的颜色（基于类型名哈希）
        let hash = 0;
        for (let i = 0; i < typeName.length; i++) {
            hash = typeName.charCodeAt(i) + ((hash << 5) - hash);
        }
        const hue = Math.abs(hash) % 360;
        return `hsl(${hue}, 65%, 55%)`; // 稍微调整饱和度和亮度
    }

    getTimeColor(intensity) {
        // 从紫色到黄色的时间渐变
        const colors = [
            '#9b59b6', // 早期 - 紫色
            '#3498db', // 中早期 - 蓝色
            '#1abc9c', // 中期 - 青色
            '#f1c40f', // 中晚期 - 黄色
            '#e67e22'  // 晚期 - 橙色
        ];
        const index = Math.floor(intensity * (colors.length - 1));
        return colors[Math.min(index, colors.length - 1)];
    }

    updateHeatmapLegend(view) {
        const container = document.getElementById('heatmapLegend');
        
        let legendContent = '';
        switch(view) {
            case 'size':
                legendContent = `
                    <div class="legend-title">Size Legend</div>
                    <div class="legend-items">
                        <div class="legend-item"><span class="legend-color" style="background: #3498db"></span>Small</div>
                        <div class="legend-item"><span class="legend-color" style="background: #2ecc71"></span>Medium</div>
                        <div class="legend-item"><span class="legend-color" style="background: #f1c40f"></span>Large</div>
                        <div class="legend-item"><span class="legend-color" style="background: #e67e22"></span>Very Large</div>
                        <div class="legend-item"><span class="legend-color" style="background: #e74c3c"></span>Huge</div>
                    </div>
                `;
                break;
            case 'type':
                legendContent = `
                    <div class="legend-title">Type Legend</div>
                    <div class="legend-items">
                        <div class="legend-item"><span class="legend-color" style="background: #3498db"></span>Vec</div>
                        <div class="legend-item"><span class="legend-color" style="background: #2ecc71"></span>String</div>
                        <div class="legend-item"><span class="legend-color" style="background: #e74c3c"></span>Box</div>
                        <div class="legend-item"><span class="legend-color" style="background: #9b59b6"></span>HashMap</div>
                        <div class="legend-item"><span class="legend-color" style="background: #95a5a6"></span>Other</div>
                    </div>
                `;
                break;
            case 'time':
                legendContent = `
                    <div class="legend-title">Time Legend</div>
                    <div class="legend-items">
                        <div class="legend-item"><span class="legend-color" style="background: #9b59b6"></span>Early</div>
                        <div class="legend-item"><span class="legend-color" style="background: #3498db"></span>Mid-Early</div>
                        <div class="legend-item"><span class="legend-color" style="background: #1abc9c"></span>Middle</div>
                        <div class="legend-item"><span class="legend-color" style="background: #f1c40f"></span>Mid-Late</div>
                        <div class="legend-item"><span class="legend-color" style="background: #e67e22"></span>Late</div>
                    </div>
                `;
                break;
        }
        
        container.innerHTML = legendContent;
    }

    showTooltip(event, data) {
        let tooltip = document.getElementById('dynamicTooltip');
        if (!tooltip) {
            tooltip = document.createElement('div');
            tooltip.id = 'dynamicTooltip';
            tooltip.className = 'dynamic-tooltip';
            document.body.appendChild(tooltip);
        }
        
        tooltip.innerHTML = `
            <div class="tooltip-header">${data.title}</div>
            <div class="tooltip-content">
                <div><strong>Size:</strong> ${data.size}</div>
                <div><strong>Type:</strong> ${data.type}</div>
                <div><strong>Time:</strong> ${data.timestamp}</div>
            </div>
        `;
        
        tooltip.style.display = 'block';
        tooltip.style.left = event.pageX + 10 + 'px';
        tooltip.style.top = event.pageY + 10 + 'px';
    }

    hideTooltip() {
        const tooltip = document.getElementById('dynamicTooltip');
        if (tooltip) {
            tooltip.style.display = 'none';
        }
    }

    setupHeatmapControls() {
        document.querySelectorAll('.heatmap-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.heatmap-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                
                const view = e.target.dataset.view;
                this.updateHeatmapView(view);
            });
        });
    }

    updateHeatmapView(view) {
        const allocations = this.data.allocations;
        let heatmapData;
        
        switch(view) {
            case 'size':
                const maxSize = Math.max(...allocations.map(a => a.size));
                heatmapData = allocations.map((alloc, index) => ({
                    x: (index % 20) * 25 + 10,
                    y: Math.floor(index / 20) * 25 + 10,
                    size: alloc.size,
                    intensity: alloc.size / maxSize,
                    color: this.getHeatmapColor(alloc.size / maxSize),
                    allocation: alloc
                }));
                break;
                
            case 'type':
                const typeColors = new Map();
                const uniqueTypes = [...new Set(allocations.map(a => this.getDisplayTypeName(a)))];
                uniqueTypes.forEach((type, index) => {
                    typeColors.set(type, this.getTypeColor(type));
                });
                
                heatmapData = allocations.map((alloc, index) => ({
                    x: (index % 20) * 25 + 10,
                    y: Math.floor(index / 20) * 25 + 10,
                    size: alloc.size,
                    intensity: 0.8, // 固定强度，主要看颜色
                    color: typeColors.get(alloc.type_name || 'Unknown'),
                    allocation: alloc
                }));
                break;
                
            case 'time':
                const timestamps = allocations.map(a => a.timestamp);
                const minTime = Math.min(...timestamps);
                const maxTime = Math.max(...timestamps);
                const timeRange = maxTime - minTime || 1;
                
                heatmapData = allocations.map((alloc, index) => {
                    const timeIntensity = (alloc.timestamp - minTime) / timeRange;
                    return {
                        x: (index % 20) * 25 + 10,
                        y: Math.floor(index / 20) * 25 + 10,
                        size: alloc.size,
                        intensity: timeIntensity,
                        color: this.getTimeColor(timeIntensity),
                        allocation: alloc
                    };
                });
                break;
                
            default:
                return;
        }
        
        this.renderHeatmapCanvas(heatmapData);
        this.updateHeatmapLegend(view);
    }

    setupTypeDistributionInteractions(types) {
        document.querySelectorAll('.type-bar').forEach((bar, index) => {
            const typeData = types[index][1];
            
            bar.addEventListener('mouseenter', () => {
                bar.style.transform = 'scaleY(1.1)';
                bar.style.filter = 'brightness(1.2)';
            });
            
            bar.addEventListener('mouseleave', () => {
                bar.style.transform = 'scaleY(1)';
                bar.style.filter = 'brightness(1)';
            });
            
            bar.addEventListener('click', () => {
                this.showTypeDetails(types[index]);
            });
        });
    }

    showTypeDetails(typeData) {
        const [typeName, data] = typeData;
        alert(`Type: ${typeName}\nAllocations: ${data.count}\nTotal Size: ${this.formatBytes(data.size)}`);
    }

    // 作用域矩阵渲染
    renderScopeMatrix() {
        const container = document.getElementById('scopeMatrix');
        const trackedVars = this.data.allocations.filter(a => a.var_name);
        
        if (trackedVars.length === 0) {
            container.innerHTML = '<div class="no-data">No tracked variables found</div>';
            return;
        }
        
        // 按作用域分组变量
        const scopes = this.groupVariablesByScope(trackedVars);
        
        container.innerHTML = Object.entries(scopes).map(([scopeName, vars]) => `
            <div class="scope-container" data-scope="${scopeName}">
                <div class="scope-header">
                    <h4>📦 ${scopeName}</h4>
                    <span class="scope-stats">${vars.length} variables</span>
                </div>
                <div class="scope-variables">
                    ${vars.map(v => `
                        <div class="variable-item">
                            <div class="var-name">${v.var_name}</div>
                            <div class="var-progress">
                                <div class="progress-bar" style="width: ${Math.random() * 100}%; background: ${this.getTypeColor(v.type_name || 'Unknown')}"></div>
                            </div>
                            <div class="var-size">${this.formatBytes(v.size)}</div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `).join('');
    }

    groupVariablesByScope(variables) {
        const scopes = {};
        variables.forEach(v => {
            const scope = this.extractScope(v);
            if (!scopes[scope]) scopes[scope] = [];
            scopes[scope].push(v);
        });
        return scopes;
    }

    extractScope(variable) {
        // 简单的作用域提取逻辑
        if (variable.var_name) {
            if (variable.var_name.includes('global')) return 'Global';
            if (variable.var_name.includes('main')) return 'Main Function';
            if (variable.var_name.includes('test')) return 'Test Scope';
        }
        return 'Local Scope';
    }

    renderVariableRelationships() {
        const container = document.getElementById('variableRelationships');
        container.innerHTML = `
            <div class="relationships-header">
                <h4>🔗 Variable Relationships</h4>
            </div>
            <div class="relationship-graph">
                <svg width="100%" height="200" viewBox="0 0 500 200">
                    <!-- 这里可以添加变量关系的连线图 -->
                    <text x="250" y="100" text-anchor="middle" fill="#64748b">
                        Relationship analysis coming soon...
                    </text>
                </svg>
            </div>
        `;
    }

    setupTimelineControls() {
        // 时间轴控制逻辑
        document.getElementById('playBtn')?.addEventListener('click', () => {
            console.log('Timeline play');
        });
        
        document.getElementById('pauseBtn')?.addEventListener('click', () => {
            console.log('Timeline pause');
        });
        
        document.getElementById('resetBtn')?.addEventListener('click', () => {
            console.log('Timeline reset');
        });
    }

    renderFFIMetrics() {
        const container = document.getElementById('ffiMetrics');
        const ffiData = this.data.unsafeFFI;
        
        container.innerHTML = `
            <div class="ffi-metric-cards">
                <div class="ffi-card danger">
                    <div class="card-value">${ffiData.violations.length}</div>
                    <div class="card-label">Safety Violations</div>
                </div>
                <div class="ffi-card warning">
                    <div class="card-value">${ffiData.allocations.length}</div>
                    <div class="card-label">Unsafe Allocations</div>
                </div>
                <div class="ffi-card info">
                    <div class="card-value">${ffiData.boundaryEvents.length}</div>
                    <div class="card-label">Boundary Events</div>
                </div>
            </div>
        `;
    }

    renderFFIFlow() {
        const container = document.getElementById('ffiFlow');
        container.innerHTML = `
            <div class="flow-diagram">
                <h4>🔄 Memory Flow Analysis</h4>
                <div class="flow-visualization">
                    <!-- 动态流程图将在这里渲染 -->
                    <div class="flow-placeholder">Interactive flow diagram coming soon...</div>
                </div>
            </div>
        `;
    }

    renderFFIHotspots() {
        const container = document.getElementById('ffiHotspots');
        container.innerHTML = `
            <div class="hotspots-map">
                <h4>🔥 Memory Hotspots</h4>
                <div class="hotspot-visualization">
                    <!-- 热点气泡图将在这里渲染 -->
                    <div class="hotspot-placeholder">Hotspot visualization coming soon...</div>
                </div>
            </div>
        `;
    }

    // ===========================================
    // 完整12个模块实现 (对应原始SVG)
    // ===========================================

    // 模块5: 内存碎片化分析
    renderFragmentationAnalysis() {
        const container = document.getElementById('fragmentationAnalysis');
        const allocations = this.data.allocations;
        
        // 计算碎片化指标
        const totalMemory = allocations.reduce((sum, a) => sum + a.size, 0);
        const avgSize = totalMemory / allocations.length || 0;
        const sizeVariance = allocations.reduce((sum, a) => sum + Math.pow(a.size - avgSize, 2), 0) / allocations.length;
        const fragmentationScore = Math.min(100, (sizeVariance / (avgSize * avgSize)) * 100);
        
        container.innerHTML = `
            <div class="analysis-header">
                <h3>🧩 Memory Fragmentation Analysis</h3>
                <div class="fragmentation-score ${fragmentationScore > 70 ? 'high' : fragmentationScore > 40 ? 'medium' : 'low'}">
                    ${fragmentationScore.toFixed(1)}% Fragmented
                </div>
            </div>
            <div class="fragmentation-visual">
                <div class="memory-blocks" id="memoryBlocks"></div>
                <div class="fragmentation-metrics">
                    <div class="metric-item">
                        <span class="metric-label">Average Size:</span>
                        <span class="metric-value">${this.formatBytes(avgSize)}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">Size Variance:</span>
                        <span class="metric-value">${this.formatBytes(Math.sqrt(sizeVariance))}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">Total Blocks:</span>
                        <span class="metric-value">${allocations.length}</span>
                    </div>
                </div>
            </div>
        `;
        
        this.renderMemoryBlocks(allocations);
    }

    renderMemoryBlocks(allocations) {
        const container = document.getElementById('memoryBlocks');
        const maxSize = Math.max(...allocations.map(a => a.size));
        
        // 创建内存块可视化
        const blocks = allocations.slice(0, 20).map((alloc, index) => {
            const width = Math.max(10, (alloc.size / maxSize) * 100);
            const height = 15;
            const color = this.getTypeColor(alloc.type_name || 'Unknown');
            
            return `
                <div class="memory-block" 
                     style="width: ${width}px; height: ${height}px; background: ${color}; margin: 2px;"
                     title="${alloc.var_name || 'Unknown'}: ${this.formatBytes(alloc.size)}">
                </div>
            `;
        }).join('');
        
        container.innerHTML = `<div class="blocks-container">${blocks}</div>`;
    }

    // 模块6: 分类分配
    renderCategorizedAllocations() {
        const container = document.getElementById('categorizedAllocations');
        const allocations = this.data.allocations;
        
        // 按大小分类
        const categories = {
            'Small (< 1KB)': allocations.filter(a => a.size < 1024),
            'Medium (1KB - 100KB)': allocations.filter(a => a.size >= 1024 && a.size < 102400),
            'Large (100KB - 1MB)': allocations.filter(a => a.size >= 102400 && a.size < 1048576),
            'Huge (> 1MB)': allocations.filter(a => a.size >= 1048576)
        };
        
        container.innerHTML = `
            <div class="categories-header">
                <h3>📂 Categorized Allocations</h3>
                <div class="category-toggle">
                    <button class="cat-btn active" data-cat="size">By Size</button>
                    <button class="cat-btn" data-cat="type">By Type</button>
                </div>
            </div>
            <div class="categories-list" id="categoriesList"></div>
        `;
        
        this.renderCategoryList(categories);
        this.setupCategoryToggle();
    }

    renderCategoryList(categories) {
        const container = document.getElementById('categoriesList');
        
        container.innerHTML = Object.entries(categories).map(([name, allocs]) => {
            const totalSize = allocs.reduce((sum, a) => sum + a.size, 0);
            const percentage = (allocs.length / this.data.allocations.length * 100).toFixed(1);
            
            return `
                <div class="category-item">
                    <div class="category-header">
                        <span class="category-name">${name}</span>
                        <span class="category-count">${allocs.length} (${percentage}%)</span>
                    </div>
                    <div class="category-bar">
                        <div class="bar-fill" style="width: ${percentage}%; background: ${this.getCategoryColor(name)}"></div>
                    </div>
                    <div class="category-size">${this.formatBytes(totalSize)}</div>
                </div>
            `;
        }).join('');
    }

    // 模块7: 调用栈分析
    renderCallStackAnalysis() {
        const container = document.getElementById('callstackAnalysis');
        const allocations = this.data.allocations.filter(a => a.call_stack && a.call_stack.length > 0);
        
        if (allocations.length === 0) {
            container.innerHTML = `
                <div class="analysis-header">
                    <h3>📞 Call Stack Analysis</h3>
                </div>
                <div class="no-callstack">No call stack information available</div>
            `;
            return;
        }
        
        // 分析调用栈深度
        const stackDepths = allocations.map(a => a.call_stack.length);
        const avgDepth = stackDepths.reduce((sum, d) => sum + d, 0) / stackDepths.length;
        const maxDepth = Math.max(...stackDepths);
        
        // 统计常见函数
        const functionCounts = new Map();
        allocations.forEach(a => {
            a.call_stack.forEach(frame => {
                const funcName = frame.function_name || 'unknown';
                functionCounts.set(funcName, (functionCounts.get(funcName) || 0) + 1);
            });
        });
        
        const topFunctions = Array.from(functionCounts.entries())
            .sort((a, b) => b[1] - a[1])
            .slice(0, 8);
        
        container.innerHTML = `
            <div class="analysis-header">
                <h3>📞 Call Stack Analysis</h3>
                <div class="stack-stats">
                    <span>Avg Depth: ${avgDepth.toFixed(1)}</span>
                    <span>Max Depth: ${maxDepth}</span>
                </div>
            </div>
            <div class="callstack-visual">
                <div class="depth-distribution" id="depthDistribution"></div>
                <div class="top-functions">
                    <h4>Top Functions</h4>
                    ${topFunctions.map(([func, count]) => `
                        <div class="function-item">
                            <span class="func-name">${this.truncateText(func, 20)}</span>
                            <span class="func-count">${count}</span>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
        
        this.renderDepthDistribution(stackDepths);
    }

    renderDepthDistribution(depths) {
        const container = document.getElementById('depthDistribution');
        const maxDepth = Math.max(...depths);
        const depthCounts = new Array(maxDepth + 1).fill(0);
        
        depths.forEach(depth => depthCounts[depth]++);
        const maxCount = Math.max(...depthCounts);
        
        container.innerHTML = `
            <h4>Stack Depth Distribution</h4>
            <div class="depth-bars">
                ${depthCounts.map((count, depth) => {
                    const height = count > 0 ? (count / maxCount * 60) : 0;
                    return `
                        <div class="depth-bar" style="height: ${height}px" title="Depth ${depth}: ${count} allocations">
                            <span class="depth-label">${depth}</span>
                        </div>
                    `;
                }).join('')}
            </div>
        `;
    }

    // 模块8: 内存增长趋势
    renderMemoryGrowthTrends() {
        const container = document.getElementById('memoryGrowthTrends');
        const allocations = this.data.allocations.sort((a, b) => a.timestamp - b.timestamp);
        
        container.innerHTML = `
            <div class="trends-header">
                <h3>📈 Memory Growth Trends</h3>
                <div class="trend-controls">
                    <button class="trend-btn active" data-trend="cumulative">Cumulative</button>
                    <button class="trend-btn" data-trend="rate">Growth Rate</button>
                </div>
            </div>
            <div class="trends-chart" id="trendsChart"></div>
        `;
        
        this.renderTrendsChart(allocations);
        this.setupTrendControls();
    }

    renderTrendsChart(allocations) {
        const container = document.getElementById('trendsChart');
        
        if (allocations.length === 0) {
            container.innerHTML = '<div class="no-data">No allocation data for trends</div>';
            return;
        }
        
        // 计算累积内存使用
        let cumulativeMemory = 0;
        const dataPoints = allocations.map((alloc, index) => {
            cumulativeMemory += alloc.size;
            return {
                x: index,
                y: cumulativeMemory,
                timestamp: alloc.timestamp
            };
        });
        
        const maxMemory = Math.max(...dataPoints.map(p => p.y));
        
        // 大幅增加图表尺寸，让它更加突出
        const chartWidth = 600;
        const chartHeight = 280;
        const margin = { top: 20, right: 40, bottom: 60, left: 60 };
        const innerWidth = chartWidth - margin.left - margin.right;
        const innerHeight = chartHeight - margin.top - margin.bottom;
        
        // 修复路径计算，确保不越界，添加边距
        const pathData = dataPoints.map((point, index) => {
            const x = dataPoints.length > 1 ? 
                margin.left + (point.x / (dataPoints.length - 1)) * innerWidth : 
                margin.left + innerWidth / 2;
            const y = maxMemory > 0 ? 
                margin.top + innerHeight - (point.y / maxMemory) * innerHeight : 
                margin.top + innerHeight / 2;
            
            // 确保坐标在有效范围内
            const safeX = Math.max(margin.left, Math.min(margin.left + innerWidth, x));
            const safeY = Math.max(margin.top, Math.min(margin.top + innerHeight, y));
            
            return index === 0 ? `M ${safeX} ${safeY}` : `L ${safeX} ${safeY}`;
        }).join(' ');
        
        // 生成网格线
        const gridLines = [];
        for (let i = 0; i <= 5; i++) {
            const y = margin.top + (i / 5) * innerHeight;
            const value = maxMemory * (1 - i / 5);
            gridLines.push(`
                <line x1="${margin.left}" y1="${y}" x2="${margin.left + innerWidth}" y2="${y}" 
                      stroke="#ecf0f1" stroke-width="1"/>
                <text x="${margin.left - 10}" y="${y + 4}" text-anchor="end" font-size="10" fill="#7f8c8d">
                    ${this.formatBytes(value)}
                </text>
            `);
        }
        
        container.innerHTML = `
            <div class="chart-container">
                <svg width="100%" height="${chartHeight + 40}" viewBox="0 0 ${chartWidth} ${chartHeight + 40}" class="trends-svg">
                    <defs>
                        <linearGradient id="trendGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                            <stop offset="0%" style="stop-color:#3498db;stop-opacity:0.6" />
                            <stop offset="100%" style="stop-color:#3498db;stop-opacity:0.1" />
                        </linearGradient>
                        <filter id="dropShadow">
                            <feDropShadow dx="0" dy="2" stdDeviation="3" flood-color="rgba(52, 152, 219, 0.3)"/>
                        </filter>
                    </defs>
                    
                    <!-- 背景 -->
                    <rect x="${margin.left}" y="${margin.top}" width="${innerWidth}" height="${innerHeight}" 
                          fill="#f8fafc" stroke="#ecf0f1" stroke-width="1" rx="4"/>
                    
                    <!-- 网格线 -->
                    ${gridLines.join('')}
                    
                    <!-- 数据可视化 -->
                    ${dataPoints.length > 1 ? `
                        <!-- 填充区域 -->
                        <path d="${pathData} L ${margin.left + innerWidth} ${margin.top + innerHeight} L ${margin.left} ${margin.top + innerHeight} Z" 
                              fill="url(#trendGradient)" stroke="none"/>
                        
                        <!-- 趋势线 -->
                        <path d="${pathData}" fill="none" stroke="#3498db" stroke-width="3" 
                              filter="url(#dropShadow)" stroke-linecap="round"/>
                        
                        <!-- 数据点 -->
                        ${dataPoints.map((point, index) => {
                            const x = margin.left + (point.x / (dataPoints.length - 1)) * innerWidth;
                            const y = margin.top + innerHeight - (point.y / maxMemory) * innerHeight;
                            return `
                                <circle cx="${x}" cy="${y}" r="4" fill="#3498db" stroke="white" stroke-width="2"
                                        class="data-point" data-index="${index}"/>
                            `;
                        }).join('')}
                    ` : `
                        <circle cx="${margin.left + innerWidth/2}" cy="${margin.top + innerHeight/2}" r="8" 
                                fill="#3498db" stroke="white" stroke-width="3"/>
                        <text x="${margin.left + innerWidth/2}" y="${margin.top + innerHeight/2 + 30}" 
                              text-anchor="middle" font-size="12" fill="#7f8c8d">
                            Single allocation
                        </text>
                    `}
                    
                    <!-- 坐标轴标签 -->
                    <text x="${margin.left}" y="${chartHeight + 20}" font-size="12" fill="#7f8c8d">Start</text>
                    <text x="${margin.left + innerWidth}" y="${chartHeight + 20}" font-size="12" fill="#7f8c8d" text-anchor="end">Now</text>
                    
                    <!-- 标题 -->
                    <text x="${chartWidth/2}" y="15" text-anchor="middle" font-size="14" font-weight="600" fill="#2c3e50">
                        Memory Growth Over Time (Peak: ${this.formatBytes(maxMemory)})
                    </text>
                </svg>
            </div>
        `;
        
        // 添加数据点交互
        this.setupTrendsInteraction(dataPoints);
    }

    // 模块9: 变量分配时间轴
    renderVariableTimeline() {
        const container = document.getElementById('variableTimeline');
        const trackedVars = this.data.allocations.filter(a => a.var_name);
        
        container.innerHTML = `
            <div class="timeline-header">
                <h3>⏰ Variable Allocation Timeline</h3>
                <div class="timeline-info">
                    ${trackedVars.length} tracked variables
                </div>
            </div>
            <div class="timeline-visual" id="timelineVisual"></div>
        `;
        
        this.renderTimelineVisual(trackedVars);
    }

    renderTimelineVisual(variables) {
        const container = document.getElementById('timelineVisual');
        
        if (variables.length === 0) {
            container.innerHTML = '<div class="no-timeline">No tracked variables for timeline</div>';
            return;
        }
        
        const sortedVars = variables.sort((a, b) => a.timestamp - b.timestamp);
        const timelineWidth = 500;
        const itemHeight = 25;
        
        container.innerHTML = `
            <div class="timeline-container">
                ${sortedVars.slice(0, 15).map((variable, index) => {
                    const relativeTime = index / (sortedVars.length - 1);
                    const x = relativeTime * timelineWidth;
                    const color = this.getTypeColor(variable.type_name || 'Unknown');
                    
                    return `
                        <div class="timeline-item" style="top: ${index * itemHeight}px;">
                            <div class="timeline-dot" style="left: ${x}px; background: ${color}"></div>
                            <div class="timeline-label" style="left: ${x + 15}px;">
                                <span class="var-name">${variable.var_name}</span>
                                <span class="var-size">${this.formatBytes(variable.size)}</span>
                            </div>
                        </div>
                    `;
                }).join('')}
            </div>
        `;
    }

    // 模块10: 交互式图例
    renderInteractiveLegend() {
        const container = document.getElementById('interactiveLegend');
        
        const legendItems = [
            { color: '#3498db', label: 'Active Memory', description: 'Currently allocated memory' },
            { color: '#e74c3c', label: 'Peak Memory', description: 'Maximum memory usage' },
            { color: '#2ecc71', label: 'Safe Allocations', description: 'Memory-safe allocations' },
            { color: '#f39c12', label: 'Medium Priority', description: 'Moderate memory usage' },
            { color: '#9b59b6', label: 'Large Objects', description: 'Objects > 100KB' },
            { color: '#1abc9c', label: 'Small Objects', description: 'Objects < 1KB' }
        ];
        
        container.innerHTML = `
            <div class="legend-header">
                <h3>🎨 Interactive Legend & Guide</h3>
            </div>
            <div class="legend-grid">
                ${legendItems.map(item => `
                    <div class="legend-item" data-color="${item.color}">
                        <div class="legend-color" style="background: ${item.color}"></div>
                        <div class="legend-text">
                            <div class="legend-label">${item.label}</div>
                            <div class="legend-desc">${item.description}</div>
                        </div>
                    </div>
                `).join('')}
            </div>
        `;
        
        this.setupLegendInteractions();
    }

    // 模块11: 综合摘要
    renderComprehensiveSummary() {
        const container = document.getElementById('comprehensiveSummary');
        const stats = this.data.stats;
        const allocations = this.data.allocations;
        
        // 计算关键指标 - 安全计算避免NaN
        const currentMemory = stats.active_memory || 0;
        const peakMemory = stats.peak_memory || 0;
        const efficiency = peakMemory > 0 ? ((currentMemory / peakMemory) * 100).toFixed(1) : '0.0';
        
        const totalMemoryUsed = allocations.reduce((sum, a) => sum + (a.size || 0), 0);
        const avgSize = allocations.length > 0 ? (totalMemoryUsed / allocations.length) : 0;
        
        const trackedVars = allocations.filter(a => a.var_name && a.var_name !== 'Unknown').length;
        const trackedPercentage = allocations.length > 0 ? ((trackedVars / allocations.length) * 100).toFixed(1) : '0.0';
        
        container.innerHTML = `
            <div class="summary-header">
                <h3>📋 Comprehensive Memory Analysis Summary</h3>
            </div>
            <div class="summary-grid">
                <div class="summary-section">
                    <h4>Memory Efficiency</h4>
                    <div class="efficiency-meter">
                        <div class="meter-bar">
                            <div class="meter-fill" style="width: ${efficiency}%; background: ${efficiency > 80 ? '#e74c3c' : efficiency > 60 ? '#f39c12' : '#2ecc71'}"></div>
                        </div>
                        <span class="meter-value">${efficiency}%</span>
                    </div>
                </div>
                
                <div class="summary-section">
                    <h4>Key Metrics</h4>
                    <div class="metrics-list">
                        <div class="metric-row">
                            <span>Average Allocation Size:</span>
                            <span>${this.formatBytes(avgSize)}</span>
                        </div>
                        <div class="metric-row">
                            <span>Tracked Variables:</span>
                            <span>${trackedVars} (${trackedPercentage}%)</span>
                        </div>
                        <div class="metric-row">
                            <span>Memory Utilization:</span>
                            <span>${efficiency}%</span>
                        </div>
                    </div>
                </div>
                
                <div class="summary-section">
                    <h4>Recommendations</h4>
                    <div class="recommendations">
                        ${this.generateRecommendations(stats, allocations)}
                    </div>
                </div>
            </div>
        `;
    }

    generateRecommendations(stats, allocations) {
        const recommendations = [];
        const efficiency = (stats.active_memory / stats.peak_memory) * 100;
        
        if (efficiency > 80) {
            recommendations.push('⚠️ High memory utilization - consider optimization');
        }
        
        if (allocations.length > 1000) {
            recommendations.push('📊 Large number of allocations - consider pooling');
        }
        
        const largeAllocs = allocations.filter(a => a.size > 1048576).length;
        if (largeAllocs > 0) {
            recommendations.push(`🔍 ${largeAllocs} large allocations detected`);
        }
        
        if (recommendations.length === 0) {
            recommendations.push('✅ Memory usage appears optimal');
        }
        
        return recommendations.map(rec => `<div class="recommendation">${rec}</div>`).join('');
    }

    // 辅助函数
    getCategoryColor(categoryName) {
        const colors = {
            'Small': '#2ecc71',
            'Medium': '#3498db', 
            'Large': '#f39c12',
            'Huge': '#e74c3c'
        };
        
        for (const [key, color] of Object.entries(colors)) {
            if (categoryName.includes(key)) return color;
        }
        return '#95a5a6';
    }

    setupCategoryToggle() {
        document.querySelectorAll('.cat-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.cat-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                
                const category = e.target.dataset.cat;
                if (category === 'type') {
                    this.renderCategoriesByType();
                } else {
                    this.renderCategorizedAllocations();
                }
            });
        });
    }

    setupTrendControls() {
        document.querySelectorAll('.trend-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.trend-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                
                const trend = e.target.dataset.trend;
                console.log(`Switching to ${trend} trend view`);
            });
        });
    }

    setupLegendInteractions() {
        document.querySelectorAll('.legend-item').forEach(item => {
            item.addEventListener('click', () => {
                const color = item.dataset.color;
                this.highlightElementsByColor(color);
            });
        });
    }

    highlightElementsByColor(color) {
        // 高亮显示对应颜色的元素
        console.log(`Highlighting elements with color: ${color}`);
    }

    setupTrendsInteraction(dataPoints) {
        document.querySelectorAll('.data-point').forEach((point, index) => {
            const data = dataPoints[index];
            
            point.addEventListener('mouseenter', (e) => {
                point.setAttribute('r', '6');
                point.style.filter = 'brightness(1.2)';
                
                this.showTooltip(e, {
                    title: `Data Point ${index + 1}`,
                    size: this.formatBytes(data.y),
                    type: 'Cumulative Memory',
                    timestamp: new Date(data.timestamp / 1000000).toLocaleString()
                });
            });
            
            point.addEventListener('mouseleave', () => {
                point.setAttribute('r', '4');
                point.style.filter = 'none';
                this.hideTooltip();
            });
        });
    }
}

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    // Global instance for easy access
    window.memscope = new MemScopeVisualizer(MEMORY_DATA);
    
    // Add some debug info to console
    console.log('🔍 MemScope-RS Interactive Visualizer Loaded');
    console.log('📊 Data Summary:', {
        allocations: MEMORY_DATA.allocations.length,
        totalMemory: window.memscope.formatBytes(MEMORY_DATA.stats.active_memory),
        hasUnsafeFFI: !!MEMORY_DATA.unsafeFFI,
        timestamp: MEMORY_DATA.timestamp
    });
});
    // 🔧 Render Complex Types Analysis
    renderComplexTypesAnalysis() {
        const container = document.getElementById("complex-types");
        const complexTypesData = this.data.complex_types || {};
        
        container.innerHTML = `
            <div class="complex-types-dashboard">
                <h2>🔧 Complex Types Analysis</h2>
                <div class="complex-types-grid">
                    <div class="complex-type-card">
                        <h3>📊 Type Categories</h3>
                        <div id="typeCategoriesChart"></div>
                    </div>
                    <div class="complex-type-card">
                        <h3>🎯 Complexity Distribution</h3>
                        <div id="complexityDistribution"></div>
                    </div>
                    <div class="complex-type-card">
                        <h3>📈 Type Analysis Summary</h3>
                        <div id="typeAnalysisSummary"></div>
                    </div>
                    <div class="complex-type-card">
                        <h3>🔍 Detailed Type Breakdown</h3>
                        <div id="detailedTypeBreakdown"></div>
                    </div>
                </div>
            </div>
        `;
        this.populateComplexTypesData(complexTypesData);
    }

    populateComplexTypesData(data) {
        const categorized = data.categorized_types || {};
        const analysis = data.complex_type_analysis || [];
        const summary = data.summary || {};
        
        const categoriesEl = document.getElementById("typeCategoriesChart");
        if (categoriesEl) {
            categoriesEl.innerHTML = `
                <div class="type-category">
                    <span class="category-label">Collections:</span>
                    <span class="category-count">${categorized.collections?.length || 0}</span>
                </div>
                <div class="type-category">
                    <span class="category-label">Generic Types:</span>
                    <span class="category-count">${categorized.generic_types?.length || 0}</span>
                </div>
                <div class="type-category">
                    <span class="category-label">Smart Pointers:</span>
                    <span class="category-count">${categorized.smart_pointers?.length || 0}</span>
                </div>
                <div class="type-category">
                    <span class="category-label">Trait Objects:</span>
                    <span class="category-count">${categorized.trait_objects?.length || 0}</span>
                </div>
            `;
        }
    }

    // 🔗 Render Variable Relationships
    renderVariableRelationships() {
        const container = document.getElementById("variable-relationships");
        const relationshipsData = this.data.variable_relationships || {};
        
        container.innerHTML = `
            <div class="relationships-dashboard">
                <h2>🔗 Variable Relationships Analysis</h2>
                <div class="relationships-grid">
                    <div class="relationship-card">
                        <h3>🌐 Dependency Graph</h3>
                        <div id="dependencyGraph">
                            <div class="no-data">No dependency graph data available</div>
                        </div>
                    </div>
                    <div class="relationship-card">
                        <h3>🏗️ Scope Hierarchy</h3>
                        <div id="scopeHierarchy">
                            <div class="no-data">No scope hierarchy data available</div>
                        </div>
                    </div>
                    <div class="relationship-card">
                        <h3>🔄 Variable Interactions</h3>
                        <div id="variableInteractions">
                            <div class="no-data">No variable interactions data available</div>
                        </div>
                    </div>
                    <div class="relationship-card">
                        <h3>📊 Relationship Statistics</h3>
                        <div id="relationshipStats">
                            <div class="relationship-summary">
                                <div class="stat-item">
                                    <span class="stat-label">Total Relationships:</span>
                                    <span class="stat-value">0</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        `;
    }
}

// ===========================================
// 应用初始化和数据加载管理
// ===========================================

/**
 * 初始化MemScope应用
 */
async function initializeMemScopeApp() {
    try {
        console.log('🚀 开始初始化MemScope应用');
        
        // 显示加载状态
        showLoadingState();
        
        // 创建数据加载器
        globalDataLoader = new JSONDataLoader();
        
        // 设置进度回调
        globalDataLoader.onProgress((source, progress) => {
            updateLoadingProgress(source, progress);
        });
        
        // 尝试加载JSON数据
        let data;
        try {
            data = await globalDataLoader.loadAllData();
            console.log('✅ JSON数据加载成功');
        } catch (error) {
            console.warn('⚠️ JSON数据加载失败，使用嵌入数据:', error);
            data = processEmbeddedData();
        }
        
        // 创建可视化器
        globalVisualizer = new MemScopeVisualizer(data);
        
        // 隐藏加载状态
        hideLoadingState();
        
        console.log('🎉 MemScope应用初始化完成');
        
    } catch (error) {
        console.error('❌ 应用初始化失败:', error);
        showErrorState(error);
    }
}

/**
 * 处理嵌入数据
 */
function processEmbeddedData() {
    console.log('🔄 处理嵌入数据');
    
    if (typeof EMBEDDED_DATA !== 'undefined' && EMBEDDED_DATA) {
        // 如果有嵌入数据，直接使用
        return EMBEDDED_DATA;
    } else {
        // 创建默认数据结构
        return {
            allocations: [],
            performance: {
                active_allocations: 0,
                active_memory: 0,
                peak_memory: 0
            },
            security: {
                violations: [],
                risk_level: 'LOW'
            },
            unsafeFFI: {
                allocations: [],
                safety_score: 100
            },
            complexTypes: {
                categories: {},
                summary: { total_types: 0 }
            },
            metadata: {
                timestamp: Date.now(),
                sources: ['embedded'],
                note: '使用默认数据结构'
            }
        };
    }
}

/**
 * 显示加载状态
 */
function showLoadingState() {
    const overlay = document.createElement('div');
    overlay.id = 'loadingOverlay';
    overlay.style.cssText = `
        position: fixed; top: 0; left: 0; width: 100%; height: 100%;
        background: rgba(255,255,255,0.95); z-index: 9999;
        display: flex; flex-direction: column; align-items: center; justify-content: center;
    `;
    
    overlay.innerHTML = `
        <div style="text-align: center;">
            <h2>🔄 加载内存分析数据</h2>
            <div id="loadingProgress" style="margin: 20px 0;">
                <div style="margin: 10px 0;">
                    <span>内存分析数据:</span> <span id="progress-memory_analysis">⏳</span>
                </div>
                <div style="margin: 10px 0;">
                    <span>性能数据:</span> <span id="progress-performance">⏳</span>
                </div>
                <div style="margin: 10px 0;">
                    <span>安全违规数据:</span> <span id="progress-security_violations">⏳</span>
                </div>
                <div style="margin: 10px 0;">
                    <span>不安全FFI数据:</span> <span id="progress-unsafe_ffi">⏳</span>
                </div>
                <div style="margin: 10px 0;">
                    <span>复杂类型数据:</span> <span id="progress-complex_types">⏳</span>
                </div>
            </div>
            <p style="color: #666;">正在从JSON文件加载数据，请稍候...</p>
        </div>
    `;
    
    document.body.appendChild(overlay);
}

/**
 * 更新加载进度
 */
function updateLoadingProgress(source, progress) {
    const element = document.getElementById(`progress-${source}`);
    if (element) {
        if (progress === -1) {
            element.textContent = '❌ 失败';
            element.style.color = '#e74c3c';
        } else if (progress === 100) {
            element.textContent = '✅ 完成';
            element.style.color = '#2ecc71';
        } else {
            element.textContent = `${progress}%`;
            element.style.color = '#3498db';
        }
    }

    /**
     * 任务 5.1: 初始化过滤控件
     */
    initializeFilterControls() {
        console.log('🎛️ 初始化过滤控件系统');
        try {
            // 创建过滤控件实例
            this.filterControls = new FilterControls(this);
            console.log('✅ 过滤控件初始化成功');
        } catch (error) {
            console.error('❌ 过滤控件初始化失败:', error);
        }
    }

    /**
     * 任务 6.1: 初始化性能仪表板
     */
    initializePerformanceDashboard() {
        console.log('📊 初始化性能仪表板系统');
        try {
            // 创建性能仪表板实例
            this.performanceDashboard = new PerformanceDashboard(this);
            console.log('✅ 性能仪表板初始化成功');
        } catch (error) {
            console.error('❌ 性能仪表板初始化失败:', error);
        }
    }

    /**
     * 任务 7.1: 初始化安全仪表板
     */
    initializeSecurityDashboard() {
        console.log('🔒 初始化安全仪表板系统');
        try {
            // 创建安全仪表板实例
            this.securityDashboard = new SecurityDashboard(this);
            console.log('✅ 安全仪表板初始化成功');
        } catch (error) {
            console.error('❌ 安全仪表板初始化失败:', error);
        }
    }
}

/**
 * 隐藏加载状态
 */
function hideLoadingState() {
    const overlay = document.getElementById('loadingOverlay');
    if (overlay) {
        overlay.style.opacity = '0';
        setTimeout(() => overlay.remove(), 300);
    }
}

/**
 * 显示错误状态
 */
function showErrorState(error) {
    const overlay = document.getElementById('loadingOverlay');
    if (overlay) {
        overlay.innerHTML = `
            <div style="text-align: center; color: #e74c3c;">
                <h2>❌ 数据加载失败</h2>
                <p style="margin: 20px 0; max-width: 500px;">${error.message}</p>
                <button onclick="location.reload()" style="
                    padding: 10px 20px; background: #3498db; color: white;
                    border: none; border-radius: 5px; cursor: pointer;
                ">重新加载</button>
                <button onclick="loadFallbackData()" style="
                    padding: 10px 20px; background: #95a5a6; color: white;
                    border: none; border-radius: 5px; cursor: pointer; margin-left: 10px;
                ">使用示例数据</button>
            </div>
        `;
    }
}

/**
 * 加载回退数据
 */
function loadFallbackData() {
    try {
        console.log('🔄 使用回退数据');
        const data = processEmbeddedData();
        globalVisualizer = new MemScopeVisualizer(data);
        hideLoadingState();
        console.log('✅ 回退数据加载成功');
    } catch (error) {
        console.error('❌ 回退数据加载失败:', error);
        showErrorState(new Error('所有数据源都不可用'));
    }
}

// 导出全局函数供HTML使用
window.initializeMemScopeApp = initializeMemScopeApp;
window.loadFallbackData = loadFallbackData;