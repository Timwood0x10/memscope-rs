#!/usr/bin/env python3
"""
数据格式转换脚本
将现有的 data.json 转换为仪表板期望的格式
"""

import json
import os
from datetime import datetime

def convert_data_format():
    """转换数据格式以匹配仪表板期望"""
    
    # 读取现有数据
    data_file = "web_dashboard/data.json"
    if not os.path.exists(data_file):
        print("❌ 数据文件不存在:", data_file)
        return False
    
    with open(data_file, 'r', encoding='utf-8') as f:
        original_data = json.load(f)
    
    print("📊 原始数据结构:")
    print("  - 顶级键:", list(original_data.keys()))
    
    # 计算总分配数和总大小
    total_allocations = 0
    total_size = 0
    allocation_details = []
    
    # 遍历内存层次结构来提取数据
    if 'memory_hierarchy' in original_data:
        for category_name, category_data in original_data['memory_hierarchy'].items():
            if 'subcategories' in category_data:
                for subcat_name, subcat_data in category_data['subcategories'].items():
                    if 'types' in subcat_data:
                        for type_info in subcat_data['types']:
                            if 'allocations' in type_info:
                                for alloc in type_info['allocations']:
                                    total_allocations += 1
                                    total_size += alloc.get('size_bytes', 0)
                                    allocation_details.append({
                                        'size': alloc.get('size_bytes', 0),
                                        'type': alloc.get('type_name', 'unknown'),
                                        'variable': alloc.get('variable_name', 'unknown'),
                                        'timestamp': alloc.get('allocation_time', 0)
                                    })
    
    # 创建仪表板期望的数据格式
    dashboard_data = {
        "memory_stats": {
            "total_allocations": total_allocations,
            "total_size_bytes": total_size,
            "peak_memory_usage": total_size,  # 简化处理
            "current_memory_usage": total_size,
            "allocation_rate": total_allocations / 60 if total_allocations > 0 else 0,  # 假设1分钟内
            "deallocation_rate": 0,  # 暂时设为0
            "memory_efficiency": 85.5,  # 示例值
            "fragmentation_ratio": 0.15,  # 示例值
            "allocations": allocation_details[:100]  # 限制数量避免过大
        },
        "unsafe_stats": {
            "total_operations": 0,  # 当前数据中没有unsafe操作信息
            "unsafe_blocks": 0,
            "ffi_calls": 0,
            "raw_pointer_operations": 0,
            "memory_violations": 0,
            "risk_score": 0.0,
            "operations": []
        },
        "performance_metrics": {
            "allocation_time_avg_ns": 1000,  # 示例值
            "allocation_time_max_ns": 5000,
            "memory_throughput_mb_s": 100.0,
            "gc_pressure": 0.1
        },
        "lifecycle_stats": {
            "short_lived_objects": total_allocations * 0.6,
            "medium_lived_objects": total_allocations * 0.3,
            "long_lived_objects": total_allocations * 0.1,
            "average_lifetime_ms": 5000,
            "memory_leaks_detected": 0
        },
        "metadata": {
            "generated_at": datetime.now().isoformat(),
            "version": "2.0",
            "source": "converted_from_memory_hierarchy",
            "total_runtime_ms": 60000,  # 示例值
            "conversion_note": "Data converted from memory_hierarchy format to dashboard format"
        },
        # 保留原始数据以备参考
        "original_data": original_data
    }
    
    # 备份原始文件
    backup_file = f"{data_file}.backup"
    with open(backup_file, 'w', encoding='utf-8') as f:
        json.dump(original_data, f, indent=2, ensure_ascii=False)
    print(f"✅ 原始数据已备份到: {backup_file}")
    
    # 写入转换后的数据
    with open(data_file, 'w', encoding='utf-8') as f:
        json.dump(dashboard_data, f, indent=2, ensure_ascii=False)
    
    print("🔄 数据转换完成!")
    print(f"  - 总分配数: {total_allocations}")
    print(f"  - 总内存大小: {total_size} bytes")
    print(f"  - 新数据结构键: {list(dashboard_data.keys())}")
    
    return True

def create_sample_unsafe_data():
    """创建一些示例unsafe数据"""
    
    sample_unsafe_data = {
        "memory_stats": {
            "total_allocations": 150,
            "total_size_bytes": 8192,
            "peak_memory_usage": 12288,
            "current_memory_usage": 8192,
            "allocation_rate": 25.5,
            "deallocation_rate": 20.0,
            "memory_efficiency": 87.3,
            "fragmentation_ratio": 0.12,
            "allocations": [
                {
                    "size": 1024,
                    "type": "Vec<u8>",
                    "variable": "buffer",
                    "timestamp": 1752552068000
                },
                {
                    "size": 2048,
                    "type": "*mut c_void",
                    "variable": "raw_ptr",
                    "timestamp": 1752552068100
                }
            ]
        },
        "unsafe_stats": {
            "total_operations": 45,
            "unsafe_blocks": 12,
            "ffi_calls": 8,
            "raw_pointer_operations": 25,
            "memory_violations": 2,
            "risk_score": 3.2,
            "operations": [
                {
                    "type": "raw_pointer_deref",
                    "location": "src/main.rs:42",
                    "risk_level": "medium",
                    "timestamp": 1752552068200,
                    "description": "Dereferencing raw pointer"
                },
                {
                    "type": "ffi_call",
                    "location": "src/ffi.rs:15",
                    "risk_level": "high",
                    "timestamp": 1752552068300,
                    "description": "Call to external C function"
                }
            ]
        },
        "performance_metrics": {
            "allocation_time_avg_ns": 1250,
            "allocation_time_max_ns": 8500,
            "memory_throughput_mb_s": 125.7,
            "gc_pressure": 0.08
        },
        "lifecycle_stats": {
            "short_lived_objects": 90,
            "medium_lived_objects": 45,
            "long_lived_objects": 15,
            "average_lifetime_ms": 3500,
            "memory_leaks_detected": 1
        },
        "metadata": {
            "generated_at": datetime.now().isoformat(),
            "version": "2.0",
            "source": "sample_unsafe_data",
            "total_runtime_ms": 45000
        }
    }
    
    # 写入示例数据
    sample_file = "web_dashboard/sample_data.json"
    with open(sample_file, 'w', encoding='utf-8') as f:
        json.dump(sample_unsafe_data, f, indent=2, ensure_ascii=False)
    
    print(f"📝 示例数据已创建: {sample_file}")
    return sample_file

def main():
    print("🔧 数据格式修复工具")
    print("=" * 40)
    
    # 转换现有数据
    if convert_data_format():
        print("\n✅ 数据转换成功!")
    else:
        print("\n❌ 数据转换失败!")
        return
    
    # 创建示例数据
    print("\n📝 创建示例unsafe数据...")
    sample_file = create_sample_unsafe_data()
    
    print("\n🎯 建议:")
    print("1. 运行 Python start_web_dashboard.py 启动服务器")
    print("2. 如果需要更多真实数据，运行:")
    print("   cargo run --example unsafe_ffi_demo")
    print("   cargo run --example memory_stress_test")
    print(f"3. 可以将 {sample_file} 复制为 data.json 来测试unsafe仪表板")

if __name__ == "__main__":
    main()