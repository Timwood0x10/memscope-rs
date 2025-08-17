//! Binary serialization implementations for memory layout types

use crate::core::types::{
    ContainerAnalysis, FieldLayoutInfo, LayoutEfficiency, MemoryLayoutInfo, OptimizationPotential,
    PaddingAnalysis, PaddingLocation, PaddingReason,
};
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::serializable::{primitives, BinarySerializable};
use std::io::{Read, Write};

impl BinarySerializable for PaddingReason {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        match self {
            PaddingReason::FieldAlignment => primitives::write_u8(writer, 0),
            PaddingReason::StructAlignment => primitives::write_u8(writer, 1),
            PaddingReason::EnumDiscriminant => primitives::write_u8(writer, 2),
            PaddingReason::Other(s) => {
                let mut size = primitives::write_u8(writer, 3)?;
                size += primitives::write_string(writer, s)?;
                Ok(size)
            }
        }
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let type_id = primitives::read_u8(reader)?;
        match type_id {
            0 => Ok(PaddingReason::FieldAlignment),
            1 => Ok(PaddingReason::StructAlignment),
            2 => Ok(PaddingReason::EnumDiscriminant),
            3 => Ok(PaddingReason::Other(primitives::read_string(reader)?)),
            _ => Err(BinaryExportError::CorruptedData(format!(
                "Invalid padding reason type ID: {}",
                type_id
            ))),
        }
    }

    fn binary_size(&self) -> usize {
        match self {
            PaddingReason::FieldAlignment
            | PaddingReason::StructAlignment
            | PaddingReason::EnumDiscriminant => 1,
            PaddingReason::Other(s) => 1 + 4 + s.len(),
        }
    }
}

impl BinarySerializable for PaddingLocation {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;
        size += primitives::write_usize(writer, self.start_offset)?;
        size += primitives::write_usize(writer, self.size)?;
        size += self.reason.write_binary(writer)?;
        Ok(size)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let start_offset = primitives::read_usize(reader)?;
        let size = primitives::read_usize(reader)?;
        let reason = PaddingReason::read_binary(reader)?;

        Ok(PaddingLocation {
            start_offset,
            size,
            reason,
        })
    }

    fn binary_size(&self) -> usize {
        8 + 8 + self.reason.binary_size() // start_offset + size + reason
    }
}

impl BinarySerializable for PaddingAnalysis {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;
        size += primitives::write_usize(writer, self.total_padding_bytes)?;
        size += primitives::write_vec(writer, &self.padding_locations)?;
        size += primitives::write_f64(writer, self.padding_ratio)?;

        // Write optimization suggestions
        size += primitives::write_u32(writer, self.optimization_suggestions.len() as u32)?;
        for suggestion in &self.optimization_suggestions {
            size += primitives::write_string(writer, suggestion)?;
        }

        Ok(size)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let total_padding_bytes = primitives::read_usize(reader)?;
        let padding_locations = primitives::read_vec(reader)?;
        let padding_ratio = primitives::read_f64(reader)?;

        // Read optimization suggestions
        let suggestions_len = primitives::read_u32(reader)? as usize;
        let mut optimization_suggestions = Vec::with_capacity(suggestions_len);
        for _ in 0..suggestions_len {
            optimization_suggestions.push(primitives::read_string(reader)?);
        }

        Ok(PaddingAnalysis {
            total_padding_bytes,
            padding_locations,
            padding_ratio,
            optimization_suggestions,
        })
    }

    fn binary_size(&self) -> usize {
        let mut size = 8 + 8 + 4; // total_padding_bytes + padding_ratio + suggestions_len

        // padding_locations
        size += 4; // vec length
        for location in &self.padding_locations {
            size += location.binary_size();
        }

        // optimization_suggestions
        for suggestion in &self.optimization_suggestions {
            size += 4 + suggestion.len(); // length + content
        }

        size
    }
}

impl BinarySerializable for OptimizationPotential {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        match self {
            OptimizationPotential::None => primitives::write_u8(writer, 0),
            OptimizationPotential::Minor { potential_savings } => {
                let mut size = primitives::write_u8(writer, 1)?;
                size += primitives::write_usize(writer, *potential_savings)?;
                Ok(size)
            }
            OptimizationPotential::Moderate {
                potential_savings,
                suggestions,
            } => {
                let mut size = primitives::write_u8(writer, 2)?;
                size += primitives::write_usize(writer, *potential_savings)?;
                size += primitives::write_u32(writer, suggestions.len() as u32)?;
                for suggestion in suggestions {
                    size += primitives::write_string(writer, suggestion)?;
                }
                Ok(size)
            }
            OptimizationPotential::Major {
                potential_savings,
                suggestions,
            } => {
                let mut size = primitives::write_u8(writer, 3)?;
                size += primitives::write_usize(writer, *potential_savings)?;
                size += primitives::write_u32(writer, suggestions.len() as u32)?;
                for suggestion in suggestions {
                    size += primitives::write_string(writer, suggestion)?;
                }
                Ok(size)
            }
        }
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let type_id = primitives::read_u8(reader)?;
        match type_id {
            0 => Ok(OptimizationPotential::None),
            1 => {
                let potential_savings = primitives::read_usize(reader)?;
                Ok(OptimizationPotential::Minor { potential_savings })
            }
            2 => {
                let potential_savings = primitives::read_usize(reader)?;
                let suggestions_len = primitives::read_u32(reader)? as usize;
                let mut suggestions = Vec::with_capacity(suggestions_len);
                for _ in 0..suggestions_len {
                    suggestions.push(primitives::read_string(reader)?);
                }
                Ok(OptimizationPotential::Moderate {
                    potential_savings,
                    suggestions,
                })
            }
            3 => {
                let potential_savings = primitives::read_usize(reader)?;
                let suggestions_len = primitives::read_u32(reader)? as usize;
                let mut suggestions = Vec::with_capacity(suggestions_len);
                for _ in 0..suggestions_len {
                    suggestions.push(primitives::read_string(reader)?);
                }
                Ok(OptimizationPotential::Major {
                    potential_savings,
                    suggestions,
                })
            }
            _ => Err(BinaryExportError::CorruptedData(format!(
                "Invalid optimization potential type ID: {}",
                type_id
            ))),
        }
    }

    fn binary_size(&self) -> usize {
        match self {
            OptimizationPotential::None => 1,
            OptimizationPotential::Minor { .. } => 1 + 8,
            OptimizationPotential::Moderate { suggestions, .. }
            | OptimizationPotential::Major { suggestions, .. } => {
                let mut size = 1 + 8 + 4; // type + potential_savings + suggestions_len
                for suggestion in suggestions {
                    size += 4 + suggestion.len(); // length + content
                }
                size
            }
        }
    }
}

impl BinarySerializable for LayoutEfficiency {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;
        size += primitives::write_f64(writer, self.memory_utilization)?;
        size += primitives::write_f64(writer, self.cache_friendliness)?;
        size += primitives::write_usize(writer, self.alignment_waste)?;
        size += self.optimization_potential.write_binary(writer)?;
        Ok(size)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let memory_utilization = primitives::read_f64(reader)?;
        let cache_friendliness = primitives::read_f64(reader)?;
        let alignment_waste = primitives::read_usize(reader)?;
        let optimization_potential = OptimizationPotential::read_binary(reader)?;

        Ok(LayoutEfficiency {
            memory_utilization,
            cache_friendliness,
            alignment_waste,
            optimization_potential,
        })
    }

    fn binary_size(&self) -> usize {
        8 + 8 + 8 + self.optimization_potential.binary_size()
    }
}

impl BinarySerializable for FieldLayoutInfo {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;
        size += primitives::write_string(writer, &self.field_name)?;
        size += primitives::write_string(writer, &self.field_type)?;
        size += primitives::write_usize(writer, self.offset)?;
        size += primitives::write_usize(writer, self.size)?;
        size += primitives::write_usize(writer, self.alignment)?;
        size += primitives::write_u8(writer, if self.is_padding { 1 } else { 0 })?;
        Ok(size)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let field_name = primitives::read_string(reader)?;
        let field_type = primitives::read_string(reader)?;
        let offset = primitives::read_usize(reader)?;
        let size = primitives::read_usize(reader)?;
        let alignment = primitives::read_usize(reader)?;
        let is_padding = primitives::read_u8(reader)? == 1;

        Ok(FieldLayoutInfo {
            field_name,
            field_type,
            offset,
            size,
            alignment,
            is_padding,
        })
    }

    fn binary_size(&self) -> usize {
        4 + self.field_name.len() + // field_name
        4 + self.field_type.len() + // field_type
        8 + 8 + 8 + 1 // offset + size + alignment + is_padding
    }
}

// For now, we'll implement simplified versions of the complex container types
// These can be expanded later as needed

impl BinarySerializable for ContainerAnalysis {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        // For now, serialize as JSON string to maintain compatibility
        // This can be optimized later with full binary serialization
        let json_str = serde_json::to_string(self).map_err(|e| {
            BinaryExportError::CorruptedData(format!(
                "Container analysis JSON serialization failed: {e}"
            ))
        })?;
        primitives::write_string(writer, &json_str)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let json_str = primitives::read_string(reader)?;
        serde_json::from_str(&json_str).map_err(|e| {
            BinaryExportError::CorruptedData(format!(
                "Container analysis JSON deserialization failed: {e}"
            ))
        })
    }

    fn binary_size(&self) -> usize {
        // Estimate based on JSON serialization
        if let Ok(json_str) = serde_json::to_string(self) {
            4 + json_str.len()
        } else {
            4 // Just the length field if serialization fails
        }
    }
}

impl BinarySerializable for MemoryLayoutInfo {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;
        size += primitives::write_usize(writer, self.total_size)?;
        size += primitives::write_usize(writer, self.alignment)?;
        size += primitives::write_vec(writer, &self.field_layout)?;
        size += self.padding_info.write_binary(writer)?;
        size += self.layout_efficiency.write_binary(writer)?;
        size += primitives::write_option(writer, &self.container_analysis)?;
        Ok(size)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let total_size = primitives::read_usize(reader)?;
        let alignment = primitives::read_usize(reader)?;
        let field_layout = primitives::read_vec(reader)?;
        let padding_info = PaddingAnalysis::read_binary(reader)?;
        let layout_efficiency = LayoutEfficiency::read_binary(reader)?;
        let container_analysis = primitives::read_option(reader)?;

        Ok(MemoryLayoutInfo {
            total_size,
            alignment,
            field_layout,
            padding_info,
            layout_efficiency,
            container_analysis,
        })
    }

    fn binary_size(&self) -> usize {
        let mut size = 8 + 8; // total_size + alignment

        // field_layout
        size += 4; // vec length
        for field in &self.field_layout {
            size += field.binary_size();
        }

        size += self.padding_info.binary_size();
        size += self.layout_efficiency.binary_size();

        // container_analysis (optional)
        size += 1; // option flag
        if let Some(ref analysis) = self.container_analysis {
            size += analysis.binary_size();
        }

        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_padding_reason_serialization() {
        let reasons = vec![
            PaddingReason::FieldAlignment,
            PaddingReason::StructAlignment,
            PaddingReason::EnumDiscriminant,
            PaddingReason::Other("custom reason".to_string()),
        ];

        for original_reason in reasons {
            let mut buffer = Vec::new();
            original_reason.write_binary(&mut buffer).expect("Test operation failed");

            let mut cursor = Cursor::new(&buffer);
            let read_reason = PaddingReason::read_binary(&mut cursor).expect("Failed to get test value");

            assert_eq!(original_reason, read_reason);
        }
    }

    #[test]
    fn test_field_layout_info_serialization() {
        let field = FieldLayoutInfo {
            field_name: "test_field".to_string(),
            field_type: "u32".to_string(),
            offset: 8,
            size: 4,
            alignment: 4,
            is_padding: false,
        };

        let mut buffer = Vec::new();
        let written_size = field.write_binary(&mut buffer).expect("Test operation failed");

        // Verify size calculation
        assert_eq!(written_size, field.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_field = FieldLayoutInfo::read_binary(&mut cursor).expect("Failed to get test value");

        assert_eq!(field, read_field);
    }

    #[test]
    fn test_optimization_potential_serialization() {
        let potentials = vec![
            OptimizationPotential::None,
            OptimizationPotential::Minor {
                potential_savings: 100,
            },
            OptimizationPotential::Moderate {
                potential_savings: 500,
                suggestions: vec!["reorder fields".to_string()],
            },
            OptimizationPotential::Major {
                potential_savings: 1000,
                suggestions: vec!["use smaller types".to_string(), "pack struct".to_string()],
            },
        ];

        for original_potential in potentials {
            let mut buffer = Vec::new();
            let written_size = original_potential.write_binary(&mut buffer).expect("Test operation failed");

            // Verify size calculation
            assert_eq!(written_size, original_potential.binary_size());

            let mut cursor = Cursor::new(&buffer);
            let read_potential = OptimizationPotential::read_binary(&mut cursor).expect("Failed to get test value");

            assert_eq!(original_potential, read_potential);
        }
    }
}
