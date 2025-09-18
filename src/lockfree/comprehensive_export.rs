//! Comprehensive export functionality for CPU/GPU/Memory analysis data
//! 
//! Provides JSON and enhanced HTML export with detailed resource analytics

use super::resource_integration::ComprehensiveAnalysis;
use super::visualizer::generate_comprehensive_html_report;
// Comprehensive export functionality
use std::path::Path;
use serde_json::{json, Value};

/// Export comprehensive analysis to multiple formats
pub fn export_comprehensive_analysis(
    analysis: &ComprehensiveAnalysis,
    output_dir: &Path,
    base_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;
    
    // Export detailed JSON with CPU/GPU rankings
    export_comprehensive_json(analysis, &output_dir.join(format!("{}_comprehensive.json", base_name)))?;
    
    // Export enhanced HTML dashboard
    generate_comprehensive_html_report(analysis, &output_dir.join(format!("{}_dashboard.html", base_name)))?;
    
    // Export CPU/GPU specific analysis
    export_resource_rankings_json(analysis, &output_dir.join(format!("{}_resource_rankings.json", base_name)))?;
    
    println!("📊 Comprehensive analysis exported to:");
    println!("   📄 JSON: {}_comprehensive.json", base_name);
    println!("   🌐 HTML: {}_dashboard.html", base_name);
    println!("   📈 Rankings: {}_resource_rankings.json", base_name);
    
    Ok(())
}

/// Export comprehensive analysis as detailed JSON
pub fn export_comprehensive_json(
    analysis: &ComprehensiveAnalysis,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut json_data = serde_json::to_value(analysis)?;
    
    // Add computed analytics
    let analytics = compute_comprehensive_analytics(analysis)?;
    json_data["analytics"] = analytics;
    
    // Add resource rankings
    let rankings = compute_resource_rankings(analysis)?;
    json_data["rankings"] = rankings;
    
    // Write formatted JSON
    let pretty_json = serde_json::to_string_pretty(&json_data)?;
    std::fs::write(output_path, pretty_json)?;
    
    Ok(())
}

/// Export resource rankings and analytics as focused JSON
pub fn export_resource_rankings_json(
    analysis: &ComprehensiveAnalysis,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let rankings = compute_detailed_resource_rankings(analysis)?;
    
    let rankings_json = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "summary": {
            "total_samples": analysis.resource_timeline.len(),
            "total_threads": analysis.memory_analysis.thread_stats.len(),
            "primary_bottleneck": analysis.performance_insights.primary_bottleneck,
            "overall_efficiency": (
                analysis.performance_insights.cpu_efficiency_score +
                analysis.performance_insights.memory_efficiency_score +
                analysis.performance_insights.io_efficiency_score
            ) / 3.0
        },
        "cpu_rankings": rankings["cpu_rankings"],
        "gpu_rankings": rankings["gpu_rankings"],
        "memory_rankings": rankings["memory_rankings"],
        "thread_rankings": rankings["thread_rankings"],
        "performance_insights": rankings["performance_insights"]
    });
    
    let pretty_json = serde_json::to_string_pretty(&rankings_json)?;
    std::fs::write(output_path, pretty_json)?;
    
    Ok(())
}

/// Compute comprehensive analytics from the analysis data
fn compute_comprehensive_analytics(analysis: &ComprehensiveAnalysis) -> Result<Value, Box<dyn std::error::Error>> {
    let timeline = &analysis.resource_timeline;
    
    // CPU Analytics
    let cpu_analytics = if !timeline.is_empty() {
        let cpu_usages: Vec<f32> = timeline.iter()
            .map(|r| r.cpu_metrics.overall_usage_percent)
            .collect();
        
        let avg_cpu = cpu_usages.iter().sum::<f32>() / cpu_usages.len() as f32;
        let max_cpu = cpu_usages.iter().fold(0.0f32, |a, &b| a.max(b));
        let min_cpu = cpu_usages.iter().fold(100.0f32, |a, &b| a.min(b));
        
        // CPU trend analysis
        let cpu_trend = if cpu_usages.len() > 5 {
            let first_half = &cpu_usages[..cpu_usages.len()/2];
            let second_half = &cpu_usages[cpu_usages.len()/2..];
            let first_avg = first_half.iter().sum::<f32>() / first_half.len() as f32;
            let second_avg = second_half.iter().sum::<f32>() / second_half.len() as f32;
            
            if second_avg > first_avg + 5.0 {
                "increasing"
            } else if second_avg < first_avg - 5.0 {
                "decreasing"
            } else {
                "stable"
            }
        } else {
            "insufficient_data"
        };
        
        json!({
            "average_usage": avg_cpu,
            "peak_usage": max_cpu,
            "minimum_usage": min_cpu,
            "usage_trend": cpu_trend,
            "cores_detected": timeline.first().map(|r| r.cpu_metrics.per_core_usage.len()).unwrap_or(0),
            "load_average_final": timeline.last().map(|r| r.cpu_metrics.load_average).unwrap_or((0.0, 0.0, 0.0))
        })
    } else {
        json!({
            "average_usage": 0.0,
            "peak_usage": 0.0,
            "minimum_usage": 0.0,
            "usage_trend": "no_data",
            "cores_detected": 0,
            "load_average_final": [0.0, 0.0, 0.0]
        })
    };
    
    // GPU Analytics
    let gpu_analytics = {
        let gpu_samples: Vec<&super::platform_resources::GpuResourceMetrics> = timeline.iter()
            .filter_map(|r| r.gpu_metrics.as_ref())
            .collect();
        
        if !gpu_samples.is_empty() {
            let compute_usages: Vec<f32> = gpu_samples.iter().map(|g| g.compute_usage_percent).collect();
            let memory_usages: Vec<f32> = gpu_samples.iter().map(|g| g.memory_usage_percent).collect();
            let temperatures: Vec<f32> = gpu_samples.iter().map(|g| g.temperature_celsius).collect();
            
            let avg_compute = compute_usages.iter().sum::<f32>() / compute_usages.len() as f32;
            let avg_memory = memory_usages.iter().sum::<f32>() / memory_usages.len() as f32;
            let avg_temp = temperatures.iter().sum::<f32>() / temperatures.len() as f32;
            
            json!({
                "available": true,
                "device_name": gpu_samples.first().map(|g| &g.device_name).unwrap_or(&"Unknown".to_string()),
                "vendor": gpu_samples.first().map(|g| &g.vendor).unwrap_or(&super::platform_resources::GpuVendor::Unknown),
                "average_compute_usage": avg_compute,
                "average_memory_usage": avg_memory,
                "average_temperature": avg_temp,
                "peak_compute_usage": compute_usages.iter().fold(0.0f32, |a, &b| a.max(b)),
                "peak_memory_usage": memory_usages.iter().fold(0.0f32, |a, &b| a.max(b)),
                "total_samples": gpu_samples.len()
            })
        } else {
            json!({
                "available": false,
                "reason": "no_gpu_detected_or_unsupported_platform"
            })
        }
    };
    
    // Memory Analytics (from existing memory analysis)
    let memory_analytics = json!({
        "total_allocations": analysis.memory_analysis.summary.total_allocations,
        "total_deallocations": analysis.memory_analysis.summary.total_deallocations,
        "peak_memory_usage_bytes": analysis.memory_analysis.summary.peak_memory_usage,
        "peak_memory_usage_mb": analysis.memory_analysis.summary.peak_memory_usage as f64 / 1024.0 / 1024.0,
        "allocation_efficiency": if analysis.memory_analysis.summary.total_allocations > 0 {
            analysis.memory_analysis.summary.total_deallocations as f64 / analysis.memory_analysis.summary.total_allocations as f64 * 100.0
        } else { 0.0 },
        "tracked_threads": analysis.memory_analysis.thread_stats.len()
    });
    
    // I/O Analytics
    let io_analytics = if !timeline.is_empty() {
        let total_disk_read: u64 = timeline.iter().map(|r| r.io_metrics.disk_read_bytes_per_sec).sum();
        let total_disk_write: u64 = timeline.iter().map(|r| r.io_metrics.disk_write_bytes_per_sec).sum();
        let total_network_rx: u64 = timeline.iter().map(|r| r.io_metrics.network_rx_bytes_per_sec).sum();
        let total_network_tx: u64 = timeline.iter().map(|r| r.io_metrics.network_tx_bytes_per_sec).sum();
        
        json!({
            "total_disk_read_mb": total_disk_read as f64 / 1024.0 / 1024.0,
            "total_disk_write_mb": total_disk_write as f64 / 1024.0 / 1024.0,
            "total_network_rx_mb": total_network_rx as f64 / 1024.0 / 1024.0,
            "total_network_tx_mb": total_network_tx as f64 / 1024.0 / 1024.0,
            "avg_disk_throughput_mbps": (total_disk_read + total_disk_write) as f64 / timeline.len() as f64 / 1024.0 / 1024.0,
            "avg_network_throughput_mbps": (total_network_rx + total_network_tx) as f64 / timeline.len() as f64 / 1024.0 / 1024.0
        })
    } else {
        json!({
            "total_disk_read_mb": 0.0,
            "total_disk_write_mb": 0.0,
            "total_network_rx_mb": 0.0,
            "total_network_tx_mb": 0.0,
            "avg_disk_throughput_mbps": 0.0,
            "avg_network_throughput_mbps": 0.0
        })
    };
    
    Ok(json!({
        "cpu": cpu_analytics,
        "gpu": gpu_analytics,
        "memory": memory_analytics,
        "io": io_analytics,
        "correlations": analysis.correlation_metrics
    }))
}

/// Compute resource rankings for export
fn compute_resource_rankings(analysis: &ComprehensiveAnalysis) -> Result<Value, Box<dyn std::error::Error>> {
    let timeline = &analysis.resource_timeline;
    
    // CPU usage ranking by sample
    let mut cpu_ranking = Vec::new();
    for (i, metric) in timeline.iter().enumerate() {
        cpu_ranking.push(json!({
            "sample_id": i + 1,
            "overall_usage": metric.cpu_metrics.overall_usage_percent,
            "cores": metric.cpu_metrics.per_core_usage.len(),
            "load_average_1min": metric.cpu_metrics.load_average.0,
            "performance_rating": match metric.cpu_metrics.overall_usage_percent {
                usage if usage < 30.0 => "excellent",
                usage if usage < 70.0 => "good",
                _ => "high_load"
            }
        }));
    }
    
    // Sort CPU ranking by usage
    cpu_ranking.sort_by(|a, b| {
        b["overall_usage"].as_f64().unwrap_or(0.0)
            .partial_cmp(&a["overall_usage"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // GPU usage ranking
    let mut gpu_ranking = Vec::new();
    for (i, metric) in timeline.iter().enumerate() {
        if let Some(gpu) = &metric.gpu_metrics {
            gpu_ranking.push(json!({
                "sample_id": i + 1,
                "compute_usage": gpu.compute_usage_percent,
                "memory_usage": gpu.memory_usage_percent,
                "temperature": gpu.temperature_celsius,
                "power_usage": gpu.power_usage_watts,
                "device_name": gpu.device_name,
                "efficiency_rating": match gpu.compute_usage_percent {
                    usage if usage > 80.0 => "high_utilization",
                    usage if usage > 40.0 => "moderate_utilization",
                    usage if usage > 10.0 => "low_utilization",
                    _ => "idle"
                }
            }));
        }
    }
    
    // Memory usage ranking by thread
    let mut memory_ranking = Vec::new();
    for (thread_id, stats) in &analysis.memory_analysis.thread_stats {
        let efficiency = if stats.total_allocations > 0 {
            stats.total_deallocations as f64 / stats.total_allocations as f64 * 100.0
        } else { 0.0 };
        
        memory_ranking.push(json!({
            "thread_id": thread_id,
            "total_allocations": stats.total_allocations,
            "total_deallocations": stats.total_deallocations,
            "peak_memory_usage_kb": stats.peak_memory as f64 / 1024.0,
            "allocation_efficiency": efficiency,
            "efficiency_rating": match efficiency {
                eff if eff >= 90.0 => "excellent",
                eff if eff >= 70.0 => "good",
                eff if eff >= 50.0 => "fair",
                _ => "poor"
            }
        }));
    }
    
    // Sort memory ranking by peak usage
    memory_ranking.sort_by(|a, b| {
        b["peak_memory_usage_kb"].as_f64().unwrap_or(0.0)
            .partial_cmp(&a["peak_memory_usage_kb"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    
    Ok(json!({
        "cpu_usage_ranking": cpu_ranking,
        "gpu_usage_ranking": gpu_ranking,
        "memory_usage_ranking": memory_ranking,
        "thread_performance_ranking": analysis.performance_insights.thread_performance_ranking
    }))
}

/// Compute detailed resource rankings with additional analytics
fn compute_detailed_resource_rankings(analysis: &ComprehensiveAnalysis) -> Result<Value, Box<dyn std::error::Error>> {
    let basic_rankings = compute_resource_rankings(analysis)?;
    
    // Add top performers analysis
    let cpu_top_performers = basic_rankings["cpu_usage_ranking"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .take(5)
        .cloned()
        .collect::<Vec<Value>>();
    
    let memory_top_consumers = basic_rankings["memory_usage_ranking"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .take(10)
        .cloned()
        .collect::<Vec<Value>>();
    
    // Performance insights summary
    let insights_summary = json!({
        "primary_bottleneck": analysis.performance_insights.primary_bottleneck,
        "efficiency_scores": {
            "cpu": analysis.performance_insights.cpu_efficiency_score,
            "memory": analysis.performance_insights.memory_efficiency_score,
            "io": analysis.performance_insights.io_efficiency_score
        },
        "recommendations_count": analysis.performance_insights.recommendations.len(),
        "top_recommendations": analysis.performance_insights.recommendations.iter().take(3).collect::<Vec<_>>()
    });
    
    Ok(json!({
        "cpu_rankings": {
            "all_samples": basic_rankings["cpu_usage_ranking"],
            "top_performers": cpu_top_performers,
            "summary": {
                "total_samples": analysis.resource_timeline.len(),
                "average_usage": if !analysis.resource_timeline.is_empty() {
                    analysis.resource_timeline.iter()
                        .map(|r| r.cpu_metrics.overall_usage_percent)
                        .sum::<f32>() / analysis.resource_timeline.len() as f32
                } else { 0.0 }
            }
        },
        "gpu_rankings": {
            "all_samples": basic_rankings["gpu_usage_ranking"],
            "summary": {
                "gpu_available": !basic_rankings["gpu_usage_ranking"].as_array().unwrap_or(&Vec::new()).is_empty(),
                "total_gpu_samples": basic_rankings["gpu_usage_ranking"].as_array().unwrap_or(&Vec::new()).len()
            }
        },
        "memory_rankings": {
            "all_threads": basic_rankings["memory_usage_ranking"],
            "top_consumers": memory_top_consumers,
            "summary": {
                "total_threads": analysis.memory_analysis.thread_stats.len(),
                "total_allocations": analysis.memory_analysis.summary.total_allocations,
                "peak_memory_mb": analysis.memory_analysis.summary.peak_memory_usage as f64 / 1024.0 / 1024.0
            }
        },
        "thread_rankings": {
            "performance_ranking": basic_rankings["thread_performance_ranking"],
            "summary": {
                "total_ranked_threads": analysis.performance_insights.thread_performance_ranking.len()
            }
        },
        "performance_insights": insights_summary
    }))
}

#[cfg(test)]
mod tests {
    // Test module for comprehensive export functionality
    
    #[test]
    fn test_json_export_structure() {
        // This test would require a real ComprehensiveAnalysis instance
        // For now, just test that the export directory creation works
        let temp_dir = std::env::temp_dir().join("memscope_export_test");
        std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");
        
        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}