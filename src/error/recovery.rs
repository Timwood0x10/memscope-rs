use super::{ErrorKind, ErrorSeverity, MemScopeError};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Comprehensive recovery strategy system
pub struct RecoveryStrategy {
    /// Recovery actions by error kind
    action_map: HashMap<ErrorKind, RecoveryAction>,
    /// Retry configuration
    retry_config: RetryConfig,
    /// Fallback mechanisms
    fallback_registry: FallbackRegistry,
    /// Circuit breaker for preventing cascade failures
    circuit_breaker: CircuitBreaker,
}

/// Specific recovery actions for different error types
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Retry operation with exponential backoff
    RetryWithBackoff {
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f64,
    },
    /// Switch to alternative implementation
    Fallback {
        strategy: FallbackStrategy,
        timeout: Duration,
    },
    /// Gracefully degrade functionality
    Degrade {
        level: DegradationLevel,
        duration: Duration,
    },
    /// Reset component state
    Reset {
        component: String,
        preserve_data: bool,
    },
    /// Skip operation and continue
    Skip,
    /// Terminate operation safely
    Terminate,
}

/// Types of fallback strategies
#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    /// Use cached data instead of live computation
    UseCache,
    /// Use simplified algorithm
    SimplifiedAlgorithm,
    /// Use mock/default data
    MockData,
    /// Delegate to backup system
    BackupSystem,
}

/// Levels of functionality degradation
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DegradationLevel {
    /// Minimal impact, reduce precision
    Minimal,
    /// Moderate impact, disable non-essential features
    Moderate,
    /// Significant impact, basic functionality only
    Significant,
    /// Severe impact, emergency mode
    Severe,
}

/// Retry configuration parameters
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Default maximum retry attempts
    pub default_max_attempts: u32,
    /// Default initial retry delay
    pub default_initial_delay: Duration,
    /// Default maximum retry delay
    pub default_max_delay: Duration,
    /// Default backoff multiplier
    pub default_backoff_multiplier: f64,
    /// Whether to add jitter to retry delays
    pub enable_jitter: bool,
}

/// Registry of fallback mechanisms
pub struct FallbackRegistry {
    /// Available fallback strategies by name
    strategies: HashMap<String, Box<dyn Fn() -> Result<(), MemScopeError> + Send + Sync>>,
}

/// Circuit breaker pattern implementation
pub struct CircuitBreaker {
    /// Current state of the circuit breaker
    state: CircuitState,
    /// Failure count in current window
    failure_count: u32,
    /// Threshold for opening circuit
    failure_threshold: u32,
    /// Time when circuit was opened
    opened_at: Option<Instant>,
    /// How long to wait before trying again
    timeout: Duration,
    /// Time window for counting failures
    window_duration: Duration,
    /// When current window started
    window_start: Instant,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Failing, stop trying
    Open,
    /// Testing if service is back
    HalfOpen,
}

impl RecoveryStrategy {
    /// Create new recovery strategy with default configuration
    pub fn new() -> Self {
        let mut strategy = Self {
            action_map: HashMap::new(),
            retry_config: RetryConfig::default(),
            fallback_registry: FallbackRegistry::new(),
            circuit_breaker: CircuitBreaker::new(),
        };

        strategy.setup_default_actions();
        strategy
    }

    /// Execute recovery action for given error
    pub fn recover(&mut self, error: &MemScopeError) -> RecoveryResult {
        // Check circuit breaker first
        if !self.circuit_breaker.can_execute() {
            return RecoveryResult::CircuitOpen;
        }

        // Get appropriate recovery action
        let action = self.get_recovery_action(error);

        // Execute recovery action
        let result = self.execute_action(action, error);

        // Update circuit breaker based on result
        match &result {
            RecoveryResult::Success => self.circuit_breaker.record_success(),
            RecoveryResult::Failed(_) => self.circuit_breaker.record_failure(),
            _ => {} // Other results don't affect circuit breaker
        }

        result
    }

    /// Register custom recovery action for error kind
    pub fn register_action(&mut self, kind: ErrorKind, action: RecoveryAction) {
        self.action_map.insert(kind, action);
    }

    /// Register fallback strategy
    pub fn register_fallback<F>(&mut self, name: String, strategy: F)
    where
        F: Fn() -> Result<(), MemScopeError> + Send + Sync + 'static,
    {
        self.fallback_registry.register(name, Box::new(strategy));
    }

    /// Get circuit breaker status
    pub fn get_circuit_status(&self) -> CircuitState {
        self.circuit_breaker.state.clone()
    }

    /// Force circuit breaker to reset
    pub fn reset_circuit(&mut self) {
        self.circuit_breaker.reset();
    }

    fn setup_default_actions(&mut self) {
        // Memory errors: retry with backoff
        self.action_map.insert(
            ErrorKind::MemoryError,
            RecoveryAction::RetryWithBackoff {
                max_attempts: 3,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(5),
                backoff_multiplier: 2.0,
            },
        );

        // Configuration errors: reset and retry
        self.action_map.insert(
            ErrorKind::ConfigurationError,
            RecoveryAction::Reset {
                component: "configuration".to_string(),
                preserve_data: false,
            },
        );

        // I/O errors: fallback to cache
        self.action_map.insert(
            ErrorKind::IoError,
            RecoveryAction::Fallback {
                strategy: FallbackStrategy::UseCache,
                timeout: Duration::from_secs(30),
            },
        );

        // Symbol resolution errors: degrade gracefully
        self.action_map.insert(
            ErrorKind::SymbolResolutionError,
            RecoveryAction::Degrade {
                level: DegradationLevel::Minimal,
                duration: Duration::from_secs(60),
            },
        );

        // Stack trace errors: skip and continue
        self.action_map
            .insert(ErrorKind::StackTraceError, RecoveryAction::Skip);

        // Cache errors: reset cache
        self.action_map.insert(
            ErrorKind::CacheError,
            RecoveryAction::Reset {
                component: "cache".to_string(),
                preserve_data: false,
            },
        );

        // Fatal errors: terminate safely
        self.action_map
            .insert(ErrorKind::InternalError, RecoveryAction::Terminate);
    }

    fn get_recovery_action(&self, error: &MemScopeError) -> RecoveryAction {
        // Check for registered action
        if let Some(action) = self.action_map.get(&error.kind) {
            return action.clone();
        }

        // Fallback based on severity
        match error.severity {
            ErrorSeverity::Warning => RecoveryAction::Skip,
            ErrorSeverity::Error => RecoveryAction::RetryWithBackoff {
                max_attempts: self.retry_config.default_max_attempts,
                initial_delay: self.retry_config.default_initial_delay,
                max_delay: self.retry_config.default_max_delay,
                backoff_multiplier: self.retry_config.default_backoff_multiplier,
            },
            ErrorSeverity::Critical => RecoveryAction::Fallback {
                strategy: FallbackStrategy::MockData,
                timeout: Duration::from_secs(10),
            },
            ErrorSeverity::Fatal => RecoveryAction::Terminate,
        }
    }

    fn execute_action(&mut self, action: RecoveryAction, error: &MemScopeError) -> RecoveryResult {
        match action {
            RecoveryAction::RetryWithBackoff { .. } => RecoveryResult::Retry {
                action,
                delay: self.calculate_retry_delay(error),
            },
            RecoveryAction::Fallback { strategy, .. } => {
                if let Ok(()) = self.execute_fallback(&strategy) {
                    RecoveryResult::Success
                } else {
                    RecoveryResult::Failed("Fallback strategy failed".to_string())
                }
            }
            RecoveryAction::Degrade { level, duration } => {
                RecoveryResult::Degraded { level, duration }
            }
            RecoveryAction::Reset {
                component,
                preserve_data,
            } => RecoveryResult::Reset {
                component,
                preserve_data,
            },
            RecoveryAction::Skip => RecoveryResult::Skipped,
            RecoveryAction::Terminate => RecoveryResult::Terminated,
        }
    }

    fn calculate_retry_delay(&self, _error: &MemScopeError) -> Duration {
        // Simple implementation - could be made more sophisticated
        self.retry_config.default_initial_delay
    }

    fn execute_fallback(&self, strategy: &FallbackStrategy) -> Result<(), Box<MemScopeError>> {
        match strategy {
            FallbackStrategy::UseCache => {
                // Implementation would check cache availability
                Ok(())
            }
            FallbackStrategy::SimplifiedAlgorithm => {
                // Implementation would switch to simpler algorithm
                Ok(())
            }
            FallbackStrategy::MockData => {
                // Implementation would return mock data
                Ok(())
            }
            FallbackStrategy::BackupSystem => {
                // Implementation would delegate to backup
                Ok(())
            }
        }
    }
}

/// Result of recovery action execution
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    /// Recovery successful, continue normal operation
    Success,
    /// Should retry with specified action and delay
    Retry {
        action: RecoveryAction,
        delay: Duration,
    },
    /// Operation degraded to specified level
    Degraded {
        level: DegradationLevel,
        duration: Duration,
    },
    /// Component was reset
    Reset {
        component: String,
        preserve_data: bool,
    },
    /// Operation was skipped
    Skipped,
    /// Operation was terminated
    Terminated,
    /// Circuit breaker is open, operation blocked
    CircuitOpen,
    /// Recovery failed
    Failed(String),
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreaker {
    /// Create new circuit breaker with default configuration
    pub fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold: 5,
            opened_at: None,
            timeout: Duration::from_secs(60),
            window_duration: Duration::from_secs(60),
            window_start: Instant::now(),
        }
    }

    /// Check if operation can be executed
    pub fn can_execute(&mut self) -> bool {
        self.update_state();

        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => true,
        }
    }

    /// Record successful operation
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.state = CircuitState::Closed;
                self.failure_count = 0;
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count = 0;
            }
            CircuitState::Open => {} // Shouldn't happen
        }
    }

    /// Record failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
            self.opened_at = Some(Instant::now());
        }
    }

    /// Force reset to closed state
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.opened_at = None;
        self.window_start = Instant::now();
    }

    fn update_state(&mut self) {
        // Reset window if expired
        if self.window_start.elapsed() > self.window_duration {
            self.window_start = Instant::now();
            self.failure_count = 0;
        }

        // Check if we should transition from Open to HalfOpen
        if self.state == CircuitState::Open {
            if let Some(opened_at) = self.opened_at {
                if opened_at.elapsed() > self.timeout {
                    self.state = CircuitState::HalfOpen;
                }
            }
        }
    }
}

impl Default for FallbackRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FallbackRegistry {
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: String, strategy: F)
    where
        F: Fn() -> Result<(), MemScopeError> + Send + Sync + 'static,
    {
        self.strategies.insert(name, Box::new(strategy));
    }

    pub fn execute(&self, name: &str) -> Result<(), Box<MemScopeError>> {
        if let Some(strategy) = self.strategies.get(name) {
            strategy().map_err(Box::new)
        } else {
            Err(Box::new(MemScopeError::new(
                ErrorKind::ConfigurationError,
                &format!("Fallback strategy '{}' not found", name),
            )))
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            default_max_attempts: 3,
            default_initial_delay: Duration::from_millis(100),
            default_max_delay: Duration::from_secs(10),
            default_backoff_multiplier: 2.0,
            enable_jitter: true,
        }
    }
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_strategy_creation() {
        let strategy = RecoveryStrategy::new();
        assert!(strategy.action_map.contains_key(&ErrorKind::MemoryError));
        assert!(strategy
            .action_map
            .contains_key(&ErrorKind::ConfigurationError));
    }

    #[test]
    fn test_circuit_breaker_basic() {
        let mut breaker = CircuitBreaker::new();

        // Should start closed
        assert!(breaker.can_execute());
        assert_eq!(breaker.state, CircuitState::Closed);

        // Record failures to open circuit
        for _ in 0..5 {
            breaker.record_failure();
        }

        assert!(!breaker.can_execute());
        assert_eq!(breaker.state, CircuitState::Open);
    }

    #[test]
    fn test_recovery_action_selection() {
        let mut strategy = RecoveryStrategy::new();

        let memory_error = MemScopeError::new(ErrorKind::MemoryError, "allocation failed");
        let result = strategy.recover(&memory_error);

        match result {
            RecoveryResult::Retry { .. } => {} // Expected
            _ => panic!("Expected retry for memory error"),
        }
    }

    #[test]
    fn test_fallback_registry() {
        let mut registry = FallbackRegistry::new();

        registry.register("test_fallback".to_string(), || Ok(()));

        assert!(registry.execute("test_fallback").is_ok());
        assert!(registry.execute("nonexistent").is_err());
    }

    #[test]
    fn test_degradation_levels() {
        let levels = [
            DegradationLevel::Minimal,
            DegradationLevel::Moderate,
            DegradationLevel::Significant,
            DegradationLevel::Severe,
        ];

        // Test ordering
        assert!(levels[0] < levels[1]);
        assert!(levels[1] < levels[2]);
        assert!(levels[2] < levels[3]);
    }
}
