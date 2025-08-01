//! High-performance data query interface for binary export data
//!
//! This module provides a comprehensive query engine for efficiently
//! searching and filtering binary export data with minimal memory usage.

use std::collections::{HashMap, BTreeMap, HashSet};
use std::ops::Range;
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};

use super::*;

/// High-performance query engine for binary export data
pub struct QueryEngine {
    /// Data source (loaded binary data)
    data: UnifiedData,
    /// Indexed data for fast lookups
    indices: QueryIndices,
    /// Query configuration
    config: QueryConfig,
    /// Query statistics
    stats: QueryStats,
}

/// Query configuration
#[derive(Debug, Clone)]
pub struct QueryConfig {
    /// Enable automatic indexing
    pub auto_index: bool,
    /// Maximum memory usage for indices (bytes)
    pub max_index_memory: usize,
    /// Enable query result caching
    pub enable_caching: bool,
    /// Maximum cache size (number of queries)
    pub max_cache_size: usize,
    /// Query timeout (seconds)
    pub query_timeout: Duration,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            auto_index: true,
            max_index_memory: 256 * 1024 * 1024, // 256MB
            enable_caching: true,
            max_cache_size: 1000,
            query_timeout: Duration::from_secs(30),
        }
    }
}

/// Query indices for fast data access
#[derive(Debug, Default)]
struct QueryIndices {
    /// Index by allocation ID
    allocation_by_id: HashMap<u64, usize>,
    /// Index by memory address
    allocation_by_address: BTreeMap<u64, Vec<usize>>,
    /// Index by allocation size
    allocation_by_size: BTreeMap<usize, Vec<usize>>,
    /// Index by timestamp
    allocation_by_timestamp: BTreeMap<SystemTime, Vec<usize>>,
    /// Index by type name
    allocation_by_type: HashMap<String, Vec<usize>>,
    /// Index by thread ID
    allocation_by_thread: HashMap<u32, Vec<usize>>,
    /// Call stack index
    call_stack_index: HashMap<u64, CallStack>,
    /// Memory usage over time
    memory_timeline: BTreeMap<SystemTime, u64>,
}

/// Query statistics
#[derive(Debug, Clone, Default)]
pub struct QueryStats {
    /// Total queries executed
    pub total_queries: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Average query time
    pub avg_query_time: Duration,
    /// Index build time
    pub index_build_time: Duration,
    /// Memory usage by indices
    pub index_memory_usage: usize,
}

/// Query builder for constructing complex queries
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    /// Query conditions
    conditions: Vec<QueryCondition>,
    /// Sort criteria
    sort_by: Vec<SortCriteria>,
    /// Result limit
    limit: Option<usize>,
    /// Result offset
    offset: usize,
    /// Include related data
    include_related: HashSet<RelatedDataType>,
}

/// Query condition
#[derive(Debug, Clone)]
pub enum QueryCondition {
    /// Filter by allocation ID
    AllocationId(QueryOperator<u64>),
    /// Filter by memory address
    Address(QueryOperator<u64>),
    /// Filter by allocation size
    Size(QueryOperator<usize>),
    /// Filter by timestamp
    Timestamp(QueryOperator<SystemTime>),
    /// Filter by type name
    TypeName(StringOperator),
    /// Filter by thread ID
    ThreadId(QueryOperator<u32>),
    /// Filter by allocation status (active/deallocated)
    Status(AllocationStatus),
    /// Custom filter function
    Custom(Box<dyn Fn(&AllocationRecord) -> bool + Send + Sync>),
}

/// Query operators for numeric/comparable types
#[derive(Debug, Clone)]
pub enum QueryOperator<T> {
    /// Equal to value
    Equal(T),
    /// Not equal to value
    NotEqual(T),
    /// Less than value
    LessThan(T),
    /// Less than or equal to value
    LessThanOrEqual(T),
    /// Greater than value
    GreaterThan(T),
    /// Greater than or equal to value
    GreaterThanOrEqual(T),
    /// Within range (inclusive)
    Range(T, T),
    /// In set of values
    In(Vec<T>),
    /// Not in set of values
    NotIn(Vec<T>),
}

/// String operators
#[derive(Debug, Clone)]
pub enum StringOperator {
    /// Exact match
    Equal(String),
    /// Not equal
    NotEqual(String),
    /// Contains substring
    Contains(String),
    /// Starts with prefix
    StartsWith(String),
    /// Ends with suffix
    EndsWith(String),
    /// Matches regex pattern
    Regex(String),
    /// In set of values
    In(Vec<String>),
}

/// Allocation status
#[derive(Debug, Clone, PartialEq)]
pub enum AllocationStatus {
    /// Currently active (not deallocated)
    Active,
    /// Deallocated
    Deallocated,
    /// Any status
    Any,
}

/// Sort criteria
#[derive(Debug, Clone)]
pub struct SortCriteria {
    /// Field to sort by
    pub field: SortField,
    /// Sort direction
    pub direction: SortDirection,
}

/// Fields available for sorting
#[derive(Debug, Clone)]
pub enum SortField {
    /// Sort by allocation ID
    AllocationId,
    /// Sort by memory address
    Address,
    /// Sort by allocation size
    Size,
    /// Sort by timestamp
    Timestamp,
    /// Sort by type name
    TypeName,
    /// Sort by thread ID
    ThreadId,
}

/// Sort direction
#[derive(Debug, Clone)]
pub enum SortDirection {
    /// Ascending order
    Ascending,
    /// Descending order
    Descending,
}

/// Related data types to include in results
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RelatedDataType {
    /// Include call stack information
    CallStacks,
    /// Include performance metrics
    PerformanceMetrics,
    /// Include security analysis
    SecurityAnalysis,
    /// Include lifecycle information
    LifecycleAnalysis,
}

/// Query result
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Matching allocation records
    pub allocations: Vec<AllocationRecord>,
    /// Related call stacks (if requested)
    pub call_stacks: HashMap<u64, CallStack>,
    /// Query execution statistics
    pub execution_stats: QueryExecutionStats,
    /// Total number of matches (before limit/offset)
    pub total_matches: usize,
    /// Whether results were truncated
    pub truncated: bool,
}

/// Query execution statistics
#[derive(Debug, Clone)]
pub struct QueryExecutionStats {
    /// Query execution time
    pub execution_time: Duration,
    /// Number of records scanned
    pub records_scanned: usize,
    /// Number of index lookups performed
    pub index_lookups: usize,
    /// Whether query used indices
    pub used_indices: bool,
    /// Memory usage during query
    pub memory_usage: usize,
}

/// Aggregation query for statistical analysis
#[derive(Debug, Clone)]
pub struct AggregationQuery {
    /// Group by field
    pub group_by: Option<GroupByField>,
    /// Aggregation functions to apply
    pub aggregations: Vec<AggregationFunction>,
    /// Filter conditions
    pub conditions: Vec<QueryCondition>,
}

/// Fields available for grouping
#[derive(Debug, Clone)]
pub enum GroupByField {
    /// Group by type name
    TypeName,
    /// Group by thread ID
    ThreadId,
    /// Group by size range
    SizeRange(usize), // Range size
    /// Group by time period
    TimePeriod(Duration), // Period duration
}

/// Aggregation functions
#[derive(Debug, Clone)]
pub enum AggregationFunction {
    /// Count of records
    Count,
    /// Sum of sizes
    SumSize,
    /// Average size
    AvgSize,
    /// Minimum size
    MinSize,
    /// Maximum size
    MaxSize,
    /// Memory usage over time
    MemoryTimeline,
}

/// Aggregation result
#[derive(Debug, Clone)]
pub struct AggregationResult {
    /// Grouped results
    pub groups: HashMap<String, AggregationValues>,
    /// Overall statistics
    pub overall: AggregationValues,
    /// Execution statistics
    pub execution_stats: QueryExecutionStats,
}

/// Aggregation values
#[derive(Debug, Clone, Default)]
pub struct AggregationValues {
    /// Count
    pub count: u64,
    /// Sum of sizes
    pub sum_size: u64,
    /// Average size
    pub avg_size: f64,
    /// Minimum size
    pub min_size: Option<usize>,
    /// Maximum size
    pub max_size: Option<usize>,
    /// Memory timeline (timestamp -> memory usage)
    pub memory_timeline: BTreeMap<SystemTime, u64>,
}

impl QueryEngine {
    /// Create a new query engine with data
    pub fn new(data: UnifiedData, config: QueryConfig) -> Self {
        let mut engine = Self {
            data,
            indices: QueryIndices::default(),
            config,
            stats: QueryStats::default(),
        };

        if engine.config.auto_index {
            let start_time = std::time::Instant::now();
            engine.build_indices();
            engine.stats.index_build_time = start_time.elapsed();
        }

        engine
    }

    /// Load data from binary file and create query engine
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
        config: QueryConfig,
    ) -> Result<Self, BinaryExportError> {
        let parser = BinaryDataParser::new(ParserConfig::default());
        let parse_result = parser.parse_file(path)?;
        Ok(Self::new(parse_result.data, config))
    }

    /// Build indices for fast querying
    fn build_indices(&mut self) {
        let allocations = &self.data.allocations.allocations;
        
        // Build allocation indices
        for (index, allocation) in allocations.iter().enumerate() {
            // Index by ID
            self.indices.allocation_by_id.insert(allocation.id, index);
            
            // Index by address
            self.indices.allocation_by_address
                .entry(allocation.address)
                .or_insert_with(Vec::new)
                .push(index);
            
            // Index by size
            self.indices.allocation_by_size
                .entry(allocation.size)
                .or_insert_with(Vec::new)
                .push(index);
            
            // Index by timestamp
            self.indices.allocation_by_timestamp
                .entry(allocation.timestamp)
                .or_insert_with(Vec::new)
                .push(index);
            
            // Index by type name
            self.indices.allocation_by_type
                .entry(allocation.allocation_type.clone())
                .or_insert_with(Vec::new)
                .push(index);
            
            // Index by thread ID
            self.indices.allocation_by_thread
                .entry(allocation.thread_id)
                .or_insert_with(Vec::new)
                .push(index);
        }

        // Build call stack index
        for (id, call_stack) in &self.data.allocations.call_stacks {
            self.indices.call_stack_index.insert(*id, call_stack.clone());
        }

        // Build memory timeline
        self.build_memory_timeline();

        // Calculate index memory usage
        self.stats.index_memory_usage = self.calculate_index_memory_usage();
    }

    /// Build memory usage timeline
    fn build_memory_timeline(&mut self) {
        let mut timeline: BTreeMap<SystemTime, i64> = BTreeMap::new();
        
        for allocation in &self.data.allocations.allocations {
            // Add allocation
            *timeline.entry(allocation.timestamp).or_insert(0) += allocation.size as i64;
            
            // Add deallocation if available
            // Note: In the current data model, we don't have deallocation timestamps
            // This would need to be added to the AllocationRecord structure
        }

        // Convert to cumulative memory usage
        let mut cumulative = 0u64;
        for (timestamp, change) in timeline {
            cumulative = (cumulative as i64 + change).max(0) as u64;
            self.indices.memory_timeline.insert(timestamp, cumulative);
        }
    }

    /// Calculate memory usage of indices
    fn calculate_index_memory_usage(&self) -> usize {
        // Simplified calculation - would be more accurate in production
        let mut usage = 0;
        
        usage += self.indices.allocation_by_id.len() * (8 + 8); // u64 + usize
        usage += self.indices.allocation_by_address.len() * (8 + 24); // u64 + Vec overhead
        usage += self.indices.allocation_by_size.len() * (8 + 24);
        usage += self.indices.allocation_by_timestamp.len() * (16 + 24); // SystemTime + Vec
        
        // String indices are more complex to calculate
        for (key, values) in &self.indices.allocation_by_type {
            usage += key.len() + values.len() * 8;
        }
        
        usage += self.indices.allocation_by_thread.len() * (4 + 24); // u32 + Vec
        usage += self.indices.call_stack_index.len() * 100; // Rough estimate per call stack
        usage += self.indices.memory_timeline.len() * (16 + 8); // SystemTime + u64
        
        usage
    }

    /// Create a new query builder
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new()
    }

    /// Execute a query
    pub fn execute_query(&mut self, query: QueryBuilder) -> Result<QueryResult, BinaryExportError> {
        let start_time = std::time::Instant::now();
        self.stats.total_queries += 1;

        // Apply filters
        let mut matching_indices = self.apply_filters(&query.conditions)?;

        // Apply sorting
        if !query.sort_by.is_empty() {
            self.apply_sorting(&mut matching_indices, &query.sort_by);
        }

        // Apply pagination
        let total_matches = matching_indices.len();
        let truncated = query.limit.map_or(false, |limit| total_matches > query.offset + limit);
        
        if query.offset > 0 {
            if query.offset >= matching_indices.len() {
                matching_indices.clear();
            } else {
                matching_indices.drain(0..query.offset);
            }
        }
        
        if let Some(limit) = query.limit {
            matching_indices.truncate(limit);
        }

        // Collect results
        let mut allocations = Vec::new();
        let mut call_stacks = HashMap::new();
        
        for &index in &matching_indices {
            if let Some(allocation) = self.data.allocations.allocations.get(index) {
                allocations.push(allocation.clone());
                
                // Include call stack if requested
                if query.include_related.contains(&RelatedDataType::CallStacks) {
                    if let Some(call_stack_id) = allocation.call_stack_id {
                        if let Some(call_stack) = self.indices.call_stack_index.get(&call_stack_id) {
                            call_stacks.insert(call_stack_id, call_stack.clone());
                        }
                    }
                }
            }
        }

        let execution_time = start_time.elapsed();
        self.stats.avg_query_time = Duration::from_nanos(
            ((self.stats.avg_query_time.as_nanos() * (self.stats.total_queries - 1) as u128) + 
             execution_time.as_nanos()) / self.stats.total_queries as u128
        );

        Ok(QueryResult {
            allocations,
            call_stacks,
            execution_stats: QueryExecutionStats {
                execution_time,
                records_scanned: matching_indices.len(),
                index_lookups: 0, // Would track this in a real implementation
                used_indices: true,
                memory_usage: 0, // Would track this in a real implementation
            },
            total_matches,
            truncated,
        })
    }

    /// Apply query filters
    fn apply_filters(&self, conditions: &[QueryCondition]) -> Result<Vec<usize>, BinaryExportError> {
        if conditions.is_empty() {
            // Return all indices
            return Ok((0..self.data.allocations.allocations.len()).collect());
        }

        let mut result_indices: Option<HashSet<usize>> = None;

        for condition in conditions {
            let condition_indices = self.apply_single_filter(condition)?;
            
            match &mut result_indices {
                None => {
                    result_indices = Some(condition_indices.into_iter().collect());
                }
                Some(existing) => {
                    // Intersection with existing results
                    existing.retain(|&index| condition_indices.contains(&index));
                }
            }
        }

        Ok(result_indices.unwrap_or_default().into_iter().collect())
    }

    /// Apply a single filter condition
    fn apply_single_filter(&self, condition: &QueryCondition) -> Result<Vec<usize>, BinaryExportError> {
        match condition {
            QueryCondition::AllocationId(op) => {
                self.apply_numeric_filter(&self.indices.allocation_by_id, op, |alloc| alloc.id)
            }
            QueryCondition::Address(op) => {
                self.apply_btree_numeric_filter(&self.indices.allocation_by_address, op)
            }
            QueryCondition::Size(op) => {
                self.apply_btree_numeric_filter(&self.indices.allocation_by_size, op)
            }
            QueryCondition::Timestamp(op) => {
                self.apply_btree_time_filter(&self.indices.allocation_by_timestamp, op)
            }
            QueryCondition::TypeName(op) => {
                self.apply_string_filter(&self.indices.allocation_by_type, op)
            }
            QueryCondition::ThreadId(op) => {
                self.apply_numeric_filter(&self.indices.allocation_by_thread, op, |alloc| alloc.thread_id)
            }
            QueryCondition::Status(status) => {
                self.apply_status_filter(status)
            }
            QueryCondition::Custom(filter_fn) => {
                let mut indices = Vec::new();
                for (index, allocation) in self.data.allocations.allocations.iter().enumerate() {
                    if filter_fn(allocation) {
                        indices.push(index);
                    }
                }
                Ok(indices)
            }
        }
    }

    /// Apply numeric filter using HashMap index
    fn apply_numeric_filter<T, F>(
        &self,
        index: &HashMap<T, Vec<usize>>,
        op: &QueryOperator<T>,
        _field_extractor: F,
    ) -> Result<Vec<usize>, BinaryExportError>
    where
        T: Clone + PartialEq + PartialOrd,
        F: Fn(&AllocationRecord) -> T,
    {
        match op {
            QueryOperator::Equal(value) => {
                Ok(index.get(value).cloned().unwrap_or_default())
            }
            QueryOperator::In(values) => {
                let mut result = Vec::new();
                for value in values {
                    if let Some(indices) = index.get(value) {
                        result.extend(indices);
                    }
                }
                Ok(result)
            }
            _ => {
                // For complex operations, fall back to linear scan
                self.apply_linear_numeric_filter(op, _field_extractor)
            }
        }
    }

    /// Apply numeric filter using BTreeMap index
    fn apply_btree_numeric_filter<T>(
        &self,
        index: &BTreeMap<T, Vec<usize>>,
        op: &QueryOperator<T>,
    ) -> Result<Vec<usize>, BinaryExportError>
    where
        T: Clone + PartialEq + PartialOrd,
    {
        let mut result = Vec::new();
        
        match op {
            QueryOperator::Equal(value) => {
                if let Some(indices) = index.get(value) {
                    result.extend(indices);
                }
            }
            QueryOperator::LessThan(value) => {
                for (_, indices) in index.range(..value) {
                    result.extend(indices);
                }
            }
            QueryOperator::LessThanOrEqual(value) => {
                for (_, indices) in index.range(..=value) {
                    result.extend(indices);
                }
            }
            QueryOperator::GreaterThan(value) => {
                for (key, indices) in index.range((std::ops::Bound::Excluded(value), std::ops::Bound::Unbounded)) {
                    result.extend(indices);
                }
            }
            QueryOperator::GreaterThanOrEqual(value) => {
                for (_, indices) in index.range(value..) {
                    result.extend(indices);
                }
            }
            QueryOperator::Range(start, end) => {
                for (_, indices) in index.range(start..=end) {
                    result.extend(indices);
                }
            }
            QueryOperator::In(values) => {
                for value in values {
                    if let Some(indices) = index.get(value) {
                        result.extend(indices);
                    }
                }
            }
            _ => {
                // Handle other operators with linear scan
                return Err(BinaryExportError::UnsupportedFeature(
                    "Complex numeric operators not yet implemented for BTreeMap".to_string()
                ));
            }
        }
        
        Ok(result)
    }

    /// Apply time-based filter using BTreeMap index
    fn apply_btree_time_filter(
        &self,
        index: &BTreeMap<SystemTime, Vec<usize>>,
        op: &QueryOperator<SystemTime>,
    ) -> Result<Vec<usize>, BinaryExportError> {
        // Similar to numeric filter but for SystemTime
        self.apply_btree_numeric_filter(index, op)
    }

    /// Apply string filter
    fn apply_string_filter(
        &self,
        index: &HashMap<String, Vec<usize>>,
        op: &StringOperator,
    ) -> Result<Vec<usize>, BinaryExportError> {
        let mut result = Vec::new();
        
        match op {
            StringOperator::Equal(value) => {
                if let Some(indices) = index.get(value) {
                    result.extend(indices);
                }
            }
            StringOperator::In(values) => {
                for value in values {
                    if let Some(indices) = index.get(value) {
                        result.extend(indices);
                    }
                }
            }
            StringOperator::Contains(substring) => {
                for (key, indices) in index {
                    if key.contains(substring) {
                        result.extend(indices);
                    }
                }
            }
            StringOperator::StartsWith(prefix) => {
                for (key, indices) in index {
                    if key.starts_with(prefix) {
                        result.extend(indices);
                    }
                }
            }
            StringOperator::EndsWith(suffix) => {
                for (key, indices) in index {
                    if key.ends_with(suffix) {
                        result.extend(indices);
                    }
                }
            }
            _ => {
                return Err(BinaryExportError::UnsupportedFeature(
                    "Complex string operators not yet implemented".to_string()
                ));
            }
        }
        
        Ok(result)
    }

    /// Apply status filter
    fn apply_status_filter(&self, status: &AllocationStatus) -> Result<Vec<usize>, BinaryExportError> {
        match status {
            AllocationStatus::Any => {
                Ok((0..self.data.allocations.allocations.len()).collect())
            }
            AllocationStatus::Active => {
                // In current data model, we don't track deallocation
                // So all allocations are considered active
                Ok((0..self.data.allocations.allocations.len()).collect())
            }
            AllocationStatus::Deallocated => {
                // Would need deallocation tracking in data model
                Ok(Vec::new())
            }
        }
    }

    /// Apply linear numeric filter (fallback for complex operations)
    fn apply_linear_numeric_filter<T, F>(
        &self,
        op: &QueryOperator<T>,
        field_extractor: F,
    ) -> Result<Vec<usize>, BinaryExportError>
    where
        T: PartialEq + PartialOrd,
        F: Fn(&AllocationRecord) -> T,
    {
        let mut result = Vec::new();
        
        for (index, allocation) in self.data.allocations.allocations.iter().enumerate() {
            let field_value = field_extractor(allocation);
            
            let matches = match op {
                QueryOperator::Equal(value) => field_value == *value,
                QueryOperator::NotEqual(value) => field_value != *value,
                QueryOperator::LessThan(value) => field_value < *value,
                QueryOperator::LessThanOrEqual(value) => field_value <= *value,
                QueryOperator::GreaterThan(value) => field_value > *value,
                QueryOperator::GreaterThanOrEqual(value) => field_value >= *value,
                QueryOperator::Range(start, end) => field_value >= *start && field_value <= *end,
                QueryOperator::In(values) => values.contains(&field_value),
                QueryOperator::NotIn(values) => !values.contains(&field_value),
            };
            
            if matches {
                result.push(index);
            }
        }
        
        Ok(result)
    }

    /// Apply sorting to results
    fn apply_sorting(&self, indices: &mut Vec<usize>, sort_criteria: &[SortCriteria]) {
        indices.sort_by(|&a, &b| {
            for criteria in sort_criteria {
                let alloc_a = &self.data.allocations.allocations[a];
                let alloc_b = &self.data.allocations.allocations[b];
                
                let ordering = match criteria.field {
                    SortField::AllocationId => alloc_a.id.cmp(&alloc_b.id),
                    SortField::Address => alloc_a.address.cmp(&alloc_b.address),
                    SortField::Size => alloc_a.size.cmp(&alloc_b.size),
                    SortField::Timestamp => alloc_a.timestamp.cmp(&alloc_b.timestamp),
                    SortField::TypeName => alloc_a.allocation_type.cmp(&alloc_b.allocation_type),
                    SortField::ThreadId => alloc_a.thread_id.cmp(&alloc_b.thread_id),
                };
                
                let final_ordering = match criteria.direction {
                    SortDirection::Ascending => ordering,
                    SortDirection::Descending => ordering.reverse(),
                };
                
                if final_ordering != std::cmp::Ordering::Equal {
                    return final_ordering;
                }
            }
            
            std::cmp::Ordering::Equal
        });
    }

    /// Execute aggregation query
    pub fn execute_aggregation(&mut self, query: AggregationQuery) -> Result<AggregationResult, BinaryExportError> {
        let start_time = std::time::Instant::now();
        
        // Apply filters
        let matching_indices = self.apply_filters(&query.conditions)?;
        
        // Group data
        let groups = if let Some(group_by) = &query.group_by {
            self.group_data(&matching_indices, group_by)?
        } else {
            let mut groups = HashMap::new();
            groups.insert("all".to_string(), matching_indices);
            groups
        };
        
        // Calculate aggregations for each group
        let mut result_groups = HashMap::new();
        let mut overall = AggregationValues::default();
        
        for (group_name, indices) in groups {
            let group_values = self.calculate_aggregations(&indices, &query.aggregations)?;
            
            // Update overall statistics
            overall.count += group_values.count;
            overall.sum_size += group_values.sum_size;
            if let Some(min) = group_values.min_size {
                overall.min_size = Some(overall.min_size.map_or(min, |existing| existing.min(min)));
            }
            if let Some(max) = group_values.max_size {
                overall.max_size = Some(overall.max_size.map_or(max, |existing| existing.max(max)));
            }
            
            result_groups.insert(group_name, group_values);
        }
        
        // Calculate overall average
        if overall.count > 0 {
            overall.avg_size = overall.sum_size as f64 / overall.count as f64;
        }
        
        let execution_time = start_time.elapsed();
        
        Ok(AggregationResult {
            groups: result_groups,
            overall,
            execution_stats: QueryExecutionStats {
                execution_time,
                records_scanned: matching_indices.len(),
                index_lookups: 0,
                used_indices: true,
                memory_usage: 0,
            },
        })
    }

    /// Group data by specified field
    fn group_data(
        &self,
        indices: &[usize],
        group_by: &GroupByField,
    ) -> Result<HashMap<String, Vec<usize>>, BinaryExportError> {
        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
        
        for &index in indices {
            let allocation = &self.data.allocations.allocations[index];
            
            let group_key = match group_by {
                GroupByField::TypeName => allocation.allocation_type.clone(),
                GroupByField::ThreadId => allocation.thread_id.to_string(),
                GroupByField::SizeRange(range_size) => {
                    let range_start = (allocation.size / range_size) * range_size;
                    format!("{}-{}", range_start, range_start + range_size)
                }
                GroupByField::TimePeriod(period) => {
                    // Group by time period (simplified)
                    let timestamp_secs = allocation.timestamp
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let period_secs = period.as_secs();
                    let period_start = (timestamp_secs / period_secs) * period_secs;
                    format!("period_{}", period_start)
                }
            };
            
            groups.entry(group_key).or_insert_with(Vec::new).push(index);
        }
        
        Ok(groups)
    }

    /// Calculate aggregations for a group of indices
    fn calculate_aggregations(
        &self,
        indices: &[usize],
        aggregations: &[AggregationFunction],
    ) -> Result<AggregationValues, BinaryExportError> {
        let mut values = AggregationValues::default();
        
        if indices.is_empty() {
            return Ok(values);
        }
        
        values.count = indices.len() as u64;
        
        for &index in indices {
            let allocation = &self.data.allocations.allocations[index];
            
            for aggregation in aggregations {
                match aggregation {
                    AggregationFunction::Count => {
                        // Already calculated above
                    }
                    AggregationFunction::SumSize => {
                        values.sum_size += allocation.size as u64;
                    }
                    AggregationFunction::MinSize => {
                        values.min_size = Some(
                            values.min_size.map_or(allocation.size, |existing| existing.min(allocation.size))
                        );
                    }
                    AggregationFunction::MaxSize => {
                        values.max_size = Some(
                            values.max_size.map_or(allocation.size, |existing| existing.max(allocation.size))
                        );
                    }
                    AggregationFunction::MemoryTimeline => {
                        values.memory_timeline.insert(allocation.timestamp, allocation.size as u64);
                    }
                    _ => {}
                }
            }
        }
        
        // Calculate average
        if values.count > 0 {
            values.avg_size = values.sum_size as f64 / values.count as f64;
        }
        
        Ok(values)
    }

    /// Get query statistics
    pub fn get_stats(&self) -> &QueryStats {
        &self.stats
    }

    /// Get memory timeline
    pub fn get_memory_timeline(&self) -> &BTreeMap<SystemTime, u64> {
        &self.indices.memory_timeline
    }

    /// Rebuild indices (useful after data updates)
    pub fn rebuild_indices(&mut self) {
        let start_time = std::time::Instant::now();
        self.indices = QueryIndices::default();
        self.build_indices();
        self.stats.index_build_time = start_time.elapsed();
    }
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            sort_by: Vec::new(),
            limit: None,
            offset: 0,
            include_related: HashSet::new(),
        }
    }

    /// Add a condition to the query
    pub fn where_condition(mut self, condition: QueryCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Filter by allocation ID
    pub fn where_id(self, op: QueryOperator<u64>) -> Self {
        self.where_condition(QueryCondition::AllocationId(op))
    }

    /// Filter by memory address
    pub fn where_address(self, op: QueryOperator<u64>) -> Self {
        self.where_condition(QueryCondition::Address(op))
    }

    /// Filter by allocation size
    pub fn where_size(self, op: QueryOperator<usize>) -> Self {
        self.where_condition(QueryCondition::Size(op))
    }

    /// Filter by timestamp
    pub fn where_timestamp(self, op: QueryOperator<SystemTime>) -> Self {
        self.where_condition(QueryCondition::Timestamp(op))
    }

    /// Filter by type name
    pub fn where_type(self, op: StringOperator) -> Self {
        self.where_condition(QueryCondition::TypeName(op))
    }

    /// Filter by thread ID
    pub fn where_thread(self, op: QueryOperator<u32>) -> Self {
        self.where_condition(QueryCondition::ThreadId(op))
    }

    /// Filter by allocation status
    pub fn where_status(self, status: AllocationStatus) -> Self {
        self.where_condition(QueryCondition::Status(status))
    }

    /// Add custom filter
    pub fn where_custom<F>(self, filter: F) -> Self
    where
        F: Fn(&AllocationRecord) -> bool + Send + Sync + 'static,
    {
        self.where_condition(QueryCondition::Custom(Box::new(filter)))
    }

    /// Add sort criteria
    pub fn order_by(mut self, field: SortField, direction: SortDirection) -> Self {
        self.sort_by.push(SortCriteria { field, direction });
        self
    }

    /// Set result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set result offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Include related data in results
    pub fn include(mut self, related: RelatedDataType) -> Self {
        self.include_related.insert(related);
        self
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> UnifiedData {
        let mut data = UnifiedData::new();
        
        // Add test allocations
        for i in 0..100 {
            data.allocations.allocations.push(AllocationRecord {
                id: i,
                address: 0x1000 + i * 0x100,
                size: (i as usize % 10 + 1) * 64, // Sizes from 64 to 640 bytes
                timestamp: SystemTime::UNIX_EPOCH + Duration::from_secs(i),
                call_stack_id: Some(i % 5), // 5 different call stacks
                thread_id: (i % 3) as u32 + 1, // 3 threads
                allocation_type: format!("Type{}", i % 5), // 5 different types
            });
        }
        
        // Add test call stacks
        for i in 0..5 {
            let call_stack = CallStack {
                id: i,
                frames: vec![StackFrame {
                    function_name: format!("function_{}", i),
                    file_name: Some(format!("file_{}.rs", i)),
                    line_number: Some(100 + i as u32),
                    column_number: Some(10),
                }],
            };
            data.allocations.call_stacks.insert(i, call_stack);
        }
        
        data
    }

    #[test]
    fn test_query_engine_creation() {
        let data = create_test_data();
        let config = QueryConfig::default();
        let engine = QueryEngine::new(data, config);
        
        assert_eq!(engine.data.allocations.allocations.len(), 100);
        assert_eq!(engine.indices.allocation_by_id.len(), 100);
        assert!(engine.stats.index_build_time.as_millis() >= 0);
    }

    #[test]
    fn test_simple_query() {
        let data = create_test_data();
        let config = QueryConfig::default();
        let mut engine = QueryEngine::new(data, config);
        
        // Query for allocations with size > 300
        let query = engine.query()
            .where_size(QueryOperator::GreaterThan(300))
            .limit(10);
        
        let result = engine.execute_query(query).unwrap();
        
        assert!(result.allocations.len() <= 10);
        for allocation in &result.allocations {
            assert!(allocation.size > 300);
        }
    }

    #[test]
    fn test_complex_query() {
        let data = create_test_data();
        let config = QueryConfig::default();
        let mut engine = QueryEngine::new(data, config);
        
        // Query for Type0 allocations in thread 1, sorted by size
        let query = engine.query()
            .where_type(StringOperator::Equal("Type0".to_string()))
            .where_thread(QueryOperator::Equal(1))
            .order_by(SortField::Size, SortDirection::Descending)
            .include(RelatedDataType::CallStacks);
        
        let result = engine.execute_query(query).unwrap();
        
        // Verify all results match criteria
        for allocation in &result.allocations {
            assert_eq!(allocation.allocation_type, "Type0");
            assert_eq!(allocation.thread_id, 1);
        }
        
        // Verify sorting
        for i in 1..result.allocations.len() {
            assert!(result.allocations[i-1].size >= result.allocations[i].size);
        }
        
        // Verify call stacks are included
        assert!(!result.call_stacks.is_empty());
    }

    #[test]
    fn test_aggregation_query() {
        let data = create_test_data();
        let config = QueryConfig::default();
        let mut engine = QueryEngine::new(data, config);
        
        let agg_query = AggregationQuery {
            group_by: Some(GroupByField::TypeName),
            aggregations: vec![
                AggregationFunction::Count,
                AggregationFunction::SumSize,
                AggregationFunction::AvgSize,
            ],
            conditions: vec![],
        };
        
        let result = engine.execute_aggregation(agg_query).unwrap();
        
        // Should have 5 groups (Type0 through Type4)
        assert_eq!(result.groups.len(), 5);
        
        // Overall count should be 100
        assert_eq!(result.overall.count, 100);
        
        // Each group should have 20 items (100 / 5)
        for (group_name, values) in &result.groups {
            assert_eq!(values.count, 20);
            assert!(values.sum_size > 0);
            assert!(values.avg_size > 0.0);
        }
    }

    #[test]
    fn test_memory_timeline() {
        let data = create_test_data();
        let config = QueryConfig::default();
        let engine = QueryEngine::new(data, config);
        
        let timeline = engine.get_memory_timeline();
        
        // Should have entries for each allocation
        assert!(!timeline.is_empty());
        
        // Memory usage should generally increase over time
        let mut prev_memory = 0;
        for (_, memory) in timeline {
            assert!(*memory >= prev_memory);
            prev_memory = *memory;
        }
    }

    #[test]
    fn test_query_stats() {
        let data = create_test_data();
        let config = QueryConfig::default();
        let mut engine = QueryEngine::new(data, config);
        
        // Execute a few queries
        for i in 0..5 {
            let query = engine.query()
                .where_size(QueryOperator::GreaterThan(i * 100))
                .limit(10);
            let _ = engine.execute_query(query);
        }
        
        let stats = engine.get_stats();
        assert_eq!(stats.total_queries, 5);
        assert!(stats.avg_query_time.as_millis() >= 0);
        assert!(stats.index_memory_usage > 0);
    }
}