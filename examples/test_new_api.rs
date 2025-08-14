use memscope_rs::export::binary::{
    export_binary_to_dashboard, DashboardOptions, DashboardFormat, DataScope, PerformanceMode
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试新的统一Dashboard API");
    
    // 使用一个较小的测试文件
    let binary_file = "MemoryAnalysis/large_scale_user.memscope";
    let project_name = "api_test";
    
    if !std::path::Path::new(binary_file).exists() {
        println!("❌ 二进制文件不存在: {}", binary_file);
        return Ok(());
    }
    
    println!("✅ 找到二进制文件: {} ({} KB)", 
        binary_file, 
        std::fs::metadata(binary_file)?.len() / 1024
    );
    
    // 测试1: 默认轻量级导出
    println!("\n📊 测试1: 默认轻量级导出");
    let start = std::time::Instant::now();
    let stats1 = export_binary_to_dashboard(
        binary_file,
        &format!("{}_default", project_name),
        DashboardOptions::default()
    )?;
    let duration1 = start.elapsed();
    
    println!("   ✅ 生成文件数: {}", stats1.total_files_generated);
    println!("   ✅ HTML大小: {} KB", stats1.html_size / 1024);
    println!("   ✅ JSON总大小: {} KB", stats1.total_json_size / 1024);
    println!("   ✅ 处理时间: {}ms", stats1.processing_time_ms);
    println!("   ✅ 使用格式: {:?}", stats1.format_used);
    println!("   ✅ 数据范围: {:?}", stats1.scope_used);
    println!("   ⏱️  实际耗时: {}ms", duration1.as_millis());
    
    // 测试2: 快速预设
    println!("\n🚀 测试2: 快速预设");
    let start = std::time::Instant::now();
    let stats2 = export_binary_to_dashboard(
        binary_file,
        &format!("{}_fast", project_name),
        DashboardOptions::fast_preset()
    )?;
    let duration2 = start.elapsed();
    
    println!("   ✅ 生成文件数: {}", stats2.total_files_generated);
    println!("   ✅ HTML大小: {} KB", stats2.html_size / 1024);
    println!("   ✅ JSON总大小: {} KB", stats2.total_json_size / 1024);
    println!("   ✅ 处理时间: {}ms", stats2.processing_time_ms);
    println!("   ✅ 使用格式: {:?}", stats2.format_used);
    println!("   ✅ 数据范围: {:?}", stats2.scope_used);
    println!("   ⏱️  实际耗时: {}ms", duration2.as_millis());
    
    // 测试3: 嵌入式格式（向后兼容）
    println!("\n📦 测试3: 嵌入式格式（向后兼容）");
    let start = std::time::Instant::now();
    let stats3 = export_binary_to_dashboard(
        binary_file,
        &format!("{}_embedded", project_name),
        DashboardOptions::embedded_preset()
    )?;
    let duration3 = start.elapsed();
    
    println!("   ✅ 生成文件数: {}", stats3.total_files_generated);
    println!("   ✅ HTML大小: {} KB", stats3.html_size / 1024);
    println!("   ✅ JSON总大小: {} KB", stats3.total_json_size / 1024);
    println!("   ✅ 处理时间: {}ms", stats3.processing_time_ms);
    println!("   ✅ 使用格式: {:?}", stats3.format_used);
    println!("   ✅ 数据范围: {:?}", stats3.scope_used);
    println!("   ⏱️  实际耗时: {}ms", duration3.as_millis());
    
    // 测试4: 自定义配置
    println!("\n⚙️  测试4: 自定义配置");
    let custom_options = DashboardOptions::new()
        .format(DashboardFormat::Lightweight)
        .scope(DataScope::UserOnly)
        .performance(PerformanceMode::Fast);
        
    let start = std::time::Instant::now();
    let stats4 = export_binary_to_dashboard(
        binary_file,
        &format!("{}_custom", project_name),
        custom_options
    )?;
    let duration4 = start.elapsed();
    
    println!("   ✅ 生成文件数: {}", stats4.total_files_generated);
    println!("   ✅ HTML大小: {} KB", stats4.html_size / 1024);
    println!("   ✅ JSON总大小: {} KB", stats4.total_json_size / 1024);
    println!("   ✅ 处理时间: {}ms", stats4.processing_time_ms);
    println!("   ✅ 使用格式: {:?}", stats4.format_used);
    println!("   ✅ 数据范围: {:?}", stats4.scope_used);
    println!("   ⏱️  实际耗时: {}ms", duration4.as_millis());
    
    // 性能对比
    println!("\n📈 性能对比:");
    println!("   默认模式:  {}ms (实际: {}ms)", stats1.processing_time_ms, duration1.as_millis());
    println!("   快速模式:  {}ms (实际: {}ms)", stats2.processing_time_ms, duration2.as_millis());
    println!("   嵌入模式:  {}ms (实际: {}ms)", stats3.processing_time_ms, duration3.as_millis());
    println!("   自定义:    {}ms (实际: {}ms)", stats4.processing_time_ms, duration4.as_millis());
    
    // 文件大小对比
    println!("\n📏 文件大小对比:");
    println!("   默认模式:  {} KB", stats1.html_size / 1024);
    println!("   快速模式:  {} KB", stats2.html_size / 1024);
    println!("   嵌入模式:  {} KB", stats3.html_size / 1024);
    println!("   自定义:    {} KB", stats4.html_size / 1024);
    
    println!("\n🎉 所有测试完成！");
    println!("\n📁 生成的文件:");
    println!("   MemoryAnalysis/{}_default/", project_name);
    println!("   MemoryAnalysis/{}_fast/", project_name);
    println!("   MemoryAnalysis/{}_embedded/", project_name);
    println!("   MemoryAnalysis/{}_custom/", project_name);
    
    Ok(())
}