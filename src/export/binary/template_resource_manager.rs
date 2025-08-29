//! Template resource management for binary to HTML conversion
//!
//! This module provides comprehensive resource management for HTML templates,
//! including CSS/JS embedding, shared resource loading, and placeholder processing.

use crate::export::binary::error::BinaryExportError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Template resource manager for handling CSS/JS resources and placeholders
pub struct TemplateResourceManager {
    /// Base template directory path
    template_dir: PathBuf,
    /// Cached CSS content
    css_cache: HashMap<String, String>,
    /// Cached JS content
    js_cache: HashMap<String, String>,
    /// SVG images cache
    svg_cache: HashMap<String, String>,
    /// Placeholder processors
    placeholder_processors: HashMap<String, Box<dyn PlaceholderProcessor>>,
}

/// Trait for processing template placeholders
pub trait PlaceholderProcessor: Send + Sync {
    /// Process a placeholder with given data
    fn process(&self, data: &TemplateData) -> Result<String, BinaryExportError>;
}

/// Template data structure for placeholder processing
#[derive(Debug, Clone)]
pub struct TemplateData {
    /// Project name
    pub project_name: String,
    /// Binary analysis data (JSON string)
    pub binary_data: String,
    /// Generation timestamp
    pub generation_time: String,
    /// CSS content
    pub css_content: String,
    /// JavaScript content
    pub js_content: String,
    /// SVG images content
    pub svg_images: String,
    /// Additional custom data
    pub custom_data: HashMap<String, String>,
}

/// Resource embedding configuration
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    /// Whether to embed CSS inline
    pub embed_css: bool,
    /// Whether to embed JS inline
    pub embed_js: bool,
    /// Whether to embed SVG images
    pub embed_svg: bool,
    /// Whether to minify resources
    pub minify_resources: bool,
    /// Custom resource paths
    pub custom_paths: HashMap<String, PathBuf>,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            embed_css: true,
            embed_js: true,
            embed_svg: true,
            minify_resources: false,
            custom_paths: HashMap::new(),
        }
    }
}

impl TemplateResourceManager {
    /// Create a new template resource manager
    pub fn new<P: AsRef<Path>>(template_dir: P) -> Result<Self, BinaryExportError> {
        let template_dir = template_dir.as_ref().to_path_buf();

        if !template_dir.exists() {
            return Err(BinaryExportError::CorruptedData(format!(
                "Template directory does not exist: {}",
                template_dir.display()
            )));
        }

        let mut manager = Self {
            template_dir,
            css_cache: HashMap::new(),
            js_cache: HashMap::new(),
            svg_cache: HashMap::new(),
            placeholder_processors: HashMap::new(),
        };

        // Register default placeholder processors
        manager.register_default_processors();

        Ok(manager)
    }

    /// Load and process a template with resources
    pub fn process_template(
        &mut self,
        template_name: &str,
        data: &TemplateData,
        config: &ResourceConfig,
    ) -> Result<String, BinaryExportError> {
        // Load template content
        let template_path = self.template_dir.join(template_name);
        let mut template_content =
            fs::read_to_string(&template_path).map_err(|e| BinaryExportError::Io(e))?;

        // Load and embed resources
        if config.embed_css {
            let css_content = if !data.css_content.is_empty() {
                data.css_content.clone()
            } else {
                self.load_css_resources(config)?
            };
            template_content = template_content.replace("{{CSS_CONTENT}}", &css_content);
        }

        if config.embed_js {
            let js_content = if !data.js_content.is_empty() {
                data.js_content.clone()
            } else {
                self.load_js_resources(config)?
            };
            template_content = template_content.replace("{{JS_CONTENT}}", &js_content);
        }

        if config.embed_svg {
            let svg_content = self.load_svg_resources(config)?;
            template_content = template_content.replace("{{SVG_IMAGES}}", &svg_content);
        }

        // Process all placeholders
        template_content = self.process_placeholders(template_content, data)?;

        Ok(template_content)
    }

    /// Load CSS resources from templates directory
    fn load_css_resources(&mut self, config: &ResourceConfig) -> Result<String, BinaryExportError> {
        let css_files = vec!["styles.css"];
        let mut combined_css = String::new();

        for css_file in css_files {
            if let Some(cached) = self.css_cache.get(css_file) {
                combined_css.push_str(cached);
                combined_css.push('\n');
                continue;
            }

            let css_path = self.template_dir.join(css_file);
            if css_path.exists() {
                let css_content =
                    fs::read_to_string(&css_path).map_err(|e| BinaryExportError::Io(e))?;

                let processed_css = if config.minify_resources {
                    self.minify_css(&css_content)
                } else {
                    css_content
                };

                self.css_cache
                    .insert(css_file.to_string(), processed_css.clone());
                combined_css.push_str(&processed_css);
                combined_css.push('\n');
            }
        }

        Ok(combined_css)
    }

    /// Load JavaScript resources from templates directory
    fn load_js_resources(&mut self, config: &ResourceConfig) -> Result<String, BinaryExportError> {
        let js_files = vec!["script.js"];
        let mut combined_js = String::new();

        for js_file in js_files {
            if let Some(cached) = self.js_cache.get(js_file) {
                combined_js.push_str(cached);
                combined_js.push('\n');
                continue;
            }

            let js_path = self.template_dir.join(js_file);
            if js_path.exists() {
                let js_content =
                    fs::read_to_string(&js_path).map_err(|e| BinaryExportError::Io(e))?;

                let processed_js = if config.minify_resources {
                    self.minify_js(&js_content)
                } else {
                    js_content
                };

                self.js_cache
                    .insert(js_file.to_string(), processed_js.clone());
                combined_js.push_str(&processed_js);
                combined_js.push('\n');
            }
        }

        Ok(combined_js)
    }

    /// Load SVG resources from templates directory
    fn load_svg_resources(
        &mut self,
        _config: &ResourceConfig,
    ) -> Result<String, BinaryExportError> {
        // For now, return empty string as SVG embedding is not implemented
        // In a real implementation, this would scan for SVG files and embed them
        Ok(String::new())
    }

    /// Process all placeholders in template content
    fn process_placeholders(
        &self,
        mut content: String,
        data: &TemplateData,
    ) -> Result<String, BinaryExportError> {
        // Process standard placeholders (handle both with and without spaces)
        content = content.replace("{{PROJECT_NAME}}", &data.project_name);
        content = content.replace("{{ PROJECT_NAME }}", &data.project_name);
        content = content.replace("{{BINARY_DATA}}", &data.binary_data);
        content = content.replace("{{ BINARY_DATA }}", &data.binary_data);
        content = content.replace("{{json_data}}", &data.binary_data);
        content = content.replace("{{ json_data }}", &data.binary_data);

        // Fix the specific template issue - inject data as window.analysisData assignment
        // Handle all possible variations of the template placeholder
        content = content.replace(
            "window.analysisData = {{ json_data }};",
            &format!("window.analysisData = {};", &data.binary_data),
        );
        content = content.replace(
            "window.analysisData = {{json_data}};",
            &format!("window.analysisData = {};", &data.binary_data),
        );
        content = content.replace(
            "window.analysisData = {{ json_data}};",
            &format!("window.analysisData = {};", &data.binary_data),
        );
        content = content.replace(
            "window.analysisData = {{json_data }};",
            &format!("window.analysisData = {};", &data.binary_data),
        );

        // Also handle cases where there might be line breaks or spaces
        content = content.replace(
            "window.analysisData = {{ json_data",
            &format!("window.analysisData = {}", &data.binary_data),
        );
        content = content.replace(
            "window.analysisData = {{json_data",
            &format!("window.analysisData = {}", &data.binary_data),
        );
        content = content.replace("{{GENERATION_TIME}}", &data.generation_time);
        content = content.replace("{{ GENERATION_TIME }}", &data.generation_time);

        // Process additional placeholders from custom data
        for (key, value) in &data.custom_data {
            let placeholder_with_spaces = format!("{{{{ {} }}}}", key);
            let placeholder_without_spaces = format!("{{{{{}}}}}", key);
            content = content.replace(&placeholder_with_spaces, value);
            content = content.replace(&placeholder_without_spaces, value);
        }

        // Handle common placeholders that might be in custom data
        if let Some(processing_time) = data.custom_data.get("PROCESSING_TIME") {
            content = content.replace("{{PROCESSING_TIME}}", processing_time);
            content = content.replace("{{ PROCESSING_TIME }}", processing_time);
        }

        if let Some(svg_images) = data.custom_data.get("SVG_IMAGES") {
            content = content.replace("{{SVG_IMAGES}}", svg_images);
            content = content.replace("{{ SVG_IMAGES }}", svg_images);
        }

        // Process custom placeholders using registered processors
        for (placeholder, processor) in &self.placeholder_processors {
            let placeholder_pattern = format!("{{{{{}}}}}", placeholder);
            if content.contains(&placeholder_pattern) {
                let processed_value = processor.process(data)?;
                content = content.replace(&placeholder_pattern, &processed_value);
            }
        }

        Ok(content)
    }

    /// Register default placeholder processors
    fn register_default_processors(&mut self) {
        self.placeholder_processors
            .insert("MEMORY_DATA".to_string(), Box::new(MemoryDataProcessor));
        self.placeholder_processors.insert(
            "COMPLEX_TYPES_DATA".to_string(),
            Box::new(ComplexTypesProcessor),
        );
        self.placeholder_processors
            .insert("FFI_SAFETY_DATA".to_string(), Box::new(FfiSafetyProcessor));
        self.placeholder_processors.insert(
            "RELATIONSHIP_DATA".to_string(),
            Box::new(RelationshipProcessor),
        );
    }

    /// Register a custom placeholder processor
    pub fn register_processor(
        &mut self,
        placeholder: String,
        processor: Box<dyn PlaceholderProcessor>,
    ) {
        self.placeholder_processors.insert(placeholder, processor);
    }

    /// Simple CSS minification (removes comments and extra whitespace)
    fn minify_css(&self, css: &str) -> String {
        css.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("/*"))
            .collect::<Vec<_>>()
            .join(" ")
            .replace("  ", " ")
    }

    /// Simple JavaScript minification (removes comments and extra whitespace)
    fn minify_js(&self, js: &str) -> String {
        js.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect::<Vec<_>>()
            .join(" ")
            .replace("  ", " ")
    }

    /// Get shared resource content for external use
    pub fn get_shared_css(&mut self, config: &ResourceConfig) -> Result<String, BinaryExportError> {
        self.load_css_resources(config)
    }

    /// Get shared JavaScript content for external use
    pub fn get_shared_js(&mut self, config: &ResourceConfig) -> Result<String, BinaryExportError> {
        self.load_js_resources(config)
    }

    /// Clear resource caches
    pub fn clear_cache(&mut self) {
        self.css_cache.clear();
        self.js_cache.clear();
        self.svg_cache.clear();
    }
}

/// Memory data placeholder processor
struct MemoryDataProcessor;

impl PlaceholderProcessor for MemoryDataProcessor {
    fn process(&self, data: &TemplateData) -> Result<String, BinaryExportError> {
        // Extract memory-specific data from binary data
        // This is a simplified implementation
        Ok(data.binary_data.clone())
    }
}

/// Complex types data placeholder processor
struct ComplexTypesProcessor;

impl PlaceholderProcessor for ComplexTypesProcessor {
    fn process(&self, data: &TemplateData) -> Result<String, BinaryExportError> {
        // Extract complex types data from binary data
        // This would parse the JSON and extract complex_types section
        if let Some(complex_types_data) = data.custom_data.get("complex_types") {
            Ok(complex_types_data.clone())
        } else {
            Ok("{}".to_string())
        }
    }
}

/// FFI safety data placeholder processor
struct FfiSafetyProcessor;

impl PlaceholderProcessor for FfiSafetyProcessor {
    fn process(&self, data: &TemplateData) -> Result<String, BinaryExportError> {
        // Extract FFI safety data from binary data
        if let Some(ffi_data) = data.custom_data.get("unsafe_ffi") {
            Ok(ffi_data.clone())
        } else {
            Ok("{}".to_string())
        }
    }
}

/// Variable relationship data placeholder processor
struct RelationshipProcessor;

impl PlaceholderProcessor for RelationshipProcessor {
    fn process(&self, data: &TemplateData) -> Result<String, BinaryExportError> {
        // Extract relationship data from binary data
        if let Some(relationship_data) = data.custom_data.get("variable_relationships") {
            Ok(relationship_data.clone())
        } else {
            Ok("{}".to_string())
        }
    }
}

/// Utility function to create template data from binary analysis results
pub fn create_template_data(
    project_name: &str,
    binary_data_json: &str,
    custom_data: HashMap<String, String>,
) -> TemplateData {
    let generation_time = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();

    TemplateData {
        project_name: project_name.to_string(),
        binary_data: binary_data_json.to_string(),
        generation_time,
        css_content: String::new(),
        js_content: String::new(),
        svg_images: String::new(),
        custom_data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_template_dir() -> Result<TempDir, std::io::Error> {
        let temp_dir = TempDir::new()?;

        // Create test template file
        let template_content = r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{PROJECT_NAME}}</title>
    <style>{{CSS_CONTENT}}</style>
</head>
<body>
    <div id="data">{{BINARY_DATA}}</div>
    <script>{{JS_CONTENT}}</script>
</body>
</html>
"#;
        fs::write(temp_dir.path().join("test_template.html"), template_content)?;

        // Create test CSS file
        let css_content = "body { margin: 0; padding: 0; }";
        fs::write(temp_dir.path().join("styles.css"), css_content)?;

        // Create test JS file
        let js_content = "console.log('Test script loaded');";
        fs::write(temp_dir.path().join("script.js"), js_content)?;

        Ok(temp_dir)
    }

    #[test]
    fn test_template_resource_manager_creation() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let manager = TemplateResourceManager::new(temp_dir.path());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_template_processing() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let mut manager =
            TemplateResourceManager::new(temp_dir.path()).expect("Test operation failed");

        let template_data = TemplateData {
            project_name: "Test Project".to_string(),
            binary_data: r#"{"test": "data"}"#.to_string(),
            generation_time: "2024-01-01 12:00:00 UTC".to_string(),
            css_content: String::new(),
            js_content: String::new(),
            svg_images: String::new(),
            custom_data: HashMap::new(),
        };

        let config = ResourceConfig::default();
        let result = manager.process_template("test_template.html", &template_data, &config);

        assert!(result.is_ok());
        let processed = result.expect("Test operation failed");
        assert!(processed.contains("Test Project"));
        assert!(processed.contains(r#"{"test": "data"}"#));
        assert!(processed.contains("body { margin: 0; padding: 0; }"));
        assert!(processed.contains("console.log('Test script loaded');"));
    }

    #[test]
    fn test_css_loading() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let mut manager =
            TemplateResourceManager::new(temp_dir.path()).expect("Test operation failed");
        let config = ResourceConfig::default();

        let css_content = manager
            .get_shared_css(&config)
            .expect("Test operation failed");
        assert!(css_content.contains("body { margin: 0; padding: 0; }"));
    }

    #[test]
    fn test_js_loading() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let mut manager =
            TemplateResourceManager::new(temp_dir.path()).expect("Test operation failed");
        let config = ResourceConfig::default();

        let js_content = manager
            .get_shared_js(&config)
            .expect("Test operation failed");
        assert!(js_content.contains("console.log('Test script loaded');"));
    }

    #[test]
    fn test_css_minification() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let _manager =
            TemplateResourceManager::new(temp_dir.path()).expect("Test operation failed");

        let css = "body {\n    margin: 0;\n    padding: 0;\n}";
        let minified = _manager.minify_css(css);
        assert!(!minified.contains('\n'));
        assert!(minified.len() < css.len());
    }

    #[test]
    fn test_placeholder_processors() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let _manager =
            TemplateResourceManager::new(temp_dir.path()).expect("Test operation failed");

        let mut custom_data = HashMap::new();
        custom_data.insert("complex_types".to_string(), r#"{"types": []}"#.to_string());

        let template_data = TemplateData {
            project_name: "Test".to_string(),
            binary_data: "{}".to_string(),
            generation_time: "2024-01-01".to_string(),
            css_content: String::new(),
            js_content: String::new(),
            svg_images: String::new(),
            custom_data,
        };

        let processor = ComplexTypesProcessor;
        let result = processor
            .process(&template_data)
            .expect("Test operation failed");
        assert_eq!(result, r#"{"types": []}"#);
    }

    #[test]
    fn test_cache_functionality() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let mut manager =
            TemplateResourceManager::new(temp_dir.path()).expect("Test operation failed");
        let config = ResourceConfig::default();

        // First load should read from file
        let css1 = manager
            .get_shared_css(&config)
            .expect("Test operation failed");

        // Second load should use cache
        let css2 = manager
            .get_shared_css(&config)
            .expect("Test operation failed");

        assert_eq!(css1, css2);
        assert!(!manager.css_cache.is_empty());

        // Clear cache
        manager.clear_cache();
        assert!(manager.css_cache.is_empty());
    }

    #[test]
    fn test_template_data_creation() {
        let mut custom_data = HashMap::new();
        custom_data.insert("test_key".to_string(), "test_value".to_string());

        let template_data = create_template_data("My Project", r#"{"data": "test"}"#, custom_data);

        assert_eq!(template_data.project_name, "My Project");
        assert_eq!(template_data.binary_data, r#"{"data": "test"}"#);
        assert!(template_data.generation_time.contains("UTC"));
        assert_eq!(
            template_data.custom_data.get("test_key"),
            Some(&"test_value".to_string())
        );
    }
}
