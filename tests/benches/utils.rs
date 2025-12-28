use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub iterations: usize,
    pub duration: Duration,
    pub throughput: f64, // operations per second
    pub memory_usage: Option<usize>, // in bytes
    pub transaction_count: u64,
    pub parameters: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}

impl BenchmarkResult {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            timestamp: chrono::Utc::now(),
            iterations: 0,
            duration: Duration::default(),
            throughput: 0.0,
            memory_usage: None,
            transaction_count: 0,
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn calculate_tps(&self) -> f64 {
        self.transaction_count as f64 / self.duration.as_secs_f64()
    }
    
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        
        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

#[derive(Default)]
pub struct BenchmarkCollector {
    results: Vec<BenchmarkResult>,
}

impl BenchmarkCollector {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
    
    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }
    
    pub fn save_all(&self, directory: &Path) -> std::io::Result<()> {
        for (i, result) in self.results.iter().enumerate() {
            let filename = format!(
                "benchmark_{}_{}.json",
                result.name.replace(' ', "_").to_lowercase(),
                i
            );
            let path = directory.join(filename);
            result.save_to_file(&path)?;
        }
        
        // Save summary
        self.save_summary(directory)
    }
    
    pub fn save_summary(&self, directory: &Path) -> std::io::Result<()> {
        let summary: Vec<HashMap<String, String>> = self.results
            .iter()
            .map(|r| {
                let mut map = HashMap::new();
                map.insert("name".to_string(), r.name.clone());
                map.insert("timestamp".to_string(), r.timestamp.to_rfc3339());
                map.insert("duration_secs".to_string(), r.duration.as_secs_f64().to_string());
                map.insert("throughput".to_string(), r.throughput.to_string());
                map.insert("transactions".to_string(), r.transaction_count.to_string());
                map.insert("tps".to_string(), r.calculate_tps().to_string());
                
                if let Some(memory) = r.memory_usage {
                    map.insert("memory_mb".to_string(), format!("{:.2}", memory as f64 / 1024.0 / 1024.0));
                }
                
                map
            })
            .collect();
        
        let json = serde_json::to_string_pretty(&summary)?;
        let path = directory.join("benchmark_summary.json");
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
}

pub struct PerformanceMetrics {
    pub start_time: Instant,
    pub operation_count: u64,
    pub memory_samples: Vec<usize>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            operation_count: 0,
            memory_samples: Vec::new(),
        }
    }
    
    pub fn record_operation(&mut self) {
        self.operation_count += 1;
    }
    
    pub fn record_memory(&mut self) {
        // This is a simplified memory measurement
        // In production, you might want to use more accurate methods
        let memory = get_current_memory_usage();
        self.memory_samples.push(memory);
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    pub fn operations_per_second(&self) -> f64 {
        self.operation_count as f64 / self.elapsed().as_secs_f64()
    }
    
    pub fn average_memory(&self) -> Option<usize> {
        if self.memory_samples.is_empty() {
            None
        } else {
            let sum: usize = self.memory_samples.iter().sum();
            Some(sum / self.memory_samples.len())
        }
    }
    
    pub fn max_memory(&self) -> Option<usize> {
        self.memory_samples.iter().max().copied()
    }
}

fn get_current_memory_usage() -> usize {
    // This is platform-specific and simplified
    // On Linux/Unix, you might read from /proc/self/statm
    // For cross-platform, consider using a crate like `sysinfo`
    
    // For now, return a dummy value
    0
}

pub struct LoadGenerator {
    pub transaction_rate: u32, // transactions per second
    pub duration: Duration,
    pub user_count: usize,
    pub market_count: usize,
}

impl LoadGenerator {
    pub fn new(tps: u32, duration_secs: u64) -> Self {
        Self {
            transaction_rate: tps,
            duration: Duration::from_secs(duration_secs),
            user_count: 100,
            market_count: 20,
        }
    }
    
    pub async fn generate_load(&self) -> PerformanceMetrics {
        let mut metrics = PerformanceMetrics::new();
        let interval = Duration::from_secs_f64(1.0 / self.transaction_rate as f64);
        let end_time = metrics.start_time + self.duration;
        
        while Instant::now() < end_time {
            // Simulate transaction
            tokio::time::sleep(interval).await;
            metrics.record_operation();
            metrics.record_memory();
        }
        
        metrics
    }
}

// Generate benchmark report in Markdown format
pub fn generate_markdown_report(results: &[BenchmarkResult], output_path: &Path) -> std::io::Result<()> {
    let mut content = String::new();
    
    content.push_str("# ConwayBets Performance Benchmark Report\n\n");
    content.push_str(&format!("Generated: {}\n\n", chrono::Utc::now()));
    
    // Summary table
    content.push_str("## Summary\n\n");
    content.push_str("| Benchmark | Duration (s) | Throughput (ops/s) | TPS | Memory (MB) |\n");
    content.push_str("|-----------|-------------|-------------------|-----|-------------|\n");
    
    for result in results {
        let memory_str = result.memory_usage
            .map(|m| format!("{:.2}", m as f64 / 1024.0 / 1024.0))
            .unwrap_or_else(|| "N/A".to_string());
        
        content.push_str(&format!(
            "| {} | {:.2} | {:.2} | {:.2} | {} |\n",
            result.name,
            result.duration.as_secs_f64(),
            result.throughput,
            result.calculate_tps(),
            memory_str
        ));
    }
    
    content.push_str("\n## Detailed Results\n\n");
    
    for result in results {
        content.push_str(&format!("### {}\n\n", result.name));
        content.push_str(&format!("- **Timestamp**: {}\n", result.timestamp));
        content.push_str(&format!("- **Iterations**: {}\n", result.iterations));
        content.push_str(&format!("- **Duration**: {:.2}s\n", result.duration.as_secs_f64()));
        content.push_str(&format!("- **Throughput**: {:.2} operations/second\n", result.throughput));
        content.push_str(&format!("- **Transactions**: {}\n", result.transaction_count));
        content.push_str(&format!("- **TPS**: {:.2}\n", result.calculate_tps()));
        
        if let Some(memory) = result.memory_usage {
            content.push_str(&format!("- **Memory Usage**: {:.2} MB\n", memory as f64 / 1024.0 / 1024.0));
        }
        
        if !result.parameters.is_empty() {
            content.push_str("\n**Parameters**:\n");
            for (key, value) in &result.parameters {
                content.push_str(&format!("  - {}: {}\n", key, value));
            }
        }
        
        content.push_str("\n---\n\n");
    }
    
    // Recommendations section
    content.push_str("## Recommendations\n\n");
    content.push_str("Based on the benchmark results:\n\n");
    
    let avg_tps: f64 = results.iter()
        .map(|r| r.calculate_tps())
        .sum::<f64>() / results.len() as f64;
    
    content.push_str(&format!("1. **Average TPS**: {:.2}\n", avg_tps));
    
    if avg_tps > 100.0 {
        content.push_str("   - ✅ Excellent performance for Conway testnet\n");
    } else if avg_tps > 50.0 {
        content.push_str("   - ⚠️ Good performance, consider optimization for high loads\n");
    } else {
        content.push_str("   - ❌ Performance needs improvement for production use\n");
    }
    
    content.push_str("\n2. **Optimization Opportunities**:\n");
    content.push_str("   - Batch cross-chain messages\n");
    content.push_str("   - Implement caching for frequent queries\n");
    content.push_str("   - Consider state compression for large markets\n");
    
    content.push_str("\n3. **Conway Testnet Performance**:\n");
    content.push_str("   - Target: <1 second transaction finality\n");
    content.push_str("   - Expected: 10-100 TPS depending on workload\n");
    content.push_str("   - Current: See individual benchmark results\n");
    
    let mut file = File::create(output_path)?;
    file.write_all(content.as_bytes())?;
    
    Ok(())
}