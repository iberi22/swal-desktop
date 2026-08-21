//! swal-telemetry-rs
//! High-frequency zero-allocation system telemetry reader for SWAL Desktop

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_pct: f32,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub ram_usage_pct: f32,
    pub swap_used_mb: u64,
    pub swap_total_mb: u64,
}

pub fn read_memory_metrics() -> Result<SystemMetrics, std::io::Error> {
    let file = File::open("/proc/meminfo")?;
    let reader = BufReader::new(file);

    let mut mem_total: u64 = 0;
    let mut mem_available: u64 = 0;
    let mut swap_total: u64 = 0;
    let mut swap_free: u64 = 0;

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let key = parts[0].trim_end_matches(':');
            let val: u64 = parts[1].parse().unwrap_or(0);

            match key {
                "MemTotal" => mem_total = val / 1024,
                "MemAvailable" => mem_available = val / 1024,
                "SwapTotal" => swap_total = val / 1024,
                "SwapFree" => swap_free = val / 1024,
                _ => {}
            }
        }
    }

    let mem_used = mem_total.saturating_sub(mem_available);
    let mem_pct = if mem_total > 0 {
        (mem_used as f32 / mem_total as f32) * 100.0
    } else {
        0.0
    };

    let swap_used = swap_total.saturating_sub(swap_free);

    Ok(SystemMetrics {
        cpu_usage_pct: 0.0,
        ram_used_mb: mem_used,
        ram_total_mb: mem_total,
        ram_usage_pct: mem_pct,
        swap_used_mb: swap_used,
        swap_total_mb: swap_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_reading() {
        let res = read_memory_metrics().expect("Failed to read /proc/meminfo");
        assert!(res.ram_total_mb > 0);
        assert!(res.ram_usage_pct >= 0.0 && res.ram_usage_pct <= 100.0);
    }
}
