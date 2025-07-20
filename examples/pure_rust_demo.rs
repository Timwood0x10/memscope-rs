// 纯净的 Rust 代码示例 - 没有任何 memscope 侵入式代码
// 这个程序模拟真实的业务逻辑，包含各种内存分配和释放

use std::collections::HashMap;

fn main() {
    println!("🚀 Starting pure Rust application...");
    
    // 创建一些基本变量
    let numbers = vec![1, 2, 3, 4, 5];
    let message = String::from("Hello, World!");
    let boxed_number = Box::new(42);
    
    println!("📦 Created basic variables: vec, string, box");
    
    // 调用一些函数，创建和释放内存
    process_data();
    
    // 创建更复杂的数据结构
    let mut map = HashMap::new();
    map.insert("key1".to_string(), vec![1, 2, 3]);
    map.insert("key2".to_string(), vec![4, 5, 6]);
    map.insert("key3".to_string(), vec![7, 8, 9]);
    
    println!("📊 Created HashMap with {} entries", map.len());
    
    // 调用处理 HashMap 的函数
    let result = process_hashmap(map);
    println!("🔢 HashMap processing result: {}", result);
    
    // 创建一些大的分配
    let large_buffer = create_large_buffer(1000);
    println!("💾 Created large buffer with {} bytes", large_buffer.len());
    
    // 进行一些字符串操作
    let processed_strings = string_operations();
    println!("📝 Processed {} strings", processed_strings.len());
    
    // 创建嵌套结构
    let nested_data = create_nested_structure();
    println!("🏗️ Created nested structure with {} items", nested_data.len());
    
    // 模拟一些计算密集型操作
    let computation_result = heavy_computation(&numbers);
    println!("⚡ Computation result: {}", computation_result);
    
    println!("✅ Pure Rust application completed successfully!");
}

fn process_data() {
    // 这个函数会创建一些临时变量，然后释放它们
    let temp_vec = vec![10, 20, 30, 40, 50];
    let temp_string = "Temporary data".repeat(10);
    let temp_boxes: Vec<Box<i32>> = (0..5).map(|i| Box::new(i * 10)).collect();
    
    println!("🔄 Processing temporary data...");
    
    // 进行一些操作
    let sum: i32 = temp_vec.iter().sum();
    let length = temp_string.len();
    let box_sum: i32 = temp_boxes.iter().map(|b| **b).sum();
    
    println!("📈 Temp processing: sum={}, length={}, box_sum={}", sum, length, box_sum);
    
    // 函数结束时，所有临时变量都会被释放
}

fn process_hashmap(mut map: HashMap<String, Vec<i32>>) -> usize {
    // 对 HashMap 进行一些操作
    map.insert("key4".to_string(), vec![10, 11, 12]);
    map.insert("key5".to_string(), vec![13, 14, 15]);
    
    // 创建一些临时的处理数据
    let mut total_elements = 0;
    for (key, values) in &map {
        total_elements += values.len();
        
        // 创建临时字符串
        let temp_description = format!("Key {} has {} elements", key, values.len());
        println!("📋 {}", temp_description);
    }
    
    // 创建一个新的 HashMap 进行转换
    let transformed: HashMap<String, usize> = map
        .into_iter()
        .map(|(k, v)| (k, v.len()))
        .collect();
    
    transformed.len()
}

fn create_large_buffer(size: usize) -> Vec<u8> {
    // 创建一个大的缓冲区
    let mut buffer = Vec::with_capacity(size);
    
    for i in 0..size {
        buffer.push((i % 256) as u8);
    }
    
    // 创建一些临时的处理缓冲区
    let temp_buffer1 = vec![0u8; size / 2];
    let temp_buffer2 = vec![255u8; size / 4];
    
    println!("🔧 Created temporary buffers: {} and {} bytes", 
             temp_buffer1.len(), temp_buffer2.len());
    
    buffer
}

fn string_operations() -> Vec<String> {
    let mut results = Vec::new();
    
    // 创建各种字符串
    for i in 0..10 {
        let base_string = format!("String number {}", i);
        let repeated = base_string.repeat(3);
        let uppercase = repeated.to_uppercase();
        
        // 创建一些临时字符串操作
        let temp1 = format!("Temp: {}", uppercase);
        let temp2 = temp1.replace("STRING", "string");
        
        results.push(temp2);
    }
    
    // 进行一些字符串连接操作
    let concatenated = results.join(" | ");
    results.push(concatenated);
    
    results
}

fn create_nested_structure() -> Vec<Vec<HashMap<String, Box<i32>>>> {
    let mut nested = Vec::new();
    
    for i in 0..3 {
        let mut inner_vec = Vec::new();
        
        for j in 0..2 {
            let mut inner_map = HashMap::new();
            
            for k in 0..3 {
                let key = format!("item_{}_{}", j, k);
                let value = Box::new(i * 10 + j * 5 + k);
                inner_map.insert(key, value);
            }
            
            inner_vec.push(inner_map);
        }
        
        nested.push(inner_vec);
    }
    
    nested
}

fn heavy_computation(input: &[i32]) -> i64 {
    // 模拟一些计算密集型操作，会创建临时数据
    let mut temp_results = Vec::new();
    
    for &num in input {
        // 创建临时计算数据
        let temp_vec: Vec<i64> = (0..num).map(|i| (i as i64) * (num as i64)).collect();
        let temp_sum: i64 = temp_vec.iter().sum();
        
        // 创建更多临时数据
        let temp_string = format!("Computing for {}", num);
        let temp_bytes = temp_string.into_bytes();
        let temp_checksum: u64 = temp_bytes.iter().map(|&b| b as u64).sum();
        
        temp_results.push(temp_sum + temp_checksum as i64);
    }
    
    temp_results.iter().sum()
}