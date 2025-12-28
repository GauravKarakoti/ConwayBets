use std::path::PathBuf;
use clap::Parser;
use serde_json;
use std::fs;
use std::collections::HashMap;

#[derive(Parser)]
#[command(author, version, about = "ConwayBets Benchmark Analyzer")]
struct Args {
    /// Input directory with benchmark results
    #[arg(short, long)]
    input: PathBuf,
    
    /// Output directory for reports
    #[arg(short, long)]
    output: PathBuf,
    
    /// Generate comparison report
    #[arg(short, long)]
    compare: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize)]
struct BenchmarkSummary {
    name: String,
    mean_duration_ms: f64,
    throughput_ops_per_sec: f64,
    transactions_per_sec: f64,
    confidence_interval: (f64, f64),
    samples: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Ensure output directory exists
    fs::create_dir_all(&args.output)?;
    
    println!("Analyzing benchmarks in: {}", args.input.display());
    
    // Find all benchmark results
    let mut summaries = Vec::new();
    
    for entry in fs::read_dir(&args.input)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let benchmark_name = path.file_name().unwrap().to_string_lossy();
            
            // Look for benchmark.json file
            let report_path = path.join("base").join("estimates.json");
            if report_path.exists() {
                let content = fs::read_to_string(&report_path)?;
                let estimates: serde_json::Value = serde_json::from_str(&content)?;
                
                if let Some(mean) = estimates.get("mean") {
                    let point_estimate = mean.get("point_estimate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let throughput = 1.0 / (point_estimate / 1_000_000_000.0); // Convert ns to seconds
                    
                    let summary = BenchmarkSummary {
                        name: benchmark_name.to_string(),
                        mean_duration_ms: point_estimate / 1_000_000.0, // Convert ns to ms
                        throughput_ops_per_sec: throughput,
                        transactions_per_sec: throughput, // Assuming 1 operation = 1 transaction
                        confidence_interval: (
                            mean.get("confidence_interval").and_then(|ci| ci.get("lower_bound").and_then(|v| v.as_f64())).unwrap_or(0.0) / 1_000_000.0,
                            mean.get("confidence_interval").and_then(|ci| ci.get("upper_bound").and_then(|v| v.as_f64())).unwrap_or(0.0) / 1_000_000.0,
                        ),
                        samples: mean.get("sample_size").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                    };
                    
                    summaries.push(summary);
                }
            }
        }
    }
    
    // Generate report
    let report_path = args.output.join("benchmark_analysis.json");
    let report_json = serde_json::to_string_pretty(&summaries)?;
    fs::write(&report_path, report_json)?;
    
    // Generate markdown report
    let markdown_path = args.output.join("README.md");
    let mut markdown = String::new();
    
    markdown.push_str("# ConwayBets Benchmark Analysis\n\n");
    markdown.push_str("## Performance Summary\n\n");
    markdown.push_str("| Benchmark | Mean Time (ms) | Throughput (ops/s) | TPS | Samples |\n");
    markdown.push_str("|-----------|----------------|-------------------|-----|---------|\n");
    
    for summary in &summaries {
        markdown.push_str(&format!(
            "| {} | {:.2} ± {:.2} | {:.2} | {:.2} | {} |\n",
            summary.name,
            summary.mean_duration_ms,
            (summary.confidence_interval.1 - summary.confidence_interval.0) / 2.0,
            summary.throughput_ops_per_sec,
            summary.transactions_per_sec,
            summary.samples
        ));
    }
    
    markdown.push_str("\n## Recommendations\n\n");
    
    // Analyze results and provide recommendations
    let avg_tps: f64 = summaries.iter()
        .map(|s| s.transactions_per_sec)
        .sum::<f64>() / summaries.len() as f64;
    
    markdown.push_str(&format!("**Average TPS**: {:.2}\n\n", avg_tps));
    
    if avg_tps > 100.0 {
        markdown.push_str("✅ **Excellent performance** - Meets Conway testnet requirements\n\n");
        markdown.push_str("**Next steps**:\n");
        markdown.push_str("- Consider stress testing with 1000+ concurrent users\n");
        markdown.push_str("- Implement advanced caching strategies\n");
        markdown.push_str("- Explore cross-chain optimization\n");
    } else if avg_tps > 50.0 {
        markdown.push_str("⚠️ **Good performance** - Suitable for moderate loads\n\n");
        markdown.push_str("**Optimization opportunities**:\n");
        markdown.push_str("- Batch cross-chain messages\n");
        markdown.push_str("- Optimize state synchronization\n");
        markdown.push_str("- Implement connection pooling\n");
    } else {
        markdown.push_str("❌ **Needs improvement** - Below Conway testnet expectations\n\n");
        markdown.push_str("**Critical improvements**:\n");
        markdown.push_str("- Optimize microchain communication\n");
        markdown.push_str("- Reduce cross-chain message overhead\n");
        markdown.push_str("- Implement transaction batching\n");
    }
    
    markdown.push_str("\n## Detailed Results\n\n");
    
    for summary in &summaries {
        markdown.push_str(&format!("### {}\n\n", summary.name));
        markdown.push_str(&format!("- **Mean Duration**: {:.2} ms\n", summary.mean_duration_ms));
        markdown.push_str(&format!("- **95% Confidence**: ({:.2}, {:.2}) ms\n", 
            summary.confidence_interval.0, summary.confidence_interval.1));
        markdown.push_str(&format!("- **Throughput**: {:.2} operations/second\n", summary.throughput_ops_per_sec));
        markdown.push_str(&format!("- **Estimated TPS**: {:.2}\n", summary.transactions_per_sec));
        markdown.push_str(&format!("- **Samples**: {}\n\n", summary.samples));
    }
    
    fs::write(&markdown_path, markdown)?;
    
    println!("Analysis complete!");
    println!("- JSON report: {}", report_path.display());
    println!("- Markdown report: {}", markdown_path.display());
    
    if let Some(compare_dir) = args.compare {
        println!("Generating comparison with: {}", compare_dir.display());
        // Generate comparison report
    }
    
    Ok(())
}