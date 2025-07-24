//! HTML template generation module with caching and optimization
//!
//! This module provides advanced HTML template generation capabilities with
//! template caching, responsive design support, and performance optimizations.

use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use super::data_normalizer::UnifiedMemoryData;

/// Template generation error types
#[derive(Debug)]
pub enum TemplateError {
    /// Template compilation failed
    CompilationError(String),
    /// Data serialization failed
    SerializationError(String),
    /// Cache operation failed
    CacheError(String),
    /// Resource loading failed
    ResourceError(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::CompilationError(msg) => write!(f, "Template compilation error: {}", msg),
            TemplateError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            TemplateError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            TemplateError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
        }
    }
}

impl Error for TemplateError {}

/// Template cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Cached template content
    content: String,
    /// Creation timestamp
    created_at: std::time::Instant,
    /// Access count
    access_count: usize,
    /// Template hash for validation
    hash: u64,
}

/// Template cache with LRU eviction
#[derive(Debug)]
struct TemplateCache {
    /// Cache entries
    entries: HashMap<String, CacheEntry>,
    /// Maximum cache size
    max_size: usize,
    /// Cache hits
    hits: usize,
    /// Cache misses
    misses: usize,
}

impl TemplateCache {
    fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            hits: 0,
            misses: 0,
        }
    }
    
    fn get(&mut self, key: &str) -> Option<String> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.access_count += 1;
            self.hits += 1;
            Some(entry.content.clone())
        } else {
            self.misses += 1;
            None
        }
    }
    
    fn put(&mut self, key: String, content: String, hash: u64) {
        // Evict oldest entry if cache is full
        if self.entries.len() >= self.max_size {
            self.evict_lru();
        }
        
        let entry = CacheEntry {
            content,
            created_at: Instant::now(),
            access_count: 1,
            hash,
        };
        
        self.entries.insert(key, entry);
    }
    
    fn evict_lru(&mut self) {
        if let Some((key_to_remove, _)) = self.entries
            .iter()
            .min_by_key(|(_, entry)| (entry.access_count, entry.created_at))
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.entries.remove(&key_to_remove);
        }
    }
    
    fn get_stats(&self) -> (usize, usize, f64) {
        let total_requests = self.hits + self.misses;
        let hit_rate = if total_requests > 0 {
            self.hits as f64 / total_requests as f64 * 100.0
        } else {
            0.0
        };
        (self.hits, self.misses, hit_rate)
    }
}

/// Global template cache
static TEMPLATE_CACHE: OnceLock<Arc<Mutex<TemplateCache>>> = OnceLock::new();

/// Template generation configuration
#[derive(Debug, Clone)]
pub struct TemplateConfig {
    /// Enable template caching
    pub enable_caching: bool,
    /// Cache size limit
    pub cache_size: usize,
    /// Enable responsive design
    pub enable_responsive: bool,
    /// Enable CSS minification
    pub minify_css: bool,
    /// Enable JavaScript minification
    pub minify_js: bool,
    /// Enable compression
    pub enable_compression: bool,
    /// Theme selection
    pub theme: String,
    /// Custom CSS variables
    pub css_variables: HashMap<String, String>,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_size: 50,
            enable_responsive: true,
            minify_css: false,
            minify_js: false,
            enable_compression: false,
            theme: "default".to_string(),
            css_variables: HashMap::new(),
        }
    }
}

/// Template generation statistics
#[derive(Debug)]
pub struct TemplateStats {
    /// Template generation time in milliseconds
    pub generation_time_ms: u64,
    /// CSS processing time in milliseconds
    pub css_processing_time_ms: u64,
    /// JavaScript processing time in milliseconds
    pub js_processing_time_ms: u64,
    /// Data serialization time in milliseconds
    pub serialization_time_ms: u64,
    /// Final template size in bytes
    pub template_size_bytes: usize,
    /// Cache hit rate percentage
    pub cache_hit_rate: f64,
    /// Compression ratio (if enabled)
    pub compression_ratio: Option<f64>,
}

/// Advanced HTML template generator
pub struct TemplateGenerator {
    /// Configuration
    config: TemplateConfig,
    /// CSS content cache
    css_content: Option<String>,
    /// JavaScript content cache
    js_content: Option<String>,
}

impl TemplateGenerator {
    /// Create a new template generator with default configuration
    pub fn new() -> Self {
        Self {
            config: TemplateConfig::default(),
            css_content: None,
            js_content: None,
        }
    }
    
    /// Create template generator with custom configuration
    pub fn with_config(config: TemplateConfig) -> Self {
        Self {
            config,
            css_content: None,
            js_content: None,
        }
    }
    
    /// Generate optimized HTML template
    pub fn generate_html(
        &mut self,
        unified_data: &UnifiedMemoryData,
    ) -> Result<(String, TemplateStats), TemplateError> {
        let start_time = Instant::now();
        
        println!("🎨 Starting optimized HTML template generation...");
        
        // Initialize cache if needed
        self.init_cache()?;
        
        // Load and process CSS
        let css_start = Instant::now();
        let css_content = self.get_processed_css()?;
        let css_time = css_start.elapsed().as_millis() as u64;
        
        // Load and process JavaScript
        let js_start = Instant::now();
        let js_content = self.get_processed_js()?;
        let js_time = js_start.elapsed().as_millis() as u64;
        
        // Serialize data
        let serialization_start = Instant::now();
        let json_data = self.serialize_data(unified_data)?;
        let serialization_time = serialization_start.elapsed().as_millis() as u64;
        
        // Generate template
        let template_content = self.build_template(&css_content, &js_content, &json_data, unified_data)?;
        
        // Get cache statistics
        let cache_hit_rate = self.get_cache_hit_rate();
        
        let total_time = start_time.elapsed().as_millis() as u64;
        
        let stats = TemplateStats {
            generation_time_ms: total_time,
            css_processing_time_ms: css_time,
            js_processing_time_ms: js_time,
            serialization_time_ms: serialization_time,
            template_size_bytes: template_content.len(),
            cache_hit_rate,
            compression_ratio: None, // TODO: Implement compression
        };
        
        println!("✅ Template generation completed in {}ms", total_time);
        println!("   CSS processing: {}ms", css_time);
        println!("   JS processing: {}ms", js_time);
        println!("   Data serialization: {}ms", serialization_time);
        println!("   Template size: {:.1} KB", template_content.len() as f64 / 1024.0);
        println!("   Cache hit rate: {:.1}%", cache_hit_rate);
        
        Ok((template_content, stats))
    }
    
    /// Initialize template cache
    fn init_cache(&self) -> Result<(), TemplateError> {
        if self.config.enable_caching {
            TEMPLATE_CACHE.get_or_init(|| {
                Arc::new(Mutex::new(TemplateCache::new(self.config.cache_size)))
            });
        }
        Ok(())
    }
    
    /// Get processed CSS content with caching
    fn get_processed_css(&mut self) -> Result<String, TemplateError> {
        if let Some(cached_css) = &self.css_content {
            return Ok(cached_css.clone());
        }
        
        let cache_key = format!("css_{}_{}", self.config.theme, self.config.minify_css);
        
        // Try cache first
        if self.config.enable_caching {
            if let Some(cache) = TEMPLATE_CACHE.get() {
                if let Ok(mut cache_guard) = cache.lock() {
                    if let Some(cached) = cache_guard.get(&cache_key) {
                        self.css_content = Some(cached.clone());
                        return Ok(cached);
                    }
                }
            }
        }
        
        // Load and process CSS
        let mut css_content = include_str!("../../../../templates/styles.css").to_string();
        
        // Apply theme modifications
        css_content = self.apply_theme(&css_content)?;
        
        // Apply custom CSS variables
        css_content = self.apply_css_variables(&css_content)?;
        
        // Add responsive design enhancements
        if self.config.enable_responsive {
            css_content = self.add_responsive_css(&css_content)?;
        }
        
        // Minify if requested
        if self.config.minify_css {
            css_content = self.minify_css(&css_content)?;
        }
        
        // Cache the result
        if self.config.enable_caching {
            if let Some(cache) = TEMPLATE_CACHE.get() {
                if let Ok(mut cache_guard) = cache.lock() {
                    let hash = self.calculate_hash(&css_content);
                    cache_guard.put(cache_key, css_content.clone(), hash);
                }
            }
        }
        
        self.css_content = Some(css_content.clone());
        Ok(css_content)
    }
    
    /// Get processed JavaScript content with caching
    fn get_processed_js(&mut self) -> Result<String, TemplateError> {
        if let Some(cached_js) = &self.js_content {
            return Ok(cached_js.clone());
        }
        
        let cache_key = format!("js_{}", self.config.minify_js);
        
        // Try cache first
        if self.config.enable_caching {
            if let Some(cache) = TEMPLATE_CACHE.get() {
                if let Ok(mut cache_guard) = cache.lock() {
                    if let Some(cached) = cache_guard.get(&cache_key) {
                        self.js_content = Some(cached.clone());
                        return Ok(cached);
                    }
                }
            }
        }
        
        // Load and process JavaScript
        let mut js_content = include_str!("../../../../templates/script.js").to_string();
        
        // Add enhanced functionality
        js_content = self.add_enhanced_js_features(&js_content)?;
        
        // Minify if requested
        if self.config.minify_js {
            js_content = self.minify_js(&js_content)?;
        }
        
        // Cache the result
        if self.config.enable_caching {
            if let Some(cache) = TEMPLATE_CACHE.get() {
                if let Ok(mut cache_guard) = cache.lock() {
                    let hash = self.calculate_hash(&js_content);
                    cache_guard.put(cache_key, js_content.clone(), hash);
                }
            }
        }
        
        self.js_content = Some(js_content.clone());
        Ok(js_content)
    }
    
    /// Apply theme to CSS
    fn apply_theme(&self, css: &str) -> Result<String, TemplateError> {
        let mut themed_css = css.to_string();
        
        match self.config.theme.as_str() {
            "dark" => {
                themed_css = themed_css.replace(
                    ":root {",
                    ":root {\n  --bg-color: #1a1a1a;\n  --text-color: #ffffff;\n  --card-bg: #2d2d2d;\n  --border-color: #444444;"
                );
            }
            "high-contrast" => {
                themed_css = themed_css.replace(
                    ":root {",
                    ":root {\n  --bg-color: #000000;\n  --text-color: #ffffff;\n  --card-bg: #333333;\n  --accent-color: #ffff00;"
                );
            }
            _ => {} // Default theme
        }
        
        Ok(themed_css)
    }
    
    /// Apply custom CSS variables
    fn apply_css_variables(&self, css: &str) -> Result<String, TemplateError> {
        let mut css_with_vars = css.to_string();
        
        if !self.config.css_variables.is_empty() {
            let mut variables_section = String::new();
            for (key, value) in &self.config.css_variables {
                variables_section.push_str(&format!("  --{}: {};\n", key, value));
            }
            
            css_with_vars = css_with_vars.replace(
                ":root {",
                &format!(":root {{\n{}", variables_section)
            );
        }
        
        Ok(css_with_vars)
    }
    
    /// Add responsive CSS enhancements
    fn add_responsive_css(&self, css: &str) -> Result<String, TemplateError> {
        let responsive_css = r#"
/* Enhanced Responsive Design */
@media (max-width: 768px) {
    .container {
        padding: 10px;
    }
    
    .overview-grid {
        grid-template-columns: 1fr;
        gap: 15px;
    }
    
    .tab-nav {
        flex-wrap: wrap;
        gap: 5px;
    }
    
    .tab-btn {
        flex: 1;
        min-width: 120px;
        font-size: 12px;
        padding: 8px 12px;
    }
    
    .header-stats {
        flex-direction: column;
        gap: 8px;
    }
    
    .stat-badge {
        font-size: 12px;
        padding: 6px 12px;
    }
    
    .explorer-controls {
        flex-direction: column;
        gap: 10px;
    }
    
    .control-group {
        flex-direction: column;
        align-items: flex-start;
    }
    
    .allocation-grid {
        grid-template-columns: 1fr;
    }
}

@media (max-width: 480px) {
    .header h1 {
        font-size: 20px;
    }
    
    .tab-btn {
        font-size: 11px;
        padding: 6px 8px;
    }
    
    .overview-card h3 {
        font-size: 16px;
    }
    
    .stats-grid {
        grid-template-columns: 1fr;
    }
}

/* Touch-friendly enhancements */
@media (hover: none) and (pointer: coarse) {
    .tab-btn {
        min-height: 44px;
        touch-action: manipulation;
    }
    
    button, select, input {
        min-height: 44px;
        touch-action: manipulation;
    }
    
    .allocation-item {
        padding: 12px;
        margin: 8px 0;
    }
}

/* High DPI display optimizations */
@media (-webkit-min-device-pixel-ratio: 2), (min-resolution: 192dpi) {
    .svg-image {
        image-rendering: -webkit-optimize-contrast;
        image-rendering: crisp-edges;
    }
}

/* Reduced motion preferences */
@media (prefers-reduced-motion: reduce) {
    * {
        animation-duration: 0.01ms !important;
        animation-iteration-count: 1 !important;
        transition-duration: 0.01ms !important;
    }
}

/* Dark mode preference */
@media (prefers-color-scheme: dark) {
    :root {
        --bg-color: #1a1a1a;
        --text-color: #ffffff;
        --card-bg: #2d2d2d;
        --border-color: #444444;
    }
}
"#;
        
        Ok(format!("{}\n{}", css, responsive_css))
    }
    
    /// Add enhanced JavaScript features
    fn add_enhanced_js_features(&self, js: &str) -> Result<String, TemplateError> {
        let enhanced_js = r#"
// 🎯 MemScope可视化器 - 清理后的统一实现
class MemScopeVisualizer {
    constructor(data) {
        this.data = data;
        console.log('📊 MemScope初始化，数据:', data);
        this.init();
    }

    init() {
        this.setupTabNavigation();
        this.renderOverview();
        this.updateHeaderStats();
    }

    setupTabNavigation() {
        const tabButtons = document.querySelectorAll('.tab-btn');
        const tabContents = document.querySelectorAll('.tab-content');
        
        tabButtons.forEach(button => {
            button.addEventListener('click', () => {
                const targetTab = button.getAttribute('data-tab');
                
                tabButtons.forEach(btn => {
                    btn.classList.remove('active');
                    btn.setAttribute('aria-selected', 'false');
                });
                button.classList.add('active');
                button.setAttribute('aria-selected', 'true');
                
                tabContents.forEach(content => {
                    content.classList.remove('active');
                    if (content.id === targetTab) {
                        content.classList.add('active');
                        this.renderTabContent(targetTab);
                    }
                });
            });
        });
    }

    renderTabContent(tabName) {
        switch (tabName) {
            case 'overview': this.renderOverview(); break;
            case 'memory-analysis': this.renderMemoryAnalysis(); break;
            case 'lifecycle': this.renderLifecycleTimeline(); break;
            case 'unsafe-ffi': this.renderUnsafeFFI(); break;
            case 'performance': this.renderPerformance(); break;
            case 'security': this.renderSecurity(); break;
            case 'complex-types': this.renderComplexTypes(); break;
            case 'variables': this.renderVariableRelationships(); break;
            case 'interactive': this.renderInteractiveExplorer(); break;
        }
    }

    updateHeaderStats() {
        const stats = this.data.stats || {};
        const totalMemoryEl = document.getElementById('totalMemory');
        const activeAllocsEl = document.getElementById('activeAllocs');
        const peakMemoryEl = document.getElementById('peakMemory');
        
        if (totalMemoryEl) totalMemoryEl.textContent = this.formatBytes(stats.active_memory || 0);
        if (activeAllocsEl) activeAllocsEl.textContent = `${stats.active_allocations || 0} Active`;
        if (peakMemoryEl) peakMemoryEl.textContent = this.formatBytes(stats.peak_memory || 0);
    }

    renderOverview() {
        this.renderMemoryStats();
        this.renderTypeDistribution();
        this.renderRecentAllocations();
        this.renderPerformanceInsights();
    }

    renderMemoryStats() {
        const element = document.getElementById('memoryStats');
        if (!element) return;
        
        const stats = this.data.stats || {};
        element.innerHTML = `
            <div class="stats-grid">
                <div class="stat-item">
                    <span class="stat-label">活跃内存:</span>
                    <span class="stat-value">${this.formatBytes(stats.active_memory || 0)}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">峰值内存:</span>
                    <span class="stat-value">${this.formatBytes(stats.peak_memory || 0)}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">总分配:</span>
                    <span class="stat-value">${(stats.total_allocations || 0).toLocaleString()}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">活跃分配:</span>
                    <span class="stat-value">${(stats.active_allocations || 0).toLocaleString()}</span>
                </div>
            </div>
        `;
    }

    renderTypeDistribution() {
        const element = document.getElementById('typeDistribution');
        if (!element) return;
        
        // 兼容test_data中的字段名
        const allocations = this.data.allocations || [];
        const typeMap = {};
        
        allocations.forEach(alloc => {
            const typeName = alloc.type_name || alloc.type || '未知类型';
            if (!typeMap[typeName]) {
                typeMap[typeName] = { count: 0, totalSize: 0 };
            }
            typeMap[typeName].count++;
            typeMap[typeName].totalSize += alloc.size || 0;
        });

        const sortedTypes = Object.entries(typeMap)
            .sort((a, b) => b[1].totalSize - a[1].totalSize)
            .slice(0, 10);

        if (sortedTypes.length === 0) {
            element.innerHTML = '<p>暂无类型分布数据</p>';
            return;
        }

        const html = sortedTypes.map(([typeName, data]) => `
            <div class="type-item">
                <span class="type-name">${typeName}</span>
                <span class="type-size">${this.formatBytes(data.totalSize)}</span>
                <span class="type-count">${data.count} 个</span>
            </div>
        `).join('');

        element.innerHTML = `<div class="type-list">${html}</div>`;
    }

    renderRecentAllocations() {
        const element = document.getElementById('recentAllocations');
        if (!element) return;
        
        const allocations = this.data.allocations || [];
        if (allocations.length === 0) {
            element.innerHTML = '<p>暂无分配数据</p>';
            return;
        }

        const recent = allocations
            .sort((a, b) => (b.timestamp || b.timestamp_alloc || 0) - (a.timestamp || a.timestamp_alloc || 0))
            .slice(0, 5);

        const html = recent.map(alloc => `
            <div class="allocation-item">
                <span class="alloc-var">${alloc.var_name || alloc.variable || '未知变量'}</span>
                <span class="alloc-type">${alloc.type_name || alloc.type || '未知类型'}</span>
                <span class="alloc-size">${this.formatBytes(alloc.size || 0)}</span>
            </div>
        `).join('');

        element.innerHTML = `<div class="allocation-list">${html}</div>`;
    }

    renderPerformanceInsights() {
        const element = document.getElementById('performanceInsights');
        if (!element) return;
        
        const stats = this.data.stats || {};
        const allocations = this.data.allocations || [];
        
        element.innerHTML = `
            <div class="insights-grid">
                <div class="insight-item">
                    <span class="insight-label">内存效率:</span>
                    <span class="insight-value">${(stats.memory_efficiency || 85).toFixed(1)}%</span>
                </div>
                <div class="insight-item">
                    <span class="insight-label">分配数量:</span>
                    <span class="insight-value">${allocations.length} 项</span>
                </div>
                <div class="insight-item">
                    <span class="insight-label">平均大小:</span>
                    <span class="insight-value">${this.formatBytes(allocations.length > 0 ? (stats.total_allocated || stats.active_memory || 0) / allocations.length : 0)}</span>
                </div>
                <div class="insight-item">
                    <span class="insight-label">数据状态:</span>
                    <span class="insight-value">✅ 已加载</span>
                </div>
            </div>
        `;
    }

    renderMemoryAnalysis() {
        const element = document.getElementById('memoryAnalysisContent');
        if (element) {
            const allocations = this.data.allocations || [];
            element.innerHTML = `
                <h2>📊 内存分析</h2>
                <p>已加载 ${allocations.length} 个内存分配记录</p>
                <div class="analysis-summary">
                    <p>• 总内存使用: ${this.formatBytes(this.data.stats?.active_memory || 0)}</p>
                    <p>• 峰值内存: ${this.formatBytes(this.data.stats?.peak_memory || 0)}</p>
                </div>
            `;
        }
    }

    renderLifecycleTimeline() {
        const element = document.getElementById('lifecycleContent');
        if (element) {
            const lifecycle = this.data.lifecycle_events || [];
            element.innerHTML = `
                <h2>⏱️ 生命周期时间线</h2>
                <p>检测到 ${lifecycle.length} 个生命周期事件</p>
            `;
        }
    }

    renderUnsafeFFI() {
        const element = document.getElementById('unsafeFfiContent');
        if (element) {
            const unsafeData = this.data.unsafe_ffi || {};
            element.innerHTML = `
                <h2>⚠️ Unsafe/FFI 分析</h2>
                <p>Unsafe分配: ${unsafeData.allocations?.length || 0} 个</p>
                <p>安全违规: ${unsafeData.violations?.length || 0} 个</p>
            `;
        }
    }

    renderPerformance() {
        const element = document.getElementById('performanceContent');
        if (element) {
            const performance = this.data.performance || {};
            element.innerHTML = `
                <h2>⚡ 性能分析</h2>
                <p>性能指标: ${Object.keys(performance).length} 项</p>
            `;
        }
    }

    renderSecurity() {
        const element = document.getElementById('securityContent');
        if (element) {
            const security = this.data.security || {};
            element.innerHTML = `
                <h2>🔒 安全分析</h2>
                <p>安全检查: ${security.violations?.length || 0} 个问题</p>
            `;
        }
    }

    renderComplexTypes() {
        const element = document.getElementById('complexTypesContent');
        if (element) {
            const complexTypes = this.data.categorized_types || this.data.complex_types || {};
            const genericTypes = complexTypes.generic_types || [];
            const collections = complexTypes.collections || [];
            
            element.innerHTML = `
                <h2>🔧 复杂类型分析</h2>
                <p>泛型类型: ${genericTypes.length} 个</p>
                <p>集合类型: ${collections.length} 个</p>
            `;
        }
    }

    renderVariableRelationships() {
        const element = document.getElementById('variableContent');
        if (element) {
            const relationships = this.data.variable_relationships || [];
            element.innerHTML = `
                <h2>🔗 变量关系</h2>
                <p>变量关系: ${relationships.length} 个</p>
            `;
        }
    }

    renderInteractiveExplorer() {
        const gridElement = document.getElementById('allocationGrid');
        if (!gridElement) return;
        
        const allocations = this.data.allocations || [];
        if (allocations.length === 0) {
            gridElement.innerHTML = '<p>暂无分配数据可供探索</p>';
            return;
        }

        this.setupExplorerFilters();
        this.renderAllocationGrid(allocations.slice(0, 50));
    }

    setupExplorerFilters() {
        const typeFilter = document.getElementById('filterType');
        if (!typeFilter) return;
        
        const allocations = this.data.allocations || [];
        const types = [...new Set(allocations.map(a => a.type_name || a.type).filter(Boolean))];
        
        typeFilter.innerHTML = '<option value="">All Types</option>' + 
            types.map(type => `<option value="${type}">${type}</option>`).join('');
    }

    renderAllocationGrid(allocations) {
        const gridElement = document.getElementById('allocationGrid');
        if (!gridElement) return;
        
        const html = allocations.map(alloc => `
            <div class="allocation-card">
                <div class="card-header">
                    <span class="var-name">${alloc.var_name || alloc.variable || '未知变量'}</span>
                    <span class="alloc-size">${this.formatBytes(alloc.size || 0)}</span>
                </div>
                <div class="card-body">
                    <p><strong>类型:</strong> ${alloc.type_name || alloc.type || '未知'}</p>
                    <p><strong>指针:</strong> ${alloc.ptr || 'N/A'}</p>
                    <p><strong>作用域:</strong> ${alloc.scope_name || alloc.scope || '全局'}</p>
                </div>
            </div>
        `).join('');

        gridElement.innerHTML = html;
    }

    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
}

// Enhanced Performance Monitoring
class PerformanceMonitor {
    constructor() {
        this.metrics = {};
        this.observers = [];
    }
    
    startTiming(name) {
        this.metrics[name] = { start: performance.now() };
    }
    
    endTiming(name) {
        if (this.metrics[name]) {
            this.metrics[name].duration = performance.now() - this.metrics[name].start;
        }
    }
    
    getMetrics() {
        return this.metrics;
    }
}

// Virtual Scrolling Implementation
class VirtualScroller {
    constructor(container, itemHeight, renderItem) {
        this.container = container;
        this.itemHeight = itemHeight;
        this.renderItem = renderItem;
        this.items = [];
        this.visibleStart = 0;
        this.visibleEnd = 0;
        this.scrollTop = 0;
        
        this.setupScrolling();
    }
    
    setupScrolling() {
        this.container.addEventListener('scroll', () => {
            this.scrollTop = this.container.scrollTop;
            this.updateVisibleRange();
            this.render();
        });
    }
    
    setItems(items) {
        this.items = items;
        this.updateVisibleRange();
        this.render();
    }
    
    updateVisibleRange() {
        const containerHeight = this.container.clientHeight;
        const totalHeight = this.items.length * this.itemHeight;
        
        this.visibleStart = Math.floor(this.scrollTop / this.itemHeight);
        this.visibleEnd = Math.min(
            this.items.length,
            this.visibleStart + Math.ceil(containerHeight / this.itemHeight) + 1
        );
        
        // Update container height
        this.container.style.height = totalHeight + 'px';
    }
    
    render() {
        const fragment = document.createDocumentFragment();
        
        for (let i = this.visibleStart; i < this.visibleEnd; i++) {
            const item = this.renderItem(this.items[i], i);
            item.style.position = 'absolute';
            item.style.top = (i * this.itemHeight) + 'px';
            item.style.height = this.itemHeight + 'px';
            fragment.appendChild(item);
        }
        
        this.container.innerHTML = '';
        this.container.appendChild(fragment);
    }
}

// Progressive Loading Manager
class ProgressiveLoader {
    constructor() {
        this.loadQueue = [];
        this.isLoading = false;
        this.batchSize = 100;
    }
    
    addToQueue(loadFunction, priority = 0) {
        this.loadQueue.push({ loadFunction, priority });
        this.loadQueue.sort((a, b) => b.priority - a.priority);
        
        if (!this.isLoading) {
            this.processQueue();
        }
    }
    
    async processQueue() {
        this.isLoading = true;
        
        while (this.loadQueue.length > 0) {
            const batch = this.loadQueue.splice(0, this.batchSize);
            
            await Promise.all(batch.map(item => {
                try {
                    return item.loadFunction();
                } catch (error) {
                    console.warn('Progressive loading error:', error);
                    return Promise.resolve();
                }
            }));
            
            // Yield to browser for other tasks
            await new Promise(resolve => setTimeout(resolve, 0));
        }
        
        this.isLoading = false;
    }
}

// Initialize enhanced features
window.performanceMonitor = new PerformanceMonitor();
window.progressiveLoader = new ProgressiveLoader();

// Enhanced initialization
function initializeEnhancedFeatures() {
    // Setup virtual scrolling for large datasets
    const allocationGrid = document.getElementById('allocationGrid');
    if (allocationGrid && window.MEMORY_DATA && window.MEMORY_DATA.allocations.length > 100) {
        window.virtualScroller = new VirtualScroller(
            allocationGrid,
            60, // Item height
            (allocation, index) => {
                const div = document.createElement('div');
                div.className = 'allocation-item';
                div.innerHTML = `
                    <div class="allocation-header">
                        <span class="allocation-ptr">${allocation.ptr}</span>
                        <span class="allocation-size">${formatBytes(allocation.size)}</span>
                    </div>
                    <div class="allocation-details">
                        <span class="allocation-var">${allocation.var_name || 'unnamed'}</span>
                        <span class="allocation-type">${allocation.type_name || 'unknown'}</span>
                    </div>
                `;
                return div;
            }
        );
        
        window.virtualScroller.setItems(window.MEMORY_DATA.allocations);
    }
    
    // Setup progressive loading for heavy computations
    window.progressiveLoader.addToQueue(() => {
        return new Promise(resolve => {
            // Load heavy visualizations progressively
            setTimeout(resolve, 10);
        });
    }, 1);
}

// Add to existing initialization
if (typeof initializeBasicViewUnified === 'function') {
    const originalInit = initializeBasicViewUnified;
    initializeBasicViewUnified = function(data) {
        originalInit(data);
        initializeEnhancedFeatures();
    };
}
"#;
        
        Ok(format!("{}\n{}", js, enhanced_js))
    }
    
    /// Minify CSS (basic implementation)
    fn minify_css(&self, css: &str) -> Result<String, TemplateError> {
        // Basic CSS minification
        let minified = css
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("/*"))
            .collect::<Vec<_>>()
            .join("")
            .replace("  ", " ")
            .replace("; ", ";")
            .replace(": ", ":")
            .replace("{ ", "{")
            .replace(" }", "}");
        
        Ok(minified)
    }
    
    /// Minify JavaScript (basic implementation)
    fn minify_js(&self, js: &str) -> Result<String, TemplateError> {
        // Basic JavaScript minification
        let minified = js
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect::<Vec<_>>()
            .join(" ")
            .replace("  ", " ")
            .replace("; ", ";")
            .replace("{ ", "{")
            .replace(" }", "}");
        
        Ok(minified)
    }
    
    /// Serialize data to JSON
    fn serialize_data(&self, unified_data: &UnifiedMemoryData) -> Result<String, TemplateError> {
        serde_json::to_string(unified_data)
            .map_err(|e| TemplateError::SerializationError(e.to_string()))
    }
    
    /// Build complete HTML template
    fn build_template(
        &self,
        css_content: &str,
        js_content: &str,
        json_data: &str,
        unified_data: &UnifiedMemoryData,
    ) -> Result<String, TemplateError> {
        let stats = &unified_data.stats;
        
        // Format statistics for header
        let total_memory = format_bytes(stats.active_memory);
        let active_allocs = format!("{} Active", stats.active_allocations);
        let peak_memory = format_bytes(stats.peak_memory);
        
        // Build responsive viewport meta tag
        let viewport_meta = if self.config.enable_responsive {
            r#"<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=yes">"#
        } else {
            r#"<meta name="viewport" content="width=device-width, initial-scale=1.0">"#
        };
        
        // Build theme class
        let theme_class = if self.config.theme != "default" {
            format!(" theme-{}", self.config.theme)
        } else {
            String::new()
        };
        
        let html = format!(r#"<!DOCTYPE html>
<html lang="en"{theme_class}>
<head>
    <meta charset="UTF-8">
    {viewport_meta}
    <title>MemScope-RS Interactive Memory Analysis</title>
    <meta name="description" content="Interactive memory analysis report generated by MemScope-RS">
    <meta name="generator" content="MemScope-RS Template Generator v2.0">
    <style>
        {css_content}
    </style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1>🔍 MemScope-RS Interactive Memory Analysis</h1>
            <div class="header-stats">
                <span class="stat-badge" id="totalMemory">{total_memory}</span>
                <span class="stat-badge" id="activeAllocs">{active_allocs}</span>
                <span class="stat-badge" id="peakMemory">{peak_memory}</span>
            </div>
        </header>

        <nav class="tab-nav" role="tablist">
            <button class="tab-btn active" data-tab="overview" role="tab" aria-selected="true">📊 Overview</button>
            <button class="tab-btn" data-tab="memory-analysis" role="tab">🧠 Memory Analysis</button>
            <button class="tab-btn" data-tab="lifecycle" role="tab">⏱️ Lifecycle Timeline</button>
            <button class="tab-btn" data-tab="unsafe-ffi" role="tab">⚠️ Unsafe/FFI</button>
            <button class="tab-btn" data-tab="performance" role="tab">⚡ Performance</button>
            <button class="tab-btn" data-tab="security" role="tab">🔒 Security</button>
            <button class="tab-btn" data-tab="complex-types" role="tab">🔧 Complex Types</button>
            <button class="tab-btn" data-tab="variables" role="tab">🔗 Variable Relationships</button>
            <button class="tab-btn" data-tab="interactive" role="tab">🎮 Interactive Explorer</button>
        </nav>

        <main class="content">
            <!-- Overview Tab -->
            <div class="tab-content active" id="overview" role="tabpanel">
                <div class="overview-grid">
                    <div class="overview-card">
                        <h3>📈 Memory Statistics</h3>
                        <div id="memoryStats" aria-live="polite">Loading...</div>
                    </div>
                    <div class="overview-card">
                        <h3>🏷️ Type Distribution</h3>
                        <div id="typeDistribution" aria-live="polite">Loading...</div>
                    </div>
                    <div class="overview-card">
                        <h3>📋 Recent Allocations</h3>
                        <div id="recentAllocations" aria-live="polite">Loading...</div>
                    </div>
                    <div class="overview-card">
                        <h3>⚡ Performance Insights</h3>
                        <div id="performanceInsights" aria-live="polite">Loading...</div>
                    </div>
                </div>
            </div>

            <!-- Memory Analysis Tab -->
            <div class="tab-content" id="memory-analysis" role="tabpanel">
                <div id="memoryAnalysisContent" aria-live="polite">Loading memory analysis...</div>
            </div>

            <!-- Lifecycle Timeline Tab -->
            <div class="tab-content" id="lifecycle" role="tabpanel">
                <div id="lifecycleContent" aria-live="polite">Loading lifecycle analysis...</div>
            </div>

            <!-- Unsafe/FFI Tab -->
            <div class="tab-content" id="unsafe-ffi" role="tabpanel">
                <div id="unsafeFfiContent" aria-live="polite">Loading unsafe/FFI analysis...</div>
            </div>

            <!-- Performance Tab -->
            <div class="tab-content" id="performance" role="tabpanel">
                <div id="performanceContent" aria-live="polite">Loading performance analysis...</div>
            </div>

            <!-- Security Tab -->
            <div class="tab-content" id="security" role="tabpanel">
                <div id="securityContent" aria-live="polite">Loading security analysis...</div>
            </div>

            <!-- Complex Types Tab -->
            <div class="tab-content" id="complex-types" role="tabpanel">
                <div id="complexTypesContent" aria-live="polite">Loading complex types analysis...</div>
            </div>

            <!-- Variable Relationships Tab -->
            <div class="tab-content" id="variables" role="tabpanel">
                <div id="variableContent" aria-live="polite">Loading variable relationships...</div>
            </div>

            <!-- Interactive Explorer Tab -->
            <div class="tab-content" id="interactive" role="tabpanel">
                <div class="explorer-controls">
                    <div class="control-group">
                        <label for="filterType">Filter by Type:</label>
                        <select id="filterType" aria-describedby="filterType-help">
                            <option value="">All Types</option>
                        </select>
                        <small id="filterType-help">Filter allocations by their type</small>
                    </div>
                    <div class="control-group">
                        <label for="sizeRange">Size Range:</label>
                        <input type="range" id="sizeRange" min="0" max="100" value="100" aria-describedby="sizeRange-help">
                        <span id="sizeRangeValue" aria-live="polite">All sizes</span>
                        <small id="sizeRange-help">Filter allocations by size range</small>
                    </div>
                    <div class="control-group">
                        <label for="sortBy">Sort by:</label>
                        <select id="sortBy" aria-describedby="sortBy-help">
                            <option value="size">Size</option>
                            <option value="timestamp">Timestamp</option>
                            <option value="type">Type</option>
                        </select>
                        <small id="sortBy-help">Sort allocations by selected criteria</small>
                    </div>
                </div>
                <div class="explorer-content">
                    <div class="allocation-grid" id="allocationGrid" role="grid" aria-label="Memory allocations">
                        Loading allocations...
                    </div>
                </div>
            </div>
        </main>
    </div>

    <script>
        // 嵌入的内存数据
        const EMBEDDED_DATA = {json_data};
        
        // 初始化应用程序
        function initializeMemScopeApp() {{
            console.log('🚀 Initializing MemScope-RS Interactive App...');
            
            try {{
                if (typeof EMBEDDED_DATA !== 'undefined' && EMBEDDED_DATA) {{
                    console.log('✅ Using embedded data');
                    processEmbeddedData(EMBEDDED_DATA);
                }} else {{
                    console.warn('⚠️ No embedded data found, showing error state');
                    showErrorState(new Error('No data available'));
                }}
            }} catch (error) {{
                console.error('❌ Initialization failed:', error);
                showErrorState(error);
            }}
        }}

        // 处理嵌入的数据
        function processEmbeddedData(data) {{
            console.log('📊 Processing embedded data...');
            
            try {{
                // 更新统计信息
                updateHeaderStats(data.stats || data.memory_analysis?.stats || {{}});
                
                // 初始化各个标签页
                initializeTabs();
                
                // 渲染概览页面
                renderOverviewTab(data);
                
                console.log('✅ Data processing completed successfully');
            }} catch (error) {{
                console.error('❌ Data processing failed:', error);
                showErrorState(error);
            }}
        }}

        // 更新头部统计信息
        function updateHeaderStats(stats) {{
            const totalMemoryEl = document.getElementById('totalMemory');
            const activeAllocsEl = document.getElementById('activeAllocs');
            const peakMemoryEl = document.getElementById('peakMemory');
            
            if (totalMemoryEl) totalMemoryEl.textContent = formatBytes(stats.active_memory || 0);
            if (activeAllocsEl) activeAllocsEl.textContent = `${{stats.active_allocations || 0}} Active`;
            if (peakMemoryEl) peakMemoryEl.textContent = formatBytes(stats.peak_memory || 0);
        }}

        // 渲染概览标签页
        function renderOverviewTab(data) {{
            const stats = data.stats || data.memory_analysis?.stats || {{}};
            const allocations = data.allocations || data.memory_analysis?.allocations || [];
            const memoryByType = data.memoryByType || data.memory_analysis?.memory_by_type || {{}};
            
            renderMemoryStats(stats);
            renderTypeDistribution(memoryByType);
            renderRecentAllocations(allocations);
            renderPerformanceInsights(stats);
        }}

        // 渲染内存统计
        function renderMemoryStats(stats) {{
            const element = document.getElementById('memoryStats');
            if (!element) return;
            
            const html = `
                <div class="stats-grid">
                    <div class="stat-item">
                        <span class="stat-label">Active Memory:</span>
                        <span class="stat-value">${{formatBytes(stats.active_memory || 0)}}</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">Peak Memory:</span>
                        <span class="stat-value">${{formatBytes(stats.peak_memory || 0)}}</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">Total Allocations:</span>
                        <span class="stat-value">${{stats.total_allocations || 0}}</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">Active Allocations:</span>
                        <span class="stat-value">${{stats.active_allocations || 0}}</span>
                    </div>
                </div>
            `;
            element.innerHTML = html;
        }}

        // 渲染类型分布
        function renderTypeDistribution(memoryByType) {{
            const element = document.getElementById('typeDistribution');
            if (!element) return;
            
            const types = Object.entries(memoryByType).slice(0, 5);
            
            if (types.length === 0) {{
                element.innerHTML = '<p>No type information available</p>';
                return;
            }}
            
            const html = types.map(([typeName, data]) => {{
                const size = Array.isArray(data) ? data[0] : data.total_size || 0;
                const count = Array.isArray(data) ? data[1] : data.allocation_count || 0;
                return `
                    <div class="type-item">
                        <span class="type-name">${{typeName}}</span>
                        <span class="type-stats">${{formatBytes(size)}} (${{count}} allocs)</span>
                    </div>
                `;
            }}).join('');
            
            element.innerHTML = html;
        }}

        // 渲染最近分配
        function renderRecentAllocations(allocations) {{
            const element = document.getElementById('recentAllocations');
            if (!element) return;
            
            const recent = allocations.slice(0, 5);
            
            if (recent.length === 0) {{
                element.innerHTML = '<p>No recent allocations</p>';
                return;
            }}
            
            const html = recent.map(alloc => `
                <div class="allocation-item">
                    <span class="alloc-size">${{formatBytes(alloc.size || 0)}}</span>
                    <span class="alloc-type">${{alloc.type_name || 'Unknown'}}</span>
                </div>
            `).join('');
            
            element.innerHTML = html;
        }}

        // 渲染性能洞察
        function renderPerformanceInsights(stats) {{
            const element = document.getElementById('performanceInsights');
            if (!element) return;
            
            const insights = [];
            
            if (stats.active_memory > 1024 * 1024) {{
                insights.push('🔍 High memory usage detected');
            }}
            
            if (stats.active_allocations > 1000) {{
                insights.push('📊 Many active allocations');
            }}
            
            if (insights.length === 0) {{
                insights.push('✅ Memory usage looks healthy');
            }}
            
            const html = insights.map(insight => `<div class="insight-item">${{insight}}</div>`).join('');
            element.innerHTML = html;
        }}

        // 初始化标签页导航
        function initializeTabs() {{
            const tabButtons = document.querySelectorAll('.tab-btn');
            const tabContents = document.querySelectorAll('.tab-content');
            
            tabButtons.forEach(button => {{
                button.addEventListener('click', () => {{
                    const targetTab = button.getAttribute('data-tab');
                    
                    // 移除所有活动状态
                    tabButtons.forEach(btn => btn.classList.remove('active'));
                    tabContents.forEach(content => content.classList.remove('active'));
                    
                    // 激活当前标签
                    button.classList.add('active');
                    const targetContent = document.getElementById(targetTab);
                    if (targetContent) {{
                        targetContent.classList.add('active');
                    }}
                }});
            }});
        }}

        // 显示错误状态
        function showErrorState(error) {{
            const container = document.querySelector('.container');
            if (!container) return;
            
            container.innerHTML = `
                <div class="error-state" style="text-align: center; padding: 60px 20px; color: #e74c3c;">
                    <h2>❌ Error Loading Data</h2>
                    <p>Failed to load memory analysis data: ${{error.message}}</p>
                    <button onclick="location.reload()" style="
                        padding: 10px 20px; background: #3498db; color: white;
                        border: none; border-radius: 5px; cursor: pointer; margin-top: 20px;
                    ">Reload Page</button>
                </div>
            `;
        }}

        // 格式化字节数
        function formatBytes(bytes) {{
            if (bytes === 0) return '0 B';
            
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }}
        
        // 页面加载完成后初始化
        document.addEventListener('DOMContentLoaded', function() {{
            initializeMemScopeApp();
        }});
    </script>
</body>
</html>"#);

        Ok(html)
    }
    
    /// Calculate simple hash for caching
    fn calculate_hash(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Get cache hit rate
    fn get_cache_hit_rate(&self) -> f64 {
        if let Some(cache) = TEMPLATE_CACHE.get() {
            if let Ok(cache_guard) = cache.lock() {
                let (_, _, hit_rate) = cache_guard.get_stats();
                return hit_rate;
            }
        }
        0.0
    }
}

impl Default for TemplateGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes into human-readable string
fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}