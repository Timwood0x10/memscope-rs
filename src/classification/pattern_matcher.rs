use regex::Regex;
use std::collections::HashMap;

/// Advanced pattern matcher for type names with fuzzy matching capabilities
pub struct PatternMatcher {
    patterns: Vec<CompiledPattern>,
    fuzzy_threshold: f64,
    cache: std::sync::Mutex<HashMap<String, Vec<PatternMatch>>>,
}

/// A compiled pattern with metadata
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CompiledPattern {
    id: String,
    regex: Regex,
    weight: f64,
    tags: Vec<String>,
    description: String,
}

/// Result of a pattern match
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub score: f64,
    pub match_type: MatchType,
    pub captured_groups: Vec<String>,
    pub position: (usize, usize),
}

/// Type of match that occurred
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,
    Partial,
    Fuzzy,
    Substring,
    Prefix,
    Suffix,
}

impl PatternMatcher {
    /// Create a new pattern matcher
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            fuzzy_threshold: 0.7,
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Add a pattern to the matcher
    pub fn add_pattern(
        &mut self,
        id: &str,
        pattern: &str,
        weight: f64,
        description: &str,
    ) -> Result<(), PatternMatcherError> {
        let regex = Regex::new(pattern)
            .map_err(|e| PatternMatcherError::InvalidPattern(pattern.to_string(), e.to_string()))?;

        let compiled = CompiledPattern {
            id: id.to_string(),
            regex,
            weight,
            tags: Vec::new(),
            description: description.to_string(),
        };

        self.patterns.push(compiled);
        self.clear_cache();
        Ok(())
    }

    /// Add a pattern with tags
    pub fn add_pattern_with_tags(
        &mut self,
        id: &str,
        pattern: &str,
        weight: f64,
        description: &str,
        tags: Vec<String>,
    ) -> Result<(), PatternMatcherError> {
        let regex = Regex::new(pattern)
            .map_err(|e| PatternMatcherError::InvalidPattern(pattern.to_string(), e.to_string()))?;

        let compiled = CompiledPattern {
            id: id.to_string(),
            regex,
            weight,
            tags,
            description: description.to_string(),
        };

        self.patterns.push(compiled);
        self.clear_cache();
        Ok(())
    }

    /// Find all matches for a given input
    pub fn find_matches(&self, input: &str) -> Vec<PatternMatch> {
        // Check cache first
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached_matches) = cache.get(input) {
                return cached_matches.clone();
            }
        }

        let mut matches = Vec::new();

        // Test each pattern
        for pattern in &self.patterns {
            if let Some(pattern_match) = self.test_pattern(pattern, input) {
                matches.push(pattern_match);
            }
        }

        // Sort by score (descending)
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Cache the results
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(input.to_string(), matches.clone());
        }

        matches
    }

    /// Find the best match for a given input
    pub fn find_best_match(&self, input: &str) -> Option<PatternMatch> {
        self.find_matches(input).into_iter().next()
    }

    /// Find matches by tags
    pub fn find_matches_by_tag(&self, input: &str, tag: &str) -> Vec<PatternMatch> {
        let all_matches = self.find_matches(input);
        all_matches
            .into_iter()
            .filter(|m| {
                if let Some(pattern) = self.patterns.iter().find(|p| p.id == m.pattern_id) {
                    pattern.tags.contains(&tag.to_string())
                } else {
                    false
                }
            })
            .collect()
    }

    /// Test a single pattern against input
    fn test_pattern(&self, pattern: &CompiledPattern, input: &str) -> Option<PatternMatch> {
        // Try exact regex match first
        if let Some(regex_match) = pattern.regex.find(input) {
            let captured_groups = pattern
                .regex
                .captures(input)
                .map(|caps| {
                    caps.iter()
                        .skip(1)
                        .filter_map(|m| m.map(|m| m.as_str().to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let match_type = if regex_match.start() == 0 && regex_match.end() == input.len() {
                MatchType::Exact
            } else if regex_match.start() == 0 {
                MatchType::Prefix
            } else if regex_match.end() == input.len() {
                MatchType::Suffix
            } else {
                MatchType::Partial
            };

            let score = self.calculate_score(pattern, input, &regex_match, &match_type);

            return Some(PatternMatch {
                pattern_id: pattern.id.clone(),
                score,
                match_type,
                captured_groups,
                position: (regex_match.start(), regex_match.end()),
            });
        }

        // Try fuzzy matching if enabled
        if self.fuzzy_threshold > 0.0 {
            if let Some(fuzzy_match) = self.fuzzy_match(pattern, input) {
                return Some(fuzzy_match);
            }
        }

        None
    }

    /// Perform fuzzy matching
    fn fuzzy_match(&self, pattern: &CompiledPattern, input: &str) -> Option<PatternMatch> {
        // Simple fuzzy matching based on edit distance
        let pattern_str = pattern.regex.as_str();

        // Remove regex special characters for fuzzy matching
        let clean_pattern = self.clean_pattern_for_fuzzy(pattern_str);
        let similarity = self.calculate_similarity(&clean_pattern, input);

        if similarity >= self.fuzzy_threshold {
            Some(PatternMatch {
                pattern_id: pattern.id.clone(),
                score: similarity * pattern.weight * 0.8, // Fuzzy matches get lower score
                match_type: MatchType::Fuzzy,
                captured_groups: Vec::new(),
                position: (0, input.len()),
            })
        } else {
            None
        }
    }

    /// Calculate match score
    fn calculate_score(
        &self,
        pattern: &CompiledPattern,
        input: &str,
        regex_match: &regex::Match,
        match_type: &MatchType,
    ) -> f64 {
        let mut score = pattern.weight;

        // Bonus for match type
        let type_bonus = match match_type {
            MatchType::Exact => 1.0,
            MatchType::Prefix => 0.9,
            MatchType::Suffix => 0.8,
            MatchType::Partial => 0.7,
            MatchType::Substring => 0.6,
            MatchType::Fuzzy => 0.5,
        };
        score *= type_bonus;

        // Bonus for coverage
        let coverage = regex_match.len() as f64 / input.len() as f64;
        score *= 0.5 + coverage * 0.5;

        // Bonus for position (earlier matches are better)
        let position_bonus = 1.0 - (regex_match.start() as f64 / input.len() as f64) * 0.1;
        score *= position_bonus;

        score.min(1.0)
    }

    /// Clean regex pattern for fuzzy matching
    fn clean_pattern_for_fuzzy(&self, pattern: &str) -> String {
        // Remove common regex special characters
        pattern
            .replace("^", "")
            .replace("$", "")
            .replace("\\", "")
            .replace(".*", "")
            .replace(".+", "")
            .replace("?", "")
            .replace("*", "")
            .replace("+", "")
            .replace("(", "")
            .replace(")", "")
            .replace("[", "")
            .replace("]", "")
            .replace("{", "")
            .replace("}", "")
            .replace("|", "")
    }

    /// Calculate string similarity using Levenshtein distance
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 {
            return if len2 == 0 { 1.0 } else { 0.0 };
        }
        if len2 == 0 {
            return 0.0;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // Initialize first row and column
        for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
            row[0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        // Fill the matrix
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1, // deletion
                        matrix[i][j - 1] + 1, // insertion
                    ),
                    matrix[i - 1][j - 1] + cost, // substitution
                );
            }
        }

        let distance = matrix[len1][len2];
        let max_len = std::cmp::max(len1, len2);
        1.0 - (distance as f64 / max_len as f64)
    }

    /// Set fuzzy matching threshold
    pub fn set_fuzzy_threshold(&mut self, threshold: f64) {
        self.fuzzy_threshold = threshold.clamp(0.0, 1.0);
        self.clear_cache();
    }

    /// Get fuzzy matching threshold
    pub fn get_fuzzy_threshold(&self) -> f64 {
        self.fuzzy_threshold
    }

    /// Clear the match cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Get pattern statistics
    pub fn get_stats(&self) -> PatternMatcherStats {
        let cache = self.cache.lock().unwrap();
        let total_patterns = self.patterns.len();
        let cached_inputs = cache.len();

        let mut tag_distribution = HashMap::new();
        for pattern in &self.patterns {
            for tag in &pattern.tags {
                *tag_distribution.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        PatternMatcherStats {
            total_patterns,
            cached_inputs,
            fuzzy_threshold: self.fuzzy_threshold,
            tag_distribution,
        }
    }

    /// Get all pattern IDs
    pub fn get_pattern_ids(&self) -> Vec<String> {
        self.patterns.iter().map(|p| p.id.clone()).collect()
    }

    /// Get pattern by ID
    pub fn get_pattern(&self, id: &str) -> Option<&CompiledPattern> {
        self.patterns.iter().find(|p| p.id == id)
    }

    /// Remove pattern by ID
    pub fn remove_pattern(&mut self, id: &str) -> bool {
        let initial_len = self.patterns.len();
        self.patterns.retain(|p| p.id != id);
        let removed = self.patterns.len() != initial_len;
        if removed {
            self.clear_cache();
        }
        removed
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the pattern matcher
#[derive(Debug, Clone)]
pub struct PatternMatcherStats {
    pub total_patterns: usize,
    pub cached_inputs: usize,
    pub fuzzy_threshold: f64,
    pub tag_distribution: HashMap<String, usize>,
}

/// Pattern matcher errors
#[derive(Debug, thiserror::Error)]
pub enum PatternMatcherError {
    #[error("Invalid pattern '{0}': {1}")]
    InvalidPattern(String, String),

    #[error("Pattern not found: {0}")]
    PatternNotFound(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

/// Builder for creating pattern matchers with common patterns
pub struct PatternMatcherBuilder {
    matcher: PatternMatcher,
}

impl PatternMatcherBuilder {
    pub fn new() -> Self {
        Self {
            matcher: PatternMatcher::new(),
        }
    }

    /// Add common Rust type patterns
    pub fn with_rust_patterns(mut self) -> Result<Self, PatternMatcherError> {
        // Primitive types
        self.matcher.add_pattern_with_tags(
            "primitives",
            r"^(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize|f32|f64|bool|char)$",
            1.0,
            "Rust primitive types",
            vec!["rust".to_string(), "primitive".to_string()],
        )?;

        // String types
        self.matcher.add_pattern_with_tags(
            "strings",
            r"^(String|&str|str)$",
            1.0,
            "Rust string types",
            vec!["rust".to_string(), "string".to_string()],
        )?;

        // Collections
        self.matcher.add_pattern_with_tags(
            "collections",
            r"^(Vec|HashMap|BTreeMap|HashSet|BTreeSet|VecDeque|LinkedList)<",
            0.9,
            "Rust collection types",
            vec!["rust".to_string(), "collection".to_string()],
        )?;

        // Smart pointers
        self.matcher.add_pattern_with_tags(
            "smart_pointers",
            r"^(Box|Arc|Rc|Weak)<",
            0.9,
            "Rust smart pointer types",
            vec!["rust".to_string(), "smart_pointer".to_string()],
        )?;

        Ok(self)
    }

    /// Set fuzzy threshold
    pub fn fuzzy_threshold(mut self, threshold: f64) -> Self {
        self.matcher.set_fuzzy_threshold(threshold);
        self
    }

    /// Build the pattern matcher
    pub fn build(self) -> PatternMatcher {
        self.matcher
    }
}

impl Default for PatternMatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let mut matcher = PatternMatcher::new();
        matcher
            .add_pattern("vec", r"^Vec<", 1.0, "Vector pattern")
            .unwrap();

        let matches = matcher.find_matches("Vec<i32>");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::Prefix);
    }

    #[test]
    fn test_fuzzy_matching() {
        let mut matcher = PatternMatcher::new();
        matcher.set_fuzzy_threshold(0.6);
        matcher
            .add_pattern("vector", r"Vector", 1.0, "Vector pattern")
            .unwrap();

        let matches = matcher.find_matches("Vektor"); // Typo
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::Fuzzy);
    }

    #[test]
    fn test_pattern_with_tags() {
        let mut matcher = PatternMatcher::new();
        matcher
            .add_pattern_with_tags(
                "rust_vec",
                r"^Vec<",
                1.0,
                "Rust vector",
                vec!["rust".to_string(), "collection".to_string()],
            )
            .unwrap();

        let matches = matcher.find_matches_by_tag("Vec<i32>", "rust");
        assert_eq!(matches.len(), 1);

        let matches = matcher.find_matches_by_tag("Vec<i32>", "java");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_builder_with_rust_patterns() {
        let matcher = PatternMatcherBuilder::new()
            .with_rust_patterns()
            .unwrap()
            .fuzzy_threshold(0.8)
            .build();

        let matches = matcher.find_matches("Vec<i32>");
        assert!(!matches.is_empty());

        let matches = matcher.find_matches("i32");
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_similarity_calculation() {
        let matcher = PatternMatcher::new();

        assert_eq!(matcher.calculate_similarity("hello", "hello"), 1.0);
        assert_eq!(matcher.calculate_similarity("hello", ""), 0.0);
        assert_eq!(matcher.calculate_similarity("", "hello"), 0.0);
        assert_eq!(matcher.calculate_similarity("", ""), 1.0);

        let sim = matcher.calculate_similarity("hello", "hallo");
        assert!(sim > 0.5 && sim < 1.0);
    }

    #[test]
    fn test_cache_functionality() {
        let mut matcher = PatternMatcher::new();
        matcher
            .add_pattern("test", r"test", 1.0, "Test pattern")
            .unwrap();

        // First call
        let matches1 = matcher.find_matches("test");

        // Second call should use cache
        let matches2 = matcher.find_matches("test");

        assert_eq!(matches1.len(), matches2.len());
        assert_eq!(matches1[0].pattern_id, matches2[0].pattern_id);
    }

    #[test]
    fn test_pattern_management() {
        let mut matcher = PatternMatcher::new();

        matcher
            .add_pattern("test1", r"test1", 1.0, "Test pattern 1")
            .unwrap();
        matcher
            .add_pattern("test2", r"test2", 1.0, "Test pattern 2")
            .unwrap();

        assert_eq!(matcher.get_pattern_ids().len(), 2);

        assert!(matcher.remove_pattern("test1"));
        assert_eq!(matcher.get_pattern_ids().len(), 1);

        assert!(!matcher.remove_pattern("nonexistent"));
    }
}
