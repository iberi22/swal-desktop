//! swal-telemetry-rs
//! High-frequency zero-allocation system telemetry reader for SWAL Desktop

pub mod ipc;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemMetrics {
    pub cpu_usage_pct: f32,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub ram_usage_pct: f32,
    pub swap_used_mb: u64,
    pub swap_total_mb: u64,
    pub gpu_usage_pct: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CpuTicks {
    pub total: u64,
    pub idle: u64,
}

/// Reads current raw CPU ticks from `/proc/stat`.
pub fn read_cpu_ticks() -> Result<CpuTicks, std::io::Error> {
    let file = File::open("/proc/stat")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("cpu ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let mut total: u64 = 0;
                let mut idle: u64 = 0;
                for (idx, val_str) in parts.iter().skip(1).enumerate() {
                    let val: u64 = val_str.parse().unwrap_or(0);
                    total += val;
                    // idx 3 is idle, idx 4 is iowait
                    if idx == 3 || idx == 4 {
                        idle += val;
                    }
                }
                return Ok(CpuTicks { total, idle });
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "cpu line not found in /proc/stat",
    ))
}

/// Calculates CPU usage percentage between two `CpuTicks` readings.
pub fn calculate_cpu_usage(prev: CpuTicks, current: CpuTicks) -> f32 {
    let total_delta = current.total.saturating_sub(prev.total);
    let idle_delta = current.idle.saturating_sub(prev.idle);
    if total_delta > 0 {
        let active_delta = total_delta.saturating_sub(idle_delta);
        ((active_delta as f32) / (total_delta as f32)) * 100.0
    } else {
        0.0
    }
}

/// Reads GPU usage percentage directly from sysfs without spawning external subprocesses.
pub fn read_gpu_metrics() -> f32 {
    let paths = [
        "/sys/class/drm/card0/device/gpu_busy_percent",
        "/sys/class/drm/card1/device/gpu_busy_percent",
        "/sys/class/kgsl/kgsl-3d0/gpubusy",
    ];

    for path in paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim();
            if let Ok(val) = trimmed.trim_end_matches('%').parse::<f32>() {
                return val.clamp(0.0, 100.0);
            }
        }
    }
    0.0
}

/// Reads memory metrics from `/proc/meminfo`.
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
        gpu_usage_pct: 0.0,
    })
}

/// Reads combined system metrics (CPU, Memory, Swap, GPU).
pub fn read_system_metrics(prev_cpu: Option<CpuTicks>) -> (SystemMetrics, CpuTicks) {
    let curr_cpu = read_cpu_ticks().unwrap_or_default();
    let cpu_pct = match prev_cpu {
        Some(prev) => calculate_cpu_usage(prev, curr_cpu),
        None => 0.0,
    };

    let mut metrics = read_memory_metrics().unwrap_or(SystemMetrics {
        cpu_usage_pct: 0.0,
        ram_used_mb: 0,
        ram_total_mb: 0,
        ram_usage_pct: 0.0,
        swap_used_mb: 0,
        swap_total_mb: 0,
        gpu_usage_pct: 0.0,
    });

    metrics.cpu_usage_pct = cpu_pct;
    metrics.gpu_usage_pct = read_gpu_metrics();

    (metrics, curr_cpu)
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

    #[test]
    fn test_cpu_ticks_reading_and_calc() {
        let ticks1 = read_cpu_ticks().expect("Failed to read /proc/stat");
        assert!(ticks1.total > 0);

        let ticks2 = CpuTicks {
            total: ticks1.total + 100,
            idle: ticks1.idle + 40,
        };

        let usage = calculate_cpu_usage(ticks1, ticks2);
        assert!((usage - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_gpu_reading_fallback() {
        let gpu = read_gpu_metrics();
        assert!(gpu >= 0.0 && gpu <= 100.0);
    }

    #[test]
    fn test_read_system_metrics() {
        let (metrics, ticks) = read_system_metrics(None);
        assert!(ticks.total > 0);
        assert!(metrics.ram_total_mb > 0);

        let _ticks_next = CpuTicks {
            total: ticks.total + 200,
            idle: ticks.idle + 100,
        };
        let (metrics2, _) = read_system_metrics(Some(ticks));
        assert!(metrics2.ram_total_mb > 0);
    }
}
