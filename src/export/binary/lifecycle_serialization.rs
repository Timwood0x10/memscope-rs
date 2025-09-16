//! Optimized binary serialization for lifecycle tracking data
//!
//! This module provides efficient binary encoding for time-series lifecycle data,
//! optimizing for common patterns in object lifecycle events and access tracking.

use crate::core::types::{
    LifecycleEvent, LifecycleEventType, MemoryAccessEvent, MemoryAccessEventType,
    ObjectLifecycleInfo, MemoryAccessTrackingInfo, EventPerformanceMetrics,
    MemoryAccessEventPerformanceMetrics,
};
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::serializable::{primitives, BinarySerializable};
use std::io::{Read, Write};

/// Optimized binary serialization for ObjectLifecycleInfo
impl BinarySerializable for ObjectLifecycleInfo {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;

        // Write basic info
        size += primitives::write_usize(writer, self.object_id)?;
        size += primitives::write_string(writer, &self.type_name)?;

        // Write lifecycle events with time-series optimization
        size += self.write_lifecycle_events_optimized(writer)?;

        // Write optional total lifetime
        size += primitives::write_option(writer, &self.total_lifetime_ns)?;

        // Write stage durations
        size += self.stage_durations.write_binary(writer)?;

        // Write efficiency metrics
        size += self.efficiency_metrics.write_binary(writer)?;

        // Write lifecycle patterns
        size += primitives::write_vec(writer, &self.lifecycle_patterns)?;

        Ok(size)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let object_id = primitives::read_usize(reader)?;
        let type_name = primitives::read_string(reader)?;
        
        // Read lifecycle events
        let lifecycle_events = Self::read_lifecycle_events_optimized(reader)?;
        
        let total_lifetime_ns = primitives::read_option(reader)?;
        let stage_durations = crate::core::types::LifecycleStageDurations::read_binary(reader)?;
        let efficiency_metrics = crate::core::types::LifecycleEfficiencyMetrics::read_binary(reader)?;
        let lifecycle_patterns = primitives::read_vec(reader)?;

        Ok(ObjectLifecycleInfo {
            object_id,
            type_name,
            lifecycle_events,
            total_lifetime_ns,
            stage_durations,
            efficiency_metrics,
            lifecycle_patterns,
        })
    }

    fn binary_size(&self) -> usize {
        8 + // object_id
        4 + self.type_name.len() + // type_name
        self.calculate_lifecycle_events_size() + // lifecycle_events
        9 + // total_lifetime_ns (1 byte flag + 8 bytes value)
        self.stage_durations.binary_size() +
        self.efficiency_metrics.binary_size() +
        4 + self.lifecycle_patterns.iter().map(|p| p.binary_size()).sum::<usize>()
    }
}

impl ObjectLifecycleInfo {
    /// Write lifecycle events with time-series optimization
    fn write_lifecycle_events_optimized<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;
        
        // Write event count
        size += primitives::write_u32(writer, self.lifecycle_events.len() as u32)?;
        
        if self.lifecycle_events.is_empty() {
            return Ok(size);
        }

        // Use delta encoding for timestamps to save space
        let mut last_timestamp = 0u64;
        
        for event in &self.lifecycle_events {
            // Write event type as compact enum
            size += primitives::write_u8(writer, event.event_type.to_u8())?;
            
            // Delta-encode timestamp
            let timestamp_delta = event.timestamp.saturating_sub(last_timestamp);
            size += self.write_varint(writer, timestamp_delta)?;
            last_timestamp = event.timestamp;
            
            // Write location (simplified)
            size += primitives::write_string(writer, &event.location.file)?;
            size += primitives::write_u32(writer, event.location.line)?;
            size += primitives::write_u32(writer, event.location.column)?;
            
            // Write memory state
            size += event.memory_state.write_binary(writer)?;
            
            // Write performance metrics
            size += event.performance_metrics.write_binary(writer)?;
            
            // Write call stack with string table optimization potential
            size += primitives::write_vec(writer, &event.call_stack)?;
        }

        Ok(size)
    }

    /// Read lifecycle events with time-series optimization
    fn read_lifecycle_events_optimized<R: Read>(reader: &mut R) -> Result<Vec<LifecycleEvent>, BinaryExportError> {
        let event_count = primitives::read_u32(reader)? as usize;
        let mut events = Vec::with_capacity(event_count);
        
        let mut last_timestamp = 0u64;
        
        for _ in 0..event_count {
            let event_type = LifecycleEventType::from_u8(primitives::read_u8(reader)?)?;
            
            // Decode delta timestamp
            let timestamp_delta = Self::read_varint(reader)?;
            let timestamp = last_timestamp + timestamp_delta;
            last_timestamp = timestamp;
            
            // Read location
            let location = crate::core::types::SourceLocation {
                file: primitives::read_string(reader)?,
                line: primitives::read_u32(reader)?,
                column: primitives::read_u32(reader)?,
            };
            
            // Read memory state
            let memory_state = crate::core::types::MemoryState::read_binary(reader)?;
            
            // Read performance metrics
            let performance_metrics = EventPerformanceMetrics::read_binary(reader)?;
            
            // Read call stack
            let call_stack = primitives::read_vec(reader)?;
            
            events.push(LifecycleEvent {
                event_type,
                timestamp,
                location,
                memory_state,
                performance_metrics,
                call_stack,
            });
        }

        Ok(events)
    }

    /// Calculate size needed for lifecycle events
    fn calculate_lifecycle_events_size(&self) -> usize {
        let mut size = 4; // event count
        
        for event in &self.lifecycle_events {
            size += 1; // event type
            size += self.varint_size(event.timestamp); // timestamp (varint encoded)
            size += 4 + event.location.file.len(); // location file
            size += 8; // location line + column
            size += event.memory_state.binary_size();
            size += event.performance_metrics.binary_size();
            size += 4 + event.call_stack.iter().map(|s| 4 + s.len()).sum::<usize>();
        }
        
        size
    }

    /// Write variable-length integer (varint) for timestamp deltas
    fn write_varint<W: Write>(&self, writer: &mut W, mut value: u64) -> Result<usize, BinaryExportError> {
        let mut size = 0;
        
        while value >= 0x80 {
            writer.write_all(&[(value as u8) | 0x80])?;
            value >>= 7;
            size += 1;
        }
        
        writer.write_all(&[value as u8])?;
        size += 1;
        
        Ok(size)
    }

    /// Read variable-length integer
    fn read_varint<R: Read>(reader: &mut R) -> Result<u64, BinaryExportError> {
        let mut result = 0u64;
        let mut shift = 0;
        
        loop {
            let byte = primitives::read_u8(reader)?;
            result |= ((byte & 0x7F) as u64) << shift;
            
            if byte & 0x80 == 0 {
                break;
            }
            
            shift += 7;
            if shift >= 64 {
                return Err(BinaryExportError::CorruptedData(
                    "Varint too long".to_string()
                ));
            }
        }
        
        Ok(result)
    }

    /// Calculate size needed for varint encoding
    fn varint_size(&self, value: u64) -> usize {
        if value < 0x80 { 1 }
        else if value < 0x4000 { 2 }
        else if value < 0x200000 { 3 }
        else if value < 0x10000000 { 4 }
        else if value < 0x800000000 { 5 }
        else if value < 0x40000000000 { 6 }
        else if value < 0x2000000000000 { 7 }
        else if value < 0x100000000000000 { 8 }
        else if value < 0x8000000000000000 { 9 }
        else { 10 }
    }
}

/// Optimized binary serialization for MemoryAccessTrackingInfo
impl BinarySerializable for MemoryAccessTrackingInfo {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;

        // Write basic info
        size += primitives::write_usize(writer, self.region_id)?;
        size += self.address_range.write_binary(writer)?;

        // Write access events with time-series optimization
        size += self.write_access_events_optimized(writer)?;

        // Write access statistics
        size += self.access_statistics.write_binary(writer)?;

        // Write access patterns
        size += primitives::write_vec(writer, &self.access_patterns)?;

        // Write performance impact
        size += self.performance_impact.write_binary(writer)?;

        Ok(size)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let region_id = primitives::read_usize(reader)?;
        let address_range = crate::core::types::AddressRange::read_binary(reader)?;
        let access_events = Self::read_access_events_optimized(reader)?;
        let access_statistics = crate::core::types::MemoryAccessStatistics::read_binary(reader)?;
        let access_patterns = primitives::read_vec(reader)?;
        let performance_impact = crate::core::types::MemoryAccessPerformanceImpact::read_binary(reader)?;

        Ok(MemoryAccessTrackingInfo {
            region_id,
            address_range,
            access_events,
            access_statistics,
            access_patterns,
            performance_impact,
        })
    }

    fn binary_size(&self) -> usize {
        8 + // region_id
        self.address_range.binary_size() +
        self.calculate_access_events_size() +
        self.access_statistics.binary_size() +
        4 + self.access_patterns.iter().map(|p| p.binary_size()).sum::<usize>() +
        self.performance_impact.binary_size()
    }
}

impl MemoryAccessTrackingInfo {
    /// Write access events with time-series optimization
    fn write_access_events_optimized<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;
        
        // Write event count
        size += primitives::write_u32(writer, self.access_events.len() as u32)?;
        
        if self.access_events.is_empty() {
            return Ok(size);
        }

        // Use delta encoding for timestamps and addresses
        let mut last_timestamp = 0u64;
        let mut last_address = 0usize;
        
        for event in &self.access_events {
            // Write event type
            size += primitives::write_u8(writer, event.event_type.to_u8())?;
            
            // Delta-encode timestamp
            let timestamp_delta = event.timestamp.saturating_sub(last_timestamp);
            size += ObjectLifecycleInfo::write_varint_static(writer, timestamp_delta)?;
            last_timestamp = event.timestamp;
            
            // Delta-encode address
            let address_delta = if event.address >= last_address {
                event.address - last_address
            } else {
                event.address // Handle wraparound by using absolute address
            };
            size += ObjectLifecycleInfo::write_varint_static(writer, address_delta as u64)?;
            last_address = event.address;
            
            // Write size
            size += primitives::write_usize(writer, event.size)?;
            
            // Write access pattern
            size += event.access_pattern.write_binary(writer)?;
            
            // Write thread ID (simplified)
            size += primitives::write_u64(writer, unsafe { std::mem::transmute(event.thread_id) })?;
            
            // Write call stack
            size += primitives::write_vec(writer, &event.call_stack)?;
            
            // Write performance metrics
            size += event.performance_metrics.write_binary(writer)?;
        }

        Ok(size)
    }

    /// Read access events with time-series optimization
    fn read_access_events_optimized<R: Read>(reader: &mut R) -> Result<Vec<MemoryAccessEvent>, BinaryExportError> {
        let event_count = primitives::read_u32(reader)? as usize;
        let mut events = Vec::with_capacity(event_count);
        
        let mut last_timestamp = 0u64;
        let mut last_address = 0usize;
        
        for _ in 0..event_count {
            let event_type = MemoryAccessEventType::from_u8(primitives::read_u8(reader)?)?;
            
            // Decode delta timestamp
            let timestamp_delta = ObjectLifecycleInfo::read_varint_static(reader)?;
            let timestamp = last_timestamp + timestamp_delta;
            last_timestamp = timestamp;
            
            // Decode delta address
            let address_delta = ObjectLifecycleInfo::read_varint_static(reader)? as usize;
            let address = last_address + address_delta;
            last_address = address;
            
            let size = primitives::read_usize(reader)?;
            let access_pattern = crate::core::types::AccessPattern::read_binary(reader)?;
            let thread_id = unsafe { std::mem::transmute(primitives::read_u64(reader)?) };
            let call_stack = primitives::read_vec(reader)?;
            let performance_metrics = MemoryAccessEventPerformanceMetrics::read_binary(reader)?;
            
            events.push(MemoryAccessEvent {
                event_type,
                timestamp,
                address,
                size,
                access_pattern,
                thread_id,
                call_stack,
                performance_metrics,
            });
        }

        Ok(events)
    }

    /// Calculate size needed for access events
    fn calculate_access_events_size(&self) -> usize {
        let mut size = 4; // event count
        
        for event in &self.access_events {
            size += 1; // event type
            size += ObjectLifecycleInfo::varint_size_static(event.timestamp); // timestamp
            size += ObjectLifecycleInfo::varint_size_static(event.address as u64); // address
            size += 8; // size
            size += event.access_pattern.binary_size();
            size += 8; // thread_id
            size += 4 + event.call_stack.iter().map(|s| 4 + s.len()).sum::<usize>();
            size += event.performance_metrics.binary_size();
        }
        
        size
    }
}

impl ObjectLifecycleInfo {
    /// Static version of write_varint for use in other structs
    fn write_varint_static<W: Write>(writer: &mut W, mut value: u64) -> Result<usize, BinaryExportError> {
        let mut size = 0;
        
        while value >= 0x80 {
            writer.write_all(&[(value as u8) | 0x80])?;
            value >>= 7;
            size += 1;
        }
        
        writer.write_all(&[value as u8])?;
        size += 1;
        
        Ok(size)
    }

    /// Static version of read_varint
    fn read_varint_static<R: Read>(reader: &mut R) -> Result<u64, BinaryExportError> {
        let mut result = 0u64;
        let mut shift = 0;
        
        loop {
            let byte = primitives::read_u8(reader)?;
            result |= ((byte & 0x7F) as u64) << shift;
            
            if byte & 0x80 == 0 {
                break;
            }
            
            shift += 7;
            if shift >= 64 {
                return Err(BinaryExportError::CorruptedData(
                    "Varint too long".to_string()
                ));
            }
        }
        
        Ok(result)
    }

    /// Static version of varint_size
    fn varint_size_static(value: u64) -> usize {
        if value < 0x80 { 1 }
        else if value < 0x4000 { 2 }
        else if value < 0x200000 { 3 }
        else if value < 0x10000000 { 4 }
        else if value < 0x800000000 { 5 }
        else if value < 0x40000000000 { 6 }
        else if value < 0x2000000000000 { 7 }
        else if value < 0x100000000000000 { 8 }
        else if value < 0x8000000000000000 { 9 }
        else { 10 }
    }
}

// Helper trait implementations for enum conversions
impl LifecycleEventType {
    fn to_u8(&self) -> u8 {
        match self {
            LifecycleEventType::Creation => 0,
            LifecycleEventType::Initialization => 1,
            LifecycleEventType::FirstUse => 2,
            LifecycleEventType::Move => 3,
            LifecycleEventType::Copy => 4,
            LifecycleEventType::Clone => 5,
            LifecycleEventType::Borrow => 6,
            LifecycleEventType::MutableBorrow => 7,
            LifecycleEventType::BorrowRelease => 8,
            LifecycleEventType::Modification => 9,
            LifecycleEventType::LastUse => 10,
            LifecycleEventType::Drop => 11,
            LifecycleEventType::Destruction => 12,
            LifecycleEventType::MemoryReclaim => 13,
        }
    }

    fn from_u8(value: u8) -> Result<Self, BinaryExportError> {
        match value {
            0 => Ok(LifecycleEventType::Creation),
            1 => Ok(LifecycleEventType::Initialization),
            2 => Ok(LifecycleEventType::FirstUse),
            3 => Ok(LifecycleEventType::Move),
            4 => Ok(LifecycleEventType::Copy),
            5 => Ok(LifecycleEventType::Clone),
            6 => Ok(LifecycleEventType::Borrow),
            7 => Ok(LifecycleEventType::MutableBorrow),
            8 => Ok(LifecycleEventType::BorrowRelease),
            9 => Ok(LifecycleEventType::Modification),
            10 => Ok(LifecycleEventType::LastUse),
            11 => Ok(LifecycleEventType::Drop),
            12 => Ok(LifecycleEventType::Destruction),
            13 => Ok(LifecycleEventType::MemoryReclaim),
            _ => Err(BinaryExportError::CorruptedData(format!(
                "Invalid lifecycle event type: {}", value
            ))),
        }
    }
}

impl MemoryAccessEventType {
    fn to_u8(&self) -> u8 {
        match self {
            MemoryAccessEventType::Read => 0,
            MemoryAccessEventType::Write => 1,
            MemoryAccessEventType::ReadWrite => 2,
            MemoryAccessEventType::Prefetch => 3,
            MemoryAccessEventType::Flush => 4,
        }
    }

    fn from_u8(value: u8) -> Result<Self, BinaryExportError> {
        match value {
            0 => Ok(MemoryAccessEventType::Read),
            1 => Ok(MemoryAccessEventType::Write),
            2 => Ok(MemoryAccessEventType::ReadWrite),
            3 => Ok(MemoryAccessEventType::Prefetch),
            4 => Ok(MemoryAccessEventType::Flush),
            _ => Err(BinaryExportError::CorruptedData(format!(
                "Invalid memory access event type: {}", value
            ))),
        }
    }
}