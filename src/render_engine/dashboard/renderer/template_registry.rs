//! Template registry - manages dashboard template loading and selection
//!
//! This module provides a centralized registry for dashboard templates,
//! supporting both built-in templates and external template files from
//! the templetes/ directory.

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use super::helpers::register_helpers;

/// Both rendering APIs share one immutable compiled built-in registry.
pub(super) fn builtin_handlebars(
) -> Result<Arc<Handlebars<'static>>, Arc<handlebars::TemplateError>> {
    static BUILTIN: OnceLock<Result<Arc<Handlebars<'static>>, Arc<handlebars::TemplateError>>> =
        OnceLock::new();
    BUILTIN
        .get_or_init(|| {
            let mut handlebars = Handlebars::new();
            register_helpers(&mut handlebars);
            for (name, source) in [
                (
                    "dashboard_modes",
                    include_str!("../templates/partials/modes.hbs"),
                ),
                (
                    "dashboard_runtime",
                    include_str!("../templates/partials/runtime.hbs"),
                ),
                (
                    "dashboard_charts",
                    include_str!("../templates/partials/charts.hbs"),
                ),
                (
                    "dashboard_graphs",
                    include_str!("../templates/partials/graphs.hbs"),
                ),
                (
                    "dashboard_interactions",
                    include_str!("../templates/partials/interactions.hbs"),
                ),
            ] {
                handlebars
                    .register_template_string(name, source)
                    .map_err(Arc::new)?;
            }
            handlebars
                .register_template_string(
                    "dashboard_unified",
                    include_str!("../templates/dashboard_unified.html"),
                )
                .map_err(Arc::new)?;
            Ok(Arc::new(handlebars))
        })
        .clone()
}

/// Registered dashboard template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplate {
    /// Unique template identifier
    pub id: String,
    /// Human-readable display name
    pub name: String,
    /// Description of what this template visualizes
    pub description: String,
    /// File path to the Handlebars HTML template
    pub template_path: PathBuf,
    /// Whether this is a built-in or external template
    #[serde(default)]
    pub kind: TemplateKind,
}

/// Template source classification
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateKind {
    /// Built-in template bundled in the crate
    #[default]
    BuiltIn,
    /// External template loaded from file system
    External,
}

/// Template registry - holds all available dashboard templates
pub struct TemplateRegistry {
    /// Map of template ID to DashboardTemplate
    templates: HashMap<String, DashboardTemplate>,
    /// Handlebars instance with all templates registered
    handlebars: Arc<Handlebars<'static>>,
    /// Base directory for external templates
    external_base: Option<PathBuf>,
}

impl TemplateRegistry {
    /// Create a new empty template registry
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        // Register all Handlebars helpers (format_bytes, eq, len, risk_class, ...)
        // so templates rendered through this registry can use them. Without this,
        // helper calls like {{len thread_policies}} would be misinterpreted as
        // field accesses and fail with "Cannot access array/vector with string
        // index" errors.
        register_helpers(&mut handlebars);
        Self {
            templates: HashMap::new(),
            handlebars: Arc::new(handlebars),
            external_base: None,
        }
    }

    /// Create a registry with the single built-in merged dashboard template pre-loaded.
    ///
    /// The merged template lives at
    /// `src/render_engine/dashboard/templates/dashboard_unified.html` and bundles all
    /// eight dashboard modes (Overview, Threads, Async, Task Graph, Variables,
    /// Passports, FFI, Unsafe/Time) into one HTML file with a side-bar mode switcher.
    /// The `templates_dir` argument is kept for API compatibility but no longer used.
    pub fn with_built_in_templates(
        templates_dir: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let _ = templates_dir; // unused: single merged template location is fixed
        let mut registry = Self {
            templates: HashMap::new(),
            handlebars: builtin_handlebars()?,
            external_base: None,
        };

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let unified_path = manifest_dir
            .join("src")
            .join("render_engine")
            .join("dashboard")
            .join("templates")
            .join("dashboard_unified.html");
        registry.templates.insert("dashboard_unified".into(), DashboardTemplate {
                id: "dashboard_unified".to_string(),
                name: "Unified Dashboard".to_string(),
                description: "Merged multi-mode dashboard (Overview, Threads, Async, Task Graph, Variables, Passports, FFI, Unsafe/Time)".to_string(),
                template_path: unified_path,
                kind: TemplateKind::BuiltIn,
        });

        Ok(registry)
    }

    /// Set the base directory for external templates
    pub fn set_external_base(&mut self, path: PathBuf) {
        self.external_base = Some(path);
    }

    /// Load external templates from the templetes/ directory
    ///
    /// Scans subdirectories for code.html files and registers them.
    /// Each subdirectory becomes a template with the directory name as the ID.
    pub fn load_external_templates(
        &mut self,
        templetes_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !templetes_dir.exists() {
            tracing::warn!("Templetes directory not found: {:?}", templetes_dir);
            return Ok(());
        }

        let entries = fs::read_dir(templetes_dir)?;
        for entry in entries {
            let entry = entry?;
            let dir_path = entry.path();

            if !dir_path.is_dir() {
                continue;
            }

            let code_html = dir_path.join("code.html");
            if !code_html.exists() {
                continue;
            }

            // Derive template ID from directory name (kebab-case → snake_case)
            let dir_name = dir_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let id = dir_name.replace('-', "_").replace(" ", "_");

            // Derive display name from directory name
            let name = dir_name
                .split(['_', '-'])
                .filter(|s| !s.is_empty())
                .map(|s| {
                    s.chars().next().unwrap_or(' ').to_uppercase().to_string()
                        + &s[1..].to_lowercase()
                })
                .collect::<Vec<_>>()
                .join(" ");

            self.register_template(DashboardTemplate {
                id: format!("ext_{}", id),
                name: format!("Professional: {}", name),
                description: format!("External template: {}", name),
                template_path: code_html,
                kind: TemplateKind::External,
            })?;
        }

        tracing::info!(
            "Loaded {} external templates from {:?}",
            self.templates
                .iter()
                .filter(|(_, t)| t.kind == TemplateKind::External)
                .count(),
            templetes_dir
        );

        Ok(())
    }

    /// Register a single template
    fn register_template(
        &mut self,
        template: DashboardTemplate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let id = template.id.clone();
        // External templates belong only to this registry. Clone on mutation,
        // preserving the process-wide built-ins and other renderer instances.
        Arc::make_mut(&mut self.handlebars).register_template_file(&id, &template.template_path)?;
        self.templates.insert(id, template);
        Ok(())
    }

    /// Get all available templates
    pub fn templates(&self) -> &HashMap<String, DashboardTemplate> {
        &self.templates
    }

    /// Get a specific template by ID
    pub fn get_template(&self, id: &str) -> Option<&DashboardTemplate> {
        self.templates.get(id)
    }

    /// Render a dashboard using the specified template
    pub fn render(
        &self,
        template_id: &str,
        data: &serde_json::Value,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let template = self.templates.get(template_id).ok_or_else(|| {
            format!(
                "Template '{}' not found. Available: {:?}",
                template_id,
                self.template_ids()
            )
        })?;

        let result = self.handlebars.render(template_id, data);
        match result {
            Ok(html) => Ok(html),
            Err(e) => Err(format!(
                "Failed to render template '{}': {} ({:?})",
                template.name, e, template.template_path
            )
            .into()),
        }
    }

    /// List all available template IDs
    pub fn template_ids(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Count of available templates
    pub fn count(&self) -> usize {
        self.templates.len()
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    /// Objective: Cached built-ins cannot leak externally loaded templates between instances.
    /// Invariants: Independent registries retain their own template source and rendered data.
    #[test]
    fn external_templates_remain_instance_local() {
        let dir = tempfile::tempdir().expect("external fixture directory");
        let custom = dir.path().join("custom");
        fs::create_dir(&custom).expect("external template directory");
        let path = custom.join("code.html");
        fs::write(&path, "first: {{value}}").expect("write first template");
        let mut first = TemplateRegistry::with_built_in_templates(dir.path()).expect("built-ins");
        first
            .load_external_templates(dir.path())
            .expect("load first template");
        fs::write(&path, "second: {{value}}").expect("update external template");
        let mut second = TemplateRegistry::with_built_in_templates(dir.path()).expect("built-ins");
        second
            .load_external_templates(dir.path())
            .expect("reload updated template");
        let data = serde_json::json!({"value":42});
        assert_eq!(
            first.render("ext_custom", &data).unwrap(),
            "first: 42",
            "first instance retains its source"
        );
        assert_eq!(
            second.render("ext_custom", &data).unwrap(),
            "second: 42",
            "new instance observes external edits"
        );
        let builtin = builtin_handlebars().expect("compiled built-ins");
        assert!(
            !builtin.has_template("ext_custom"),
            "external writes cannot mutate shared built-ins"
        );
        assert!(
            builtin.has_template("dashboard_unified"),
            "embedded dashboard is always available"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = TemplateRegistry::new();
        assert_eq!(registry.count(), 0);
        assert!(registry.template_ids().is_empty());
    }

    #[test]
    fn test_register_and_retrieve() {
        let registry = TemplateRegistry::new();
        let templates = registry.templates();
        assert!(templates.is_empty());
        // Cannot fully test without a real file, but structure is verified
    }
}
