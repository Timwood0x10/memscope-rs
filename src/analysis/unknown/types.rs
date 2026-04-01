use crate::capture::types::ImplementationDifficulty;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownMemoryRegionAnalysis {
    pub total_unknown_bytes: usize,
    pub unknown_percentage: f64,
    pub unknown_categories: Vec<UnknownMemoryCategory>,
    pub potential_causes: Vec<UnknownMemoryCause>,
    pub reduction_strategies: Vec<UnknownRegionReductionStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownMemoryCategory {
    pub category_type: UnknownRegionType,
    pub description: String,
    pub estimated_size: usize,
    pub confidence_level: f64,
    pub examples: Vec<UnknownMemoryExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnknownRegionType {
    MemoryMappedRegions,
    ThreadLocalStorage,
    DynamicLibraryRegions,
    SystemReservedRegions,
    JitCodeRegions,
    ExternalLibraryAllocations,
    GuardPages,
    VdsoRegions,
    AnonymousMappings,
    SharedMemorySegments,
    PreTrackingAllocations,
    CorruptedMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownMemoryExample {
    pub address_range: (usize, usize),
    pub size: usize,
    pub suspected_origin: String,
    pub access_pattern: MemoryAccessPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnknownMemoryCause {
    ForeignFunctionInterface {
        library_name: String,
        function_name: Option<String>,
    },
    MemoryMapping {
        mapping_type: MappingType,
        file_path: Option<String>,
    },
    SystemAllocations {
        allocation_type: SystemAllocationType,
    },
    ThreadingMemory {
        thread_id: Option<u64>,
        memory_type: ThreadMemoryType,
    },
    DynamicLoading {
        library_path: String,
        load_time: u64,
    },
    InstrumentationGaps {
        gap_type: InstrumentationGapType,
        description: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MappingType {
    FileMapping,
    AnonymousMapping,
    SharedMapping,
    DeviceMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemAllocationType {
    KernelBuffers,
    DriverMemory,
    SystemCaches,
    HardwareReserved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreadMemoryType {
    ThreadStack,
    ThreadLocalStorage,
    ThreadControlBlock,
    ThreadSynchronization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstrumentationGapType {
    EarlyBootstrap,
    SignalHandlers,
    InterruptHandlers,
    AtomicOperations,
    CompilerOptimizations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownRegionReductionStrategy {
    pub strategy_type: ReductionStrategyType,
    pub description: String,
    pub implementation_steps: Vec<String>,
    pub expected_improvement: f64,
    pub implementation_difficulty: ImplementationDifficulty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReductionStrategyType {
    EnhancedInstrumentation,
    BetterSymbolResolution,
    MemoryMappingTracking,
    FfiCallInterception,
    SystemCallMonitoring,
    ThreadAwareTracking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRegionInfo {
    pub region_type: String,
    pub description: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMappingInfo {
    pub start_address: usize,
    pub end_address: usize,
    pub permissions: String,
    pub file_path: String,
}

impl LibraryMappingInfo {
    pub fn contains_address(&self, addr: usize) -> bool {
        addr >= self.start_address && addr < self.end_address
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryAccessPattern {
    Sequential,
    Random,
    Sparse,
    Unknown,
}
