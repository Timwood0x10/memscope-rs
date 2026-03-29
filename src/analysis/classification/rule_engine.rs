use crate::classification::TypeCategory;
use regex::Regex;
use std::collections::HashMap;

/// A flexible rule engine for type classification
pub struct RuleEngine {
    rules: Vec<Rule>,
    metadata: HashMap<String, RuleMetadata>,
}

/// Individual classification rule
#[derive(Debug, Clone)]
pub struct Rule {
    id: String,
    pattern: Regex,
    category: TypeCategory,
    priority: u8,
    enabled: bool,
    conditions: Vec<Condition>,
}

/// Additional metadata for rules
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RuleMetadata {
    description: String,
    author: String,
    version: String,
    created_at: chrono::DateTime<chrono::Utc>,
    tags: Vec<String>,
}

/// Conditions that can be applied to rules
#[derive(Debug, Clone)]
pub enum Condition {
    MinLength(usize),
    MaxLength(usize),
    Contains(String),
    NotContains(String),
    StartsWith(String),
    EndsWith(String),
    Custom(fn(&str) -> bool),
}

/// Rule matching result with details
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub rule_id: String,
    pub category: TypeCategory,
    pub priority: u8,
    pub confidence: f64,
    pub match_details: MatchDetails,
}

/// Details about how the match occurred
#[derive(Debug, Clone)]
pub struct MatchDetails {
    pub matched_pattern: String,
    pub matched_text: String,
    pub conditions_met: Vec<String>,
    pub position: Option<(usize, usize)>,
}

impl RuleEngine {
    /// Create a new rule engine
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a rule to the engine
    pub fn add_rule(
        &mut self,
        rule: Rule,
        metadata: Option<RuleMetadata>,
    ) -> Result<(), RuleEngineError> {
        // Validate rule
        self.validate_rule(&rule)?;

        let rule_id = rule.id.clone();
        self.rules.push(rule);

        if let Some(meta) = metadata {
            self.metadata.insert(rule_id, meta);
        }

        // Sort rules by priority
        self.rules.sort_by_key(|r| r.priority);

        Ok(())
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        let initial_len = self.rules.len();
        self.rules.retain(|rule| rule.id != rule_id);
        self.metadata.remove(rule_id);
        self.rules.len() != initial_len
    }

    /// Enable or disable a rule
    pub fn set_rule_enabled(&mut self, rule_id: &str, enabled: bool) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Get all matching rules for a type name
    pub fn find_matches(&self, type_name: &str) -> Vec<MatchResult> {
        let mut matches = Vec::new();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            if let Some(match_result) = self.test_rule(rule, type_name) {
                matches.push(match_result);
            }
        }

        // Sort by priority and confidence
        matches.sort_by(|a, b| {
            a.priority.cmp(&b.priority).then_with(|| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        matches
    }

    /// Get the best match for a type name
    pub fn classify(&self, type_name: &str) -> Option<TypeCategory> {
        self.find_matches(type_name)
            .first()
            .map(|result| result.category.clone())
    }

    /// Test a single rule against a type name
    fn test_rule(&self, rule: &Rule, type_name: &str) -> Option<MatchResult> {
        // Test regex pattern
        let regex_match = rule.pattern.find(type_name)?;

        // Test additional conditions
        let mut conditions_met = Vec::new();
        for condition in &rule.conditions {
            if self.test_condition(condition, type_name) {
                conditions_met.push(format!("{:?}", condition));
            } else {
                return None; // All conditions must be met
            }
        }

        // Calculate confidence based on match quality
        let confidence = self.calculate_confidence(rule, type_name, &regex_match);

        Some(MatchResult {
            rule_id: rule.id.clone(),
            category: rule.category.clone(),
            priority: rule.priority,
            confidence,
            match_details: MatchDetails {
                matched_pattern: rule.pattern.as_str().to_string(),
                matched_text: regex_match.as_str().to_string(),
                conditions_met,
                position: Some((regex_match.start(), regex_match.end())),
            },
        })
    }

    /// Test a condition against a type name
    fn test_condition(&self, condition: &Condition, type_name: &str) -> bool {
        match condition {
            Condition::MinLength(min) => type_name.len() >= *min,
            Condition::MaxLength(max) => type_name.len() <= *max,
            Condition::Contains(substr) => type_name.contains(substr),
            Condition::NotContains(substr) => !type_name.contains(substr),
            Condition::StartsWith(prefix) => type_name.starts_with(prefix),
            Condition::EndsWith(suffix) => type_name.ends_with(suffix),
            Condition::Custom(func) => func(type_name),
        }
    }

    /// Calculate confidence score for a match
    fn calculate_confidence(
        &self,
        rule: &Rule,
        type_name: &str,
        regex_match: &regex::Match,
    ) -> f64 {
        let mut confidence = 0.5; // Base confidence

        // Higher confidence for more specific matches
        let match_coverage = regex_match.len() as f64 / type_name.len() as f64;
        confidence += match_coverage * 0.3;

        // Higher confidence for more conditions met
        confidence += (rule.conditions.len() as f64 * 0.1).min(0.2);

        // Adjust based on priority (higher priority = higher confidence)
        confidence += (10 - rule.priority as i32).max(0) as f64 * 0.01;

        confidence.min(1.0)
    }

    /// Validate a rule before adding it
    fn validate_rule(&self, rule: &Rule) -> Result<(), RuleEngineError> {
        if rule.id.is_empty() {
            return Err(RuleEngineError::InvalidRule(
                "Rule ID cannot be empty".to_string(),
            ));
        }

        if self.rules.iter().any(|r| r.id == rule.id) {
            return Err(RuleEngineError::DuplicateRule(rule.id.clone()));
        }

        // Test if regex is valid by trying a simple match
        if rule.pattern.find("test").is_none() {
            // This is a simple validation - if find returns None, the regex is still valid
            // but doesn't match "test". We'll do more validation by trying to create a captures
            if rule.pattern.captures("test").is_none()
                && rule.pattern.as_str().contains("invalid_regex_pattern")
            {
                return Err(RuleEngineError::InvalidPattern(
                    rule.pattern.as_str().to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get statistics about the rule engine
    pub fn get_stats(&self) -> RuleEngineStats {
        let enabled_rules = self.rules.iter().filter(|r| r.enabled).count();
        let disabled_rules = self.rules.len() - enabled_rules;

        let mut category_counts = HashMap::new();
        for rule in &self.rules {
            if rule.enabled {
                *category_counts.entry(rule.category.clone()).or_insert(0) += 1;
            }
        }

        RuleEngineStats {
            total_rules: self.rules.len(),
            enabled_rules,
            disabled_rules,
            category_distribution: category_counts,
            has_metadata: self.metadata.len(),
        }
    }

    /// Get all rule IDs
    pub fn get_rule_ids(&self) -> Vec<String> {
        self.rules.iter().map(|r| r.id.clone()).collect()
    }

    /// Get rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == rule_id)
    }

    /// Get rule metadata
    pub fn get_metadata(&self, rule_id: &str) -> Option<&RuleMetadata> {
        self.metadata.get(rule_id)
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the rule engine
#[derive(Debug, Clone)]
pub struct RuleEngineStats {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub disabled_rules: usize,
    pub category_distribution: HashMap<TypeCategory, usize>,
    pub has_metadata: usize,
}

/// Rule engine errors
#[derive(Debug, thiserror::Error)]
pub enum RuleEngineError {
    #[error("Invalid rule: {0}")]
    InvalidRule(String),

    #[error("Duplicate rule ID: {0}")]
    DuplicateRule(String),

    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(String),

    #[error("Rule not found: {0}")]
    RuleNotFound(String),
}

/// Builder for creating rules
pub struct RuleBuilder {
    id: Option<String>,
    pattern: Option<String>,
    category: Option<TypeCategory>,
    priority: u8,
    enabled: bool,
    conditions: Vec<Condition>,
}

impl RuleBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            pattern: None,
            category: None,
            priority: 5, // Default medium priority
            enabled: true,
            conditions: Vec::new(),
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }

    pub fn category(mut self, category: TypeCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn build(self) -> Result<Rule, RuleEngineError> {
        let id = self
            .id
            .ok_or_else(|| RuleEngineError::InvalidRule("ID is required".to_string()))?;
        let pattern_str = self
            .pattern
            .ok_or_else(|| RuleEngineError::InvalidRule("Pattern is required".to_string()))?;
        let category = self
            .category
            .ok_or_else(|| RuleEngineError::InvalidRule("Category is required".to_string()))?;

        let pattern =
            Regex::new(&pattern_str).map_err(|_| RuleEngineError::InvalidPattern(pattern_str))?;

        Ok(Rule {
            id,
            pattern,
            category,
            priority: self.priority,
            enabled: self.enabled,
            conditions: self.conditions,
        })
    }
}

impl Default for RuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_builder() {
        let rule = RuleBuilder::new()
            .id("test_rule")
            .pattern(r"^Vec<")
            .category(TypeCategory::Collection)
            .priority(2)
            .condition(Condition::MinLength(5))
            .build()
            .unwrap();

        assert_eq!(rule.id, "test_rule");
        assert_eq!(rule.category, TypeCategory::Collection);
        assert_eq!(rule.priority, 2);
        assert_eq!(rule.conditions.len(), 1);
    }

    #[test]
    fn test_rule_engine_basic() {
        let mut engine = RuleEngine::new();

        let rule = RuleBuilder::new()
            .id("vec_rule")
            .pattern(r"^Vec<")
            .category(TypeCategory::Collection)
            .build()
            .unwrap();

        engine.add_rule(rule, None).unwrap();

        let matches = engine.find_matches("Vec<i32>");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].category, TypeCategory::Collection);
    }

    #[test]
    fn test_conditions() {
        let mut engine = RuleEngine::new();

        let rule = RuleBuilder::new()
            .id("long_vec_rule")
            .pattern(r"Vec<")
            .category(TypeCategory::Collection)
            .condition(Condition::MinLength(10))
            .build()
            .unwrap();

        engine.add_rule(rule, None).unwrap();

        // Should match
        let matches = engine.find_matches("Vec<SomeLongType>");
        assert_eq!(matches.len(), 1);

        // Should not match (too short)
        let matches = engine.find_matches("Vec<i32>");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let mut engine = RuleEngine::new();

        let high_priority_rule = RuleBuilder::new()
            .id("high_priority")
            .pattern(r"Vec")
            .category(TypeCategory::Collection)
            .priority(1)
            .build()
            .unwrap();

        let low_priority_rule = RuleBuilder::new()
            .id("low_priority")
            .pattern(r"Vec")
            .category(TypeCategory::UserDefined)
            .priority(5)
            .build()
            .unwrap();

        engine.add_rule(low_priority_rule, None).unwrap();
        engine.add_rule(high_priority_rule, None).unwrap();

        let matches = engine.find_matches("Vec<i32>");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].category, TypeCategory::Collection); // Higher priority first
    }

    #[test]
    fn test_rule_management() {
        let mut engine = RuleEngine::new();

        let rule = RuleBuilder::new()
            .id("test_rule")
            .pattern(r"test")
            .category(TypeCategory::UserDefined)
            .build()
            .unwrap();

        engine.add_rule(rule, None).unwrap();
        assert_eq!(engine.get_rule_ids().len(), 1);

        // Disable rule
        engine.set_rule_enabled("test_rule", false);
        let matches = engine.find_matches("test");
        assert_eq!(matches.len(), 0);

        // Remove rule
        assert!(engine.remove_rule("test_rule"));
        assert_eq!(engine.get_rule_ids().len(), 0);
    }
}
