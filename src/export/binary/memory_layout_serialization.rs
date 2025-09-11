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
                "Invalid padding reason type ID: {type_id}"
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
                "Invalid optimization potential type ID: {type_id}"
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
            original_reason
                .write_binary(&mut buffer)
                .expect("Test operation failed");

            let mut cursor = Cursor::new(&buffer);
            let read_reason =
                PaddingReason::read_binary(&mut cursor).expect("Failed to get test value");

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
        let written_size = field
            .write_binary(&mut buffer)
            .expect("Test operation failed");

        // Verify size calculation
        assert_eq!(written_size, field.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_field =
            FieldLayoutInfo::read_binary(&mut cursor).expect("Failed to get test value");

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
            let written_size = original_potential
                .write_binary(&mut buffer)
                .expect("Test operation failed");

            // Verify size calculation
            assert_eq!(written_size, original_potential.binary_size());

            let mut cursor = Cursor::new(&buffer);
            let read_potential =
                OptimizationPotential::read_binary(&mut cursor).expect("Failed to get test value");

            assert_eq!(original_potential, read_potential);
        }
    }

    #[test]
    fn test_memory_layout_serialization() {
        let layout = MemoryLayoutInfo {
            total_size: 64,
            alignment: 8,
            field_layout: vec![
                FieldLayoutInfo {
                    field_name: "field1".to_string(),
                    field_type: "u32".to_string(),
                    offset: 0,
                    size: 4,
                    alignment: 4,
                    is_padding: false,
                },
                FieldLayoutInfo {
                    field_name: "padding".to_string(),
                    field_type: "padding".to_string(),
                    offset: 4,
                    size: 4,
                    alignment: 1,
                    is_padding: true,
                },
                FieldLayoutInfo {
                    field_name: "field2".to_string(),
                    field_type: "u64".to_string(),
                    offset: 8,
                    size: 8,
                    alignment: 8,
                    is_padding: false,
                },
            ],
            padding_info: PaddingAnalysis {
                total_padding_bytes: 4,
                padding_locations: vec![
                    PaddingLocation {
                        start_offset: 4,
                        size: 4,
                        reason: PaddingReason::FieldAlignment,
                    },
                ],
                padding_ratio: 0.25,
                optimization_suggestions: vec!["Reorder fields".to_string()],
            },
            layout_efficiency: LayoutEfficiency {
                memory_utilization: 0.75,
                cache_friendliness: 85.0,
                alignment_waste: 4,
                optimization_potential: OptimizationPotential::Minor {
                    potential_savings: 4,
                },
            },
            container_analysis: None,
        };

        let mut buffer = Vec::new();
        let written_size = layout
            .write_binary(&mut buffer)
            .expect("Failed to write memory layout");

        // Verify size calculation
        assert_eq!(written_size, layout.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_layout = MemoryLayoutInfo::read_binary(&mut cursor).expect("Failed to read memory layout");

        assert_eq!(layout, read_layout);
    }

    #[test]
    fn test_padding_location_serialization() {
        let padding_locations = vec![
            PaddingLocation {
                start_offset: 0,
                size: 4,
                reason: PaddingReason::FieldAlignment,
            },
            PaddingLocation {
                start_offset: 8,
                size: 8,
                reason: PaddingReason::StructAlignment,
            },
            PaddingLocation {
                start_offset: 16,
                size: 1,
                reason: PaddingReason::EnumDiscriminant,
            },
            PaddingLocation {
                start_offset: 24,
                size: 16,
                reason: PaddingReason::Other("custom alignment".to_string()),
            },
        ];

        for original_location in padding_locations {
            let mut buffer = Vec::new();
            let written_size = original_location
                .write_binary(&mut buffer)
                .expect("Failed to write padding location");

            // Verify size calculation
            assert_eq!(written_size, original_location.binary_size());

            let mut cursor = Cursor::new(&buffer);
            let read_location = PaddingLocation::read_binary(&mut cursor).expect("Failed to read padding location");

            assert_eq!(original_location, read_location);
        }
    }

    #[test]
    fn test_empty_memory_layout_serialization() {
        let layout = MemoryLayoutInfo {
            total_size: 0,
            alignment: 1,
            field_layout: vec![],
            padding_info: PaddingAnalysis {
                total_padding_bytes: 0,
                padding_locations: vec![],
                padding_ratio: 0.0,
                optimization_suggestions: vec![],
            },
            layout_efficiency: LayoutEfficiency {
                memory_utilization: 1.0,
                cache_friendliness: 100.0,
                alignment_waste: 0,
                optimization_potential: OptimizationPotential::None,
            },
            container_analysis: None,
        };

        let mut buffer = Vec::new();
        let written_size = layout
            .write_binary(&mut buffer)
            .expect("Failed to write empty memory layout");

        assert_eq!(written_size, layout.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_layout = MemoryLayoutInfo::read_binary(&mut cursor).expect("Failed to read empty memory layout");

        assert_eq!(layout, read_layout);
    }

    #[test]
    fn test_large_memory_layout_serialization() {
        // Test with many fields
        let mut fields = Vec::new();
        let mut padding_info = Vec::new();

        for i in 0..100 {
            fields.push(FieldLayoutInfo {
                field_name: format!("field_{}", i),
                field_type: format!("Type{}", i),
                offset: i * 8,
                size: 8,
                alignment: 8,
                is_padding: i % 10 == 0, // Every 10th field is padding
            });

            if i % 5 == 0 {
                padding_info.push(PaddingLocation {
                    start_offset: i * 8 + 8,
                    size: 4,
                    reason: PaddingReason::FieldAlignment,
                });
            }
        }

        let layout = MemoryLayoutInfo {
            total_size: 800,
            alignment: 8,
            field_layout: fields,
            padding_info: PaddingAnalysis {
                total_padding_bytes: 80,
                padding_locations: padding_info,
                padding_ratio: 0.1,
                optimization_suggestions: vec![
                    "Reorder fields by size".to_string(),
                    "Use packed struct".to_string(),
                ],
            },
            layout_efficiency: LayoutEfficiency {
                memory_utilization: 0.9,
                cache_friendliness: 60.0,
                alignment_waste: 80,
                optimization_potential: OptimizationPotential::Major {
                    potential_savings: 200,
                    suggestions: vec![
                        "Reorder fields by size".to_string(),
                        "Use packed struct".to_string(),
                        "Consider smaller types".to_string(),
                    ],
                },
            },
            container_analysis: None,
        };

        let mut buffer = Vec::new();
        let written_size = layout
            .write_binary(&mut buffer)
            .expect("Failed to write large memory layout");

        assert_eq!(written_size, layout.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_layout = MemoryLayoutInfo::read_binary(&mut cursor).expect("Failed to read large memory layout");

        assert_eq!(layout, read_layout);
    }

    #[test]
    fn test_binary_size_calculations() {
        // Test PaddingReason binary size
        assert_eq!(PaddingReason::FieldAlignment.binary_size(), 1);
        assert_eq!(PaddingReason::StructAlignment.binary_size(), 1);
        assert_eq!(PaddingReason::EnumDiscriminant.binary_size(), 1);
        
        let custom_reason = PaddingReason::Other("test".to_string());
        assert_eq!(custom_reason.binary_size(), 1 + 4 + 4); // tag + length + content

        // Test FieldLayoutInfo binary size
        let field = FieldLayoutInfo {
            field_name: "test".to_string(),
            field_type: "u32".to_string(),
            offset: 0,
            size: 4,
            alignment: 4,
            is_padding: false,
        };
        let expected_size = 4 + 4 + 4 + 3 + 8 + 8 + 8 + 1; // field_name_len + field_name + field_type_len + field_type + offset + size + alignment + is_padding
        assert_eq!(field.binary_size(), expected_size);

        // Test PaddingLocation binary size
        let padding = PaddingLocation {
            start_offset: 0,
            size: 4,
            reason: PaddingReason::FieldAlignment,
        };
        let expected_size = 8 + 8 + 1; // start_offset + size + reason
        assert_eq!(padding.binary_size(), expected_size);

        // Test OptimizationPotential binary size
        assert_eq!(OptimizationPotential::None.binary_size(), 1);
        
        let minor = OptimizationPotential::Minor { potential_savings: 100 };
        assert_eq!(minor.binary_size(), 1 + 8); // tag + savings

        let moderate = OptimizationPotential::Moderate {
            potential_savings: 500,
            suggestions: vec!["test".to_string()],
        };
        let expected_size = 1 + 8 + 4 + (4 + 4); // tag + savings + vec_len + (str_len + str_content)
        assert_eq!(moderate.binary_size(), expected_size);
    }

    #[test]
    fn test_field_layout_with_special_characters() {
        let field = FieldLayoutInfo {
            field_name: "field_with_unicode_🦀".to_string(),
            field_type: "Type<'a, T: Clone + Send>".to_string(),
            offset: 16,
            size: 32,
            alignment: 8,
            is_padding: false,
        };

        let mut buffer = Vec::new();
        let written_size = field
            .write_binary(&mut buffer)
            .expect("Failed to write field with special characters");

        assert_eq!(written_size, field.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_field = FieldLayoutInfo::read_binary(&mut cursor)
            .expect("Failed to read field with special characters");

        assert_eq!(field, read_field);
    }

    #[test]
    fn test_optimization_potential_edge_cases() {
        // Test with empty suggestions
        let moderate_empty = OptimizationPotential::Moderate {
            potential_savings: 0,
            suggestions: vec![],
        };

        let mut buffer = Vec::new();
        let written_size = moderate_empty
            .write_binary(&mut buffer)
            .expect("Failed to write moderate with empty suggestions");

        assert_eq!(written_size, moderate_empty.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_potential = OptimizationPotential::read_binary(&mut cursor)
            .expect("Failed to read moderate with empty suggestions");

        assert_eq!(moderate_empty, read_potential);

        // Test with many suggestions
        let major_many = OptimizationPotential::Major {
            potential_savings: usize::MAX,
            suggestions: (0..50).map(|i| format!("suggestion_{}", i)).collect(),
        };

        let mut buffer = Vec::new();
        let written_size = major_many
            .write_binary(&mut buffer)
            .expect("Failed to write major with many suggestions");

        assert_eq!(written_size, major_many.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_potential = OptimizationPotential::read_binary(&mut cursor)
            .expect("Failed to read major with many suggestions");

        assert_eq!(major_many, read_potential);
    }

    #[test]
    fn test_padding_reason_other_edge_cases() {
        // Test with empty string
        let empty_other = PaddingReason::Other(String::new());
        let mut buffer = Vec::new();
        let written_size = empty_other
            .write_binary(&mut buffer)
            .expect("Failed to write empty other reason");

        assert_eq!(written_size, empty_other.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_reason = PaddingReason::read_binary(&mut cursor)
            .expect("Failed to read empty other reason");

        assert_eq!(empty_other, read_reason);

        // Test with very long string
        let long_string = "a".repeat(10000);
        let long_other = PaddingReason::Other(long_string.clone());
        
        let mut buffer = Vec::new();
        let written_size = long_other
            .write_binary(&mut buffer)
            .expect("Failed to write long other reason");

        assert_eq!(written_size, long_other.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_reason = PaddingReason::read_binary(&mut cursor)
            .expect("Failed to read long other reason");

        assert_eq!(long_other, read_reason);
    }

    #[test]
    fn test_memory_layout_with_mixed_field_types() {
        let layout = MemoryLayoutInfo {
            total_size: 128,
            alignment: 16,
            field_layout: vec![
                FieldLayoutInfo {
                    field_name: "bool_field".to_string(),
                    field_type: "bool".to_string(),
                    offset: 0,
                    size: 1,
                    alignment: 1,
                    is_padding: false,
                },
                FieldLayoutInfo {
                    field_name: "padding1".to_string(),
                    field_type: "padding".to_string(),
                    offset: 1,
                    size: 7,
                    alignment: 1,
                    is_padding: true,
                },
                FieldLayoutInfo {
                    field_name: "u64_field".to_string(),
                    field_type: "u64".to_string(),
                    offset: 8,
                    size: 8,
                    alignment: 8,
                    is_padding: false,
                },
                FieldLayoutInfo {
                    field_name: "array_field".to_string(),
                    field_type: "[u32; 10]".to_string(),
                    offset: 16,
                    size: 40,
                    alignment: 4,
                    is_padding: false,
                },
                FieldLayoutInfo {
                    field_name: "padding2".to_string(),
                    field_type: "padding".to_string(),
                    offset: 56,
                    size: 8,
                    alignment: 1,
                    is_padding: true,
                },
                FieldLayoutInfo {
                    field_name: "ptr_field".to_string(),
                    field_type: "*const u8".to_string(),
                    offset: 64,
                    size: 8,
                    alignment: 8,
                    is_padding: false,
                },
            ],
            padding_info: PaddingAnalysis {
                total_padding_bytes: 15,
                padding_locations: vec![
                    PaddingLocation {
                        start_offset: 1,
                        size: 7,
                        reason: PaddingReason::FieldAlignment,
                    },
                    PaddingLocation {
                        start_offset: 56,
                        size: 8,
                        reason: PaddingReason::StructAlignment,
                    },
                ],
                padding_ratio: 15.0 / 128.0,
                optimization_suggestions: vec![
                    "Move bool field to end".to_string(),
                    "Pack struct with #[repr(packed)]".to_string(),
                ],
            },
            layout_efficiency: LayoutEfficiency {
                memory_utilization: 113.0 / 128.0,
                cache_friendliness: 70.0,
                alignment_waste: 15,
                optimization_potential: OptimizationPotential::Moderate {
                    potential_savings: 15,
                    suggestions: vec![
                        "Move bool field to end".to_string(),
                        "Pack struct with #[repr(packed)]".to_string(),
                    ],
                },
            },
            container_analysis: None,
        };

        let mut buffer = Vec::new();
        let written_size = layout
            .write_binary(&mut buffer)
            .expect("Failed to write mixed field layout");

        assert_eq!(written_size, layout.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_layout = MemoryLayoutInfo::read_binary(&mut cursor)
            .expect("Failed to read mixed field layout");

        assert_eq!(layout, read_layout);
    }

    #[test]
    fn test_serialization_roundtrip_consistency() {
        // Create a complex layout and ensure multiple roundtrips are consistent
        let original_layout = MemoryLayoutInfo {
            total_size: 256,
            alignment: 32,
            field_layout: vec![
                FieldLayoutInfo {
                    field_name: "complex_field".to_string(),
                    field_type: "Option<Box<dyn Trait>>".to_string(),
                    offset: 0,
                    size: 16,
                    alignment: 8,
                    is_padding: false,
                },
            ],
            padding_info: PaddingAnalysis {
                total_padding_bytes: 16,
                padding_locations: vec![
                    PaddingLocation {
                        start_offset: 16,
                        size: 16,
                        reason: PaddingReason::Other("custom alignment requirement".to_string()),
                    },
                ],
                padding_ratio: 16.0 / 256.0,
                optimization_suggestions: vec![
                    "Consider using a different data structure".to_string(),
                    "Reduce alignment requirements".to_string(),
                ],
            },
            layout_efficiency: LayoutEfficiency {
                memory_utilization: 240.0 / 256.0,
                cache_friendliness: 50.0,
                alignment_waste: 16,
                optimization_potential: OptimizationPotential::Major {
                    potential_savings: 128,
                    suggestions: vec![
                        "Consider using a different data structure".to_string(),
                        "Reduce alignment requirements".to_string(),
                    ],
                },
            },
            container_analysis: None,
        };

        // First roundtrip
        let mut buffer1 = Vec::new();
        original_layout.write_binary(&mut buffer1).expect("First write failed");
        
        let mut cursor1 = Cursor::new(&buffer1);
        let layout1 = MemoryLayoutInfo::read_binary(&mut cursor1).expect("First read failed");
        
        assert_eq!(original_layout, layout1);

        // Second roundtrip
        let mut buffer2 = Vec::new();
        layout1.write_binary(&mut buffer2).expect("Second write failed");
        
        let mut cursor2 = Cursor::new(&buffer2);
        let layout2 = MemoryLayoutInfo::read_binary(&mut cursor2).expect("Second read failed");
        
        assert_eq!(layout1, layout2);
        assert_eq!(original_layout, layout2);

        // Verify binary data is identical
        assert_eq!(buffer1, buffer2);
    }
}
