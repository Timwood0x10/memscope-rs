//! 变量日志处理器
//! 
//! 这个模块实现了基于日志的变量名关联机制，
//! 通过解析临时日志文件来为JSON中的指针地址匹配变量名。

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use crate::types::TrackingResult;
use serde_json::Value;

/// 变量日志记录
#[derive(Debug, Clone)]
pub struct VarLogEntry {
    pub ptr: usize,
    pub var_name: String,
    pub type_name: String,
    pub timestamp: u64,
}

/// 变量日志处理器
pub struct VarLogProcessor {
    log_file_path: String,
}

impl VarLogProcessor {
    /// 创建新的日志处理器
    pub fn new(log_file_path: &str) -> Self {
        Self {
            log_file_path: log_file_path.to_string(),
        }
    }

    /// 记录变量信息到日志文件
    /// 
    /// 格式: MEMSCOPE_VAR|timestamp|ptr|var_name|type_name
    pub fn log_variable(
        log_file_path: &str,
        ptr: usize,
        var_name: &str,
        type_name: &str,
    ) -> TrackingResult<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let log_entry = format!(
            "MEMSCOPE_VAR|{}|0x{:x}|{}|{}\n",
            timestamp, ptr, var_name, type_name
        );

        // 使用追加模式写入日志，静默失败以不影响主程序
        if let Err(_) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .and_then(|mut file| file.write_all(log_entry.as_bytes()))
        {
            // 静默失败，不影响主程序运行
        }

        Ok(())
    }

    /// 解析日志文件，构建指针到变量名的映射
    pub fn parse_log_file(&self) -> TrackingResult<HashMap<usize, VarLogEntry>> {
        let mut var_map = HashMap::new();

        if !std::path::Path::new(&self.log_file_path).exists() {
            return Ok(var_map);
        }

        let file = std::fs::File::open(&self.log_file_path)
            .map_err(|e| crate::types::TrackingError::IoError(e))?;
        
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|e| crate::types::TrackingError::IoError(e))?;
            
            if let Some(entry) = self.parse_log_line(&line) {
                // 如果同一个指针有多个记录，保留最新的
                if let Some(existing) = var_map.get(&entry.ptr) {
                    if entry.timestamp > existing.timestamp {
                        var_map.insert(entry.ptr, entry);
                    }
                } else {
                    var_map.insert(entry.ptr, entry);
                }
            }
        }

        println!("DEBUG: Parsed {} variable entries from log", var_map.len());
        for (ptr, entry) in var_map.iter().take(5) {
            println!("  Log entry: 0x{:x} -> '{}' ({})", ptr, entry.var_name, entry.type_name);
        }
        Ok(var_map)
    }

    /// 解析单行日志
    fn parse_log_line(&self, line: &str) -> Option<VarLogEntry> {
        if !line.starts_with("MEMSCOPE_VAR|") {
            return None;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 5 {
            return None;
        }

        let timestamp = parts[1].parse::<u64>().ok()?;
        let ptr_str = parts[2].strip_prefix("0x")?;
        let ptr = usize::from_str_radix(ptr_str, 16).ok()?;
        let var_name = parts[3].to_string();
        let type_name = parts[4].to_string();

        Some(VarLogEntry {
            ptr,
            var_name,
            type_name,
            timestamp,
        })
    }

    /// 处理JSON文件，用日志中的变量名替换指针地址
    pub fn enhance_json_with_var_names(
        &self,
        json_file_path: &str,
        output_file_path: &str,
    ) -> TrackingResult<usize> {
        // 1. 解析日志文件
        let var_map = self.parse_log_file()?;
        if var_map.is_empty() {
            tracing::debug!("No variable entries found in log, skipping enhancement");
            // 如果没有日志，直接复制原文件
            if json_file_path != output_file_path {
                fs::copy(json_file_path, output_file_path)
                    .map_err(|e| crate::types::TrackingError::IoError(e))?;
            }
            return Ok(0);
        }

        // 2. 读取JSON文件
        let json_content = fs::read_to_string(json_file_path)
            .map_err(|e| crate::types::TrackingError::IoError(e))?;

        let mut json_value: Value = serde_json::from_str(&json_content)?;

        // 3. 增强JSON数据
        println!("DEBUG: Starting JSON enhancement with {} log entries", var_map.len());
        let enhanced_count = self.enhance_json_value(&mut json_value, &var_map);
        println!("DEBUG: Enhanced {} entries in JSON", enhanced_count);

        // 4. 写入增强后的JSON
        let enhanced_json = serde_json::to_string_pretty(&json_value)?;

        fs::write(output_file_path, enhanced_json)
            .map_err(|e| crate::types::TrackingError::IoError(e))?;

        tracing::info!(
            "Enhanced JSON with {} variable names, saved to {}",
            enhanced_count, output_file_path
        );

        Ok(enhanced_count)
    }

    /// 递归增强JSON值
    fn enhance_json_value(&self, value: &mut Value, var_map: &HashMap<usize, VarLogEntry>) -> usize {
        let mut enhanced_count = 0;

        match value {
            Value::Object(obj) => {
                // 处理分配记录 - 检查两种可能的指针字段
                let ptr_field = obj.get("ptr").or_else(|| obj.get("memory_address"));
                if let (Some(ptr_val), Some(var_name_val)) = (ptr_field, obj.get("variable_name")) {
                    let ptr = if let Some(ptr_num) = ptr_val.as_u64() {
                        ptr_num as usize
                    } else if let Some(ptr_str) = ptr_val.as_str() {
                        // 处理十六进制字符串格式 "0x..."
                        if let Some(hex_str) = ptr_str.strip_prefix("0x") {
                            usize::from_str_radix(hex_str, 16).unwrap_or(0)
                        } else {
                            0
                        }
                    } else {
                        0
                    };
                    
                    if ptr != 0 {
                        if enhanced_count < 10 { // 只打印前几个用于调试
                            println!("  Checking JSON ptr: 0x{:x}, var_name: {:?}", ptr, var_name_val.as_str());
                        }
                        if let Some(var_entry) = var_map.get(&ptr) {
                            println!("  MATCH FOUND! 0x{:x} -> '{}'", ptr, var_entry.var_name);
                            // 如果当前变量名是自动生成的或为空，则替换
                            let should_replace = match var_name_val.as_str() {
                                Some(name) => {
                                    name.starts_with("medium_object_") ||
                                    name.starts_with("large_object_") ||
                                    name.starts_with("small_object_") ||
                                    name.is_empty()
                                }
                                None => true,
                            };

                            if should_replace {
                                // 先获取需要的值，然后再进行可变借用
                                let new_var_name = var_entry.var_name.clone();
                                let new_type_name = var_entry.type_name.clone();
                                
                                // 更新变量名
                                if let Some(var_name_val_mut) = obj.get_mut("variable_name") {
                                    *var_name_val_mut = Value::String(new_var_name);
                                    enhanced_count += 1;
                                }
                                
                                // 同时更新类型名
                                if let Some(type_name_val) = obj.get_mut("type_name") {
                                    *type_name_val = Value::String(new_type_name);
                                }
                            }
                        }
                    }
                }

                // 处理其他可能包含变量名的字段
                if let Some(ptr_val) = obj.get("ptr") {
                    if let Some(ptr_num) = ptr_val.as_u64() {
                        let ptr = ptr_num as usize;
                        if let Some(var_entry) = var_map.get(&ptr) {
                            // 先获取需要的值，然后再进行可变借用
                            let new_var_name = var_entry.var_name.clone();
                            
                            if let Some(variable_val) = obj.get_mut("variable") {
                                *variable_val = Value::String(new_var_name);
                                enhanced_count += 1;
                            }
                        }
                    }
                }

                // 递归处理所有子对象
                for (_, v) in obj.iter_mut() {
                    enhanced_count += self.enhance_json_value(v, var_map);
                }
            }
            Value::Array(arr) => {
                // 递归处理数组中的所有元素
                for item in arr.iter_mut() {
                    enhanced_count += self.enhance_json_value(item, var_map);
                }
            }
            _ => {
                // 其他类型不需要处理
            }
        }

        enhanced_count
    }

    /// 清理临时日志文件
    pub fn cleanup_log_file(&self) -> TrackingResult<()> {
        if std::path::Path::new(&self.log_file_path).exists() {
            fs::remove_file(&self.log_file_path)
                .map_err(|e| crate::types::TrackingError::IoError(e))?;
            tracing::debug!("Cleaned up log file: {}", self.log_file_path);
        }
        Ok(())
    }

    /// 一站式处理：增强JSON并清理日志
    pub fn process_and_cleanup(
        log_file_path: &str,
        json_file_path: &str,
        output_file_path: &str,
    ) -> TrackingResult<usize> {
        let processor = VarLogProcessor::new(log_file_path);
        
        // 增强JSON
        let enhanced_count = processor.enhance_json_with_var_names(json_file_path, output_file_path)?;
        
        // 清理日志文件
        processor.cleanup_log_file()?;
        
        // 如果输出文件和输入文件不同，删除原始JSON
        if json_file_path != output_file_path && std::path::Path::new(json_file_path).exists() {
            let _ = fs::remove_file(json_file_path);
            tracing::debug!("Cleaned up original JSON file: {}", json_file_path);
        }
        
        Ok(enhanced_count)
    }
}