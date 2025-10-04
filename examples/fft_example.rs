//! Enhanced Multi-threaded FFT and Elliptic Curve computation example
//! Uses memscope-rs lockfree API with proper data collection

use memscope_rs::lockfree::{
    export_comprehensive_analysis, finalize_thread_tracker, init_thread_tracker,
    track_allocation_lockfree, IntegratedProfilingSession,
};
use rayon::prelude::*;
use std::f64::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

static ALLOCATION_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy)]
struct Complex {
    real: f64,
    imag: f64,
}

impl Complex {
    fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }
    
    fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imag * self.imag).sqrt()
    }
    
    fn phase(&self) -> f64 {
        self.imag.atan2(self.real)
    }
}

impl std::ops::Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.real + other.real, self.imag + other.imag)
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.real - other.real, self.imag - other.imag)
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.real * other.real - self.imag * other.imag,
            self.real * other.imag + self.imag * other.real,
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct EllipticPoint {
    x: f64,
    y: f64,
    infinity: bool,
}

impl EllipticPoint {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y, infinity: false }
    }
    
    fn infinity() -> Self {
        Self { x: 0.0, y: 0.0, infinity: true }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Enhanced Multi-threaded FFT and Elliptic Curve Analysis v8");
    println!("============================================================");
    
    let output_dir = "./memoryanalysis";
    std::fs::create_dir_all(output_dir)?;
    
    let start_time = Instant::now();
    
    // Initialize lockfree profiling session
    let mut session = IntegratedProfilingSession::new(std::path::Path::new(output_dir))?;
    session.start_profiling()?;
    println!("📊 Started enhanced lockfree memory tracking...");
    
    // Initialize global thread tracker
    let _ = init_thread_tracker(std::path::Path::new(output_dir), None);
    
    // Phase 1: Aggressive multi-threaded FFT with heavy memory allocation
    println!("\n🔸 Phase 1: Aggressive Multi-threaded FFT Analysis");
    aggressive_fft_workload()?;
    
    // Phase 2: Memory-intensive elliptic curve operations
    println!("\n🔸 Phase 2: Memory-intensive Elliptic Curve Operations");
    memory_intensive_ecc_workload()?;
    
    // Phase 3: Concurrent mixed mathematical workload
    println!("\n🔸 Phase 3: Concurrent Mixed Mathematical Workload");
    concurrent_mixed_workload()?;
    
    // Phase 4: Stress test with maximum allocations
    println!("\n🔸 Phase 4: Memory Allocation Stress Test");
    memory_allocation_stress_test()?;
    
    let elapsed = start_time.elapsed();
    let total_allocations = ALLOCATION_COUNTER.load(Ordering::Relaxed);
    
    // Finalize tracking and generate comprehensive reports
    let _ = finalize_thread_tracker();
    let analysis = session.stop_profiling_and_analyze()?;
    export_comprehensive_analysis(&analysis, std::path::Path::new(output_dir), "enhanced_fft_ecc")?;
    
    println!("\n✅ Enhanced multi-threaded mathematical analysis complete!");
    println!("⏱️  Total execution time: {:.2}s", elapsed.as_secs_f64());
    println!("📊 Total tracked allocations: {}", total_allocations);
    println!("📁 Enhanced analysis exported to:");
    println!("   📄 JSON: {}/enhanced_fft_ecc_comprehensive.json", output_dir);
    println!("   🌐 HTML: {}/enhanced_fft_ecc_dashboard.html", output_dir);
    println!("   📈 Rankings: {}/enhanced_fft_ecc_resource_rankings.json", output_dir);
    
    Ok(())
}

fn aggressive_fft_workload() -> Result<(), Box<dyn std::error::Error>> {
    // Create larger FFT operations with more threads
    let fft_configs = vec![
        (2048, 12, "Small_FFT"),     // 2K points, 12 threads
        (4096, 16, "Medium_FFT"),    // 4K points, 16 threads
        (8192, 20, "Large_FFT"),     // 8K points, 20 threads
        (16384, 24, "XLarge_FFT"),   // 16K points, 24 threads
        (32768, 28, "XXLarge_FFT"),  // 32K points, 28 threads
    ];
    
    for (fft_size, thread_count, label) in fft_configs {
        println!("   🧮 {}: {} points, {} threads", label, fft_size, thread_count);
        
        // Create custom thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build()?;
        
        pool.install(|| {
            // Multiple signal processing operations per configuration
            for iteration in 0..3 {
                // Initialize tracker in this thread
                let _ = init_thread_tracker(std::path::Path::new("./memoryanalysis"), None);
                
                // Generate multiple test signals
                let signal1 = generate_complex_signal(fft_size, iteration);
                track_complex_allocation(&signal1, &format!("{}_signal1_iter{}", label, iteration));
                
                let signal2 = generate_complex_signal(fft_size, iteration + 100);
                track_complex_allocation(&signal2, &format!("{}_signal2_iter{}", label, iteration));
                
                // Perform FFT operations
                let fft_result1 = parallel_fft(&signal1);
                track_complex_allocation(&fft_result1, &format!("{}_fft1_iter{}", label, iteration));
                
                let fft_result2 = parallel_fft(&signal2);
                track_complex_allocation(&fft_result2, &format!("{}_fft2_iter{}", label, iteration));
                
                // Cross-correlation between signals
                let correlation = parallel_cross_correlation(&signal1, &signal2);
                track_complex_allocation(&correlation, &format!("{}_correlation_iter{}", label, iteration));
                
                // IFFT for verification
                let ifft_result = parallel_ifft(&fft_result1);
                track_complex_allocation(&ifft_result, &format!("{}_ifft_iter{}", label, iteration));
                
                // Spectral analysis operations
                let spectrum: Vec<f64> = fft_result1.iter().map(|c| c.magnitude()).collect();
                track_f64_allocation(&spectrum, &format!("{}_spectrum_iter{}", label, iteration));
                
                // Phase analysis
                let phases: Vec<f64> = fft_result1.iter().map(|c| c.phase()).collect();
                track_f64_allocation(&phases, &format!("{}_phases_iter{}", label, iteration));
                
                println!("     ⚡ Iteration {} completed - {} allocations tracked", 
                         iteration + 1, ALLOCATION_COUNTER.load(Ordering::Relaxed));
            }
        });
        
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    Ok(())
}

fn memory_intensive_ecc_workload() -> Result<(), Box<dyn std::error::Error>> {
    println!("   🔐 Memory-intensive elliptic curve computations");
    
    // Elliptic curve parameters
    let a = 0.0;
    let b = 7.0;
    let p = 2_f64.powi(32) - 977.0; // Simplified modulus
    
    let base_point = EllipticPoint::new(
        12345.67890, 
        67890.12345,
    );
    
    // Generate large sets of scalars for parallel computation
    let scalar_sets: Vec<Vec<u64>> = (0..8).map(|set_id| {
        (0..50).map(|i| ((set_id + 1) * 10000 + i * 123) as u64).collect()
    }).collect();
    
    // Process each set in parallel
    scalar_sets.into_par_iter().enumerate().for_each(|(set_id, scalars)| {
        // Initialize tracker for this thread
        let _ = init_thread_tracker(std::path::Path::new("./memoryanalysis"), None);
        
        // Allocate working memory for this set
        let mut ec_points: Vec<EllipticPoint> = Vec::with_capacity(scalars.len());
        let mut signatures: Vec<(u64, u64)> = Vec::with_capacity(scalars.len());
        let mut public_keys: Vec<EllipticPoint> = Vec::with_capacity(scalars.len());
        
        for (i, &scalar) in scalars.iter().enumerate() {
            // Scalar multiplication
            let point = scalar_multiply(&base_point, scalar, a, b, p);
            ec_points.push(point);
            
            // Generate public key
            let pub_key = scalar_multiply(&base_point, scalar, a, b, p);
            public_keys.push(pub_key);
            
            // Create signature (simplified)
            let message_hash = (scalar + i as u64) % (1u64 << 32);
            let k = (scalar * 2 + 0x123456789ABCDEF0) % (1u64 << 32);
            let r = (point.x as u64) % (1u64 << 32);
            let s = (message_hash + r * scalar) % (1u64 << 32);
            signatures.push((r, s));
        }
        
        // Track allocations for this thread's data
        track_ec_points_allocation(&ec_points, &format!("ecc_points_set_{}", set_id));
        track_ec_points_allocation(&public_keys, &format!("public_keys_set_{}", set_id));
        track_signatures_allocation(&signatures, &format!("signatures_set_{}", set_id));
        
        // Additional computation: key derivation chains
        let mut derived_keys: Vec<EllipticPoint> = Vec::new();
        for i in 0..20 {
            let derived_scalar = scalars[i % scalars.len()] + (i as u64 * 1000);
            let derived_key = scalar_multiply(&base_point, derived_scalar, a, b, p);
            derived_keys.push(derived_key);
        }
        track_ec_points_allocation(&derived_keys, &format!("derived_keys_set_{}", set_id));
        
        println!("     🔑 Set {} completed: {} EC operations", set_id, scalars.len() + 20);
    });
    
    Ok(())
}

fn concurrent_mixed_workload() -> Result<(), Box<dyn std::error::Error>> {
    println!("   🌟 Concurrent mixed FFT + ECC workload");
    
    let workload_configs = vec![
        ("Mixed_Small", 1024, 500, 8),
        ("Mixed_Medium", 2048, 1000, 12),
        ("Mixed_Large", 4096, 2000, 16),
        ("Mixed_XLarge", 8192, 4000, 20),
    ];
    
    workload_configs.into_par_iter().for_each(|(name, fft_size, ec_scalar_base, _thread_count)| {
        let _ = init_thread_tracker(std::path::Path::new("./memoryanalysis"), None);
        
        // FFT workload
        for i in 0..5 {
            let signal = generate_complex_signal(fft_size, i);
            track_complex_allocation(&signal, &format!("{}_mixed_signal_{}", name, i));
            
            let fft_result = parallel_fft(&signal);
            track_complex_allocation(&fft_result, &format!("{}_mixed_fft_{}", name, i));
            
            // Create frequency domain modifications
            let mut modified_spectrum = fft_result.clone();
            for (j, sample) in modified_spectrum.iter_mut().enumerate() {
                if j % 10 == 0 {
                    *sample = *sample * Complex::new(0.5, 0.0);
                }
            }
            track_complex_allocation(&modified_spectrum, &format!("{}_modified_spectrum_{}", name, i));
        }
        
        // ECC workload  
        let base_point = EllipticPoint::new(12345.0, 67890.0);
        let mut ecc_results: Vec<EllipticPoint> = Vec::new();
        
        for i in 0..30 {
            let scalar = ec_scalar_base + (i * 111) as u64;
            let point = scalar_multiply(&base_point, scalar, 0.0, 7.0, 2_f64.powi(32) - 977.0);
            ecc_results.push(point);
        }
        track_ec_points_allocation(&ecc_results, &format!("{}_ecc_results", name));
        
        // Additional matrix operations
        let matrix_size = 100;
        let mut matrix: Vec<Vec<f64>> = vec![vec![0.0; matrix_size]; matrix_size];
        for i in 0..matrix_size {
            for j in 0..matrix_size {
                matrix[i][j] = (i + j) as f64 * 0.01;
            }
        }
        track_matrix_allocation(&matrix, &format!("{}_computation_matrix", name));
        
        println!("     ⚡ {} workload completed", name);
    });
    
    Ok(())
}

fn memory_allocation_stress_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("   💾 Memory allocation stress test");
    
    // Create stress test with many small and large allocations
    let stress_configs = vec![
        (1000, 512, "SmallAlloc"),    // 1000 allocations of 512 elements each
        (500, 2048, "MediumAlloc"),   // 500 allocations of 2048 elements each
        (200, 8192, "LargeAlloc"),    // 200 allocations of 8192 elements each
        (50, 32768, "XLargeAlloc"),   // 50 allocations of 32768 elements each
    ];
    
    stress_configs.into_par_iter().for_each(|(count, size, label)| {
        let _ = init_thread_tracker(std::path::Path::new("./memoryanalysis"), None);
        
        for batch in 0..5 {
            let mut allocations: Vec<Vec<Complex>> = Vec::new();
            
            for i in 0..count / 5 {  // Divide by 5 batches
                let data: Vec<Complex> = (0..size).map(|j| {
                    let val = (i + j + batch * 1000) as f64 * 0.001;
                    Complex::new(val.sin(), val.cos())
                }).collect();
                
                track_complex_allocation(&data, &format!("{}_batch{}_alloc{}", label, batch, i));
                allocations.push(data);
            }
            
            // Additional processing to create more allocations
            let processed: Vec<Vec<f64>> = allocations.iter().map(|vec| {
                vec.iter().map(|c| c.magnitude()).collect()
            }).collect();
            
            for (i, proc_data) in processed.iter().enumerate() {
                track_f64_allocation(proc_data, &format!("{}_batch{}_processed{}", label, batch, i));
            }
        }
        
        println!("     💾 {} stress test completed", label);
    });
    
    Ok(())
}

// Enhanced signal generation with more complexity
fn generate_complex_signal(size: usize, seed: usize) -> Vec<Complex> {
    (0..size)
        .map(|i| {
            let t = i as f64 / size as f64;
            let phase_offset = seed as f64 * 0.1;
            
            // Multi-component signal with harmonics and modulation
            let fundamental = (2.0 * PI * 50.0 * t + phase_offset).sin();
            let second_harmonic = 0.7 * (2.0 * PI * 100.0 * t + phase_offset).sin();
            let third_harmonic = 0.5 * (2.0 * PI * 150.0 * t + phase_offset).sin();
            let modulation = 0.3 * (2.0 * PI * 5.0 * t).sin();
            let noise = 0.1 * (rand::random::<f64>() - 0.5);
            
            let signal = fundamental + second_harmonic + third_harmonic;
            let modulated_signal = signal * (1.0 + modulation) + noise;
            
            Complex::new(modulated_signal, 0.0)
        })
        .collect()
}

// Enhanced tracking functions with proper error handling
fn track_complex_allocation(data: &[Complex], name: &str) {
    let ptr = data.as_ptr() as usize;
    let size_bytes = data.len() * std::mem::size_of::<Complex>();
    let call_stack = vec![
        track_complex_allocation as *const () as usize,
        0xDEADBEEF, // Placeholder for caller
    ];
    
    match track_allocation_lockfree(ptr, size_bytes, &call_stack) {
        Ok(_) => {
            ALLOCATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        }
        Err(e) => {
            eprintln!("Warning: Failed to track {} allocation: {}", name, e);
        }
    }
}

fn track_f64_allocation(data: &[f64], name: &str) {
    let ptr = data.as_ptr() as usize;
    let size_bytes = data.len() * std::mem::size_of::<f64>();
    let call_stack = vec![
        track_f64_allocation as *const () as usize,
        0xCAFEBABE,
    ];
    
    match track_allocation_lockfree(ptr, size_bytes, &call_stack) {
        Ok(_) => {
            ALLOCATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        }
        Err(e) => {
            eprintln!("Warning: Failed to track {} allocation: {}", name, e);
        }
    }
}

fn track_ec_points_allocation(data: &[EllipticPoint], name: &str) {
    let ptr = data.as_ptr() as usize;
    let size_bytes = data.len() * std::mem::size_of::<EllipticPoint>();
    let call_stack = vec![
        track_ec_points_allocation as *const () as usize,
        0xFEEDBEEF,
    ];
    
    match track_allocation_lockfree(ptr, size_bytes, &call_stack) {
        Ok(_) => {
            ALLOCATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        }
        Err(e) => {
            eprintln!("Warning: Failed to track {} allocation: {}", name, e);
        }
    }
}

fn track_signatures_allocation(data: &[(u64, u64)], name: &str) {
    let ptr = data.as_ptr() as usize;
    let size_bytes = data.len() * std::mem::size_of::<(u64, u64)>();
    let call_stack = vec![
        track_signatures_allocation as *const () as usize,
        0xBEEFCAFE,
    ];
    
    match track_allocation_lockfree(ptr, size_bytes, &call_stack) {
        Ok(_) => {
            ALLOCATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        }
        Err(e) => {
            eprintln!("Warning: Failed to track {} allocation: {}", name, e);
        }
    }
}

fn track_matrix_allocation(data: &[Vec<f64>], name: &str) {
    let total_elements: usize = data.iter().map(|row| row.len()).sum();
    let size_bytes = total_elements * std::mem::size_of::<f64>();
    let call_stack = vec![
        track_matrix_allocation as *const () as usize,
        0xABCDEF00,
    ];
    
    match track_allocation_lockfree(data.as_ptr() as usize, size_bytes, &call_stack) {
        Ok(_) => {
            ALLOCATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        }
        Err(e) => {
            eprintln!("Warning: Failed to track {} allocation: {}", name, e);
        }
    }
}

// FFT Implementation (reusing previous implementation)
fn parallel_fft(input: &[Complex]) -> Vec<Complex> {
    let n = input.len();
    if n <= 1 {
        return input.to_vec();
    }
    
    let mut data = input.to_vec();
    
    // Bit-reversal permutation
    bit_reverse_permutation(&mut data);
    
    // Iterative FFT
    let mut stage = 1;
    while stage < n {
        butterfly_stage(&mut data, stage);
        stage *= 2;
    }
    
    data
}

fn parallel_ifft(input: &[Complex]) -> Vec<Complex> {
    let n = input.len();
    
    // Conjugate the input
    let conjugated: Vec<Complex> = input.iter()
        .map(|c| Complex::new(c.real, -c.imag))
        .collect();
    
    // Perform FFT
    let mut result = parallel_fft(&conjugated);
    
    // Conjugate and normalize
    for sample in &mut result {
        sample.real /= n as f64;
        sample.imag = -sample.imag / n as f64;
    }
    
    result
}

fn parallel_cross_correlation(signal1: &[Complex], signal2: &[Complex]) -> Vec<Complex> {
    let n = signal1.len() + signal2.len() - 1;
    let fft_size = n.next_power_of_two();
    
    // Zero-pad signals
    let mut padded1 = signal1.to_vec();
    let mut padded2 = signal2.to_vec();
    padded1.resize(fft_size, Complex::new(0.0, 0.0));
    padded2.resize(fft_size, Complex::new(0.0, 0.0));
    
    // FFT both signals
    let fft1 = parallel_fft(&padded1);
    let fft2 = parallel_fft(&padded2);
    
    // Multiply first by conjugate of second
    let product: Vec<Complex> = fft1.iter()
        .zip(fft2.iter())
        .map(|(a, b)| Complex::new(
            a.real * b.real + a.imag * b.imag,
            a.imag * b.real - a.real * b.imag,
        ))
        .collect();
    
    // IFFT to get correlation result
    let mut result = parallel_ifft(&product);
    result.truncate(n);
    result
}

fn bit_reverse_permutation(data: &mut [Complex]) {
    let n = data.len();
    for i in 0..n {
        let j = bit_reverse(i, n);
        if i < j {
            data.swap(i, j);
        }
    }
}

fn butterfly_stage(data: &mut [Complex], stage: usize) {
    let n = data.len();
    let step = stage * 2;
    
    for k in 0..n / step {
        let offset = k * step;
        let angle = -2.0 * PI * (k as f64) / (step as f64);
        let twiddle = Complex::new(angle.cos(), angle.sin());
        
        for i in 0..stage {
            let pos1 = offset + i;
            let pos2 = pos1 + stage;
            
            let a = data[pos1];
            let b = data[pos2];
            
            let temp = b * twiddle;
            data[pos1] = a + temp;
            data[pos2] = a - temp;
        }
    }
}

fn bit_reverse(mut n: usize, size: usize) -> usize {
    let mut result = 0;
    let mut power = size;
    
    while power > 1 {
        result = (result << 1) | (n & 1);
        n >>= 1;
        power >>= 1;
    }
    
    result
}

// Elliptic Curve Implementation (simplified)
fn scalar_multiply(point: &EllipticPoint, scalar: u64, a: f64, _b: f64, p: f64) -> EllipticPoint {
    if scalar == 0 || point.infinity {
        return EllipticPoint::infinity();
    }
    
    let mut result = EllipticPoint::infinity();
    let mut addend = *point;
    let mut k = scalar;
    
    while k > 0 {
        if k & 1 == 1 {
            result = point_add(&result, &addend, a, p);
        }
        addend = point_double(&addend, a, p);
        k >>= 1;
    }
    
    result
}

fn point_add(p1: &EllipticPoint, p2: &EllipticPoint, a: f64, p: f64) -> EllipticPoint {
    if p1.infinity {
        return *p2;
    }
    if p2.infinity {
        return *p1;
    }
    
    if (p1.x - p2.x).abs() < 1e-10 {
        if (p1.y - p2.y).abs() < 1e-10 {
            return point_double(p1, a, p);
        } else {
            return EllipticPoint::infinity();
        }
    }
    
    let lambda = (p2.y - p1.y) / (p2.x - p1.x);
    let x3 = lambda * lambda - p1.x - p2.x;
    let y3 = lambda * (p1.x - x3) - p1.y;
    
    EllipticPoint::new(x3 % p, y3 % p)
}

fn point_double(point: &EllipticPoint, a: f64, p: f64) -> EllipticPoint {
    if point.infinity || point.y.abs() < 1e-10 {
        return EllipticPoint::infinity();
    }
    
    let lambda = (3.0 * point.x * point.x + a) / (2.0 * point.y);
    let x3 = lambda * lambda - 2.0 * point.x;
    let y3 = lambda * (point.x - x3) - point.y;
    
    EllipticPoint::new(x3 % p, y3 % p)
}