//! swal-telemetry-rs
//! High-frequency zero-allocation system telemetry reader for SWAL Desktop

pub mod ipc;
pub mod rapl;
pub mod storage;

pub use storage::DiskInfo;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Mutex;

static GLOBAL_RAPL_METER: Mutex<Option<rapl::RaplPowerMeter>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemMetrics {
    pub cpu_usage_pct: f32,
    pub cpu_temp_c: f32,
    pub cpu_power_watts: f32,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub ram_usage_pct: f32,
    pub swap_used_mb: u64,
    pub swap_total_mb: u64,
    pub gpu_usage_pct: f32,
    pub gpu_temp_c: f32,
    pub gpu_junction_temp_c: f32,
    pub gpu_power_watts: f32,
    pub net_rx_kbps: f32,
    pub net_tx_kbps: f32,
    #[serde(default)]
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CpuTicks {
    pub total: u64,
    pub idle: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NetworkBytes {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
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

/// Reads a millidegree temperature integer from a sysfs path into a stack buffer.
fn read_sysfs_temp(path: &str) -> Option<f32> {
    let mut file = File::open(path).ok()?;
    let mut buf = [0u8; 32];
    use std::io::Read;
    let n = file.read(&mut buf).ok()?;
    let s = std::str::from_utf8(&buf[..n]).ok()?.trim();
    let milli: f32 = s.parse().ok()?;
    Some(milli / 1000.0)
}

/// Reads a microwatt power integer from a sysfs path into a stack buffer.
fn read_sysfs_power(path: &str) -> Option<f32> {
    let mut file = File::open(path).ok()?;
    let mut buf = [0u8; 32];
    use std::io::Read;
    let n = file.read(&mut buf).ok()?;
    let s = std::str::from_utf8(&buf[..n]).ok()?.trim();
    let micro: f32 = s.parse().ok()?;
    Some(micro / 1_000_000.0)
}

/// Reads hardware temperatures for CPU (k10temp/coretemp) and GPU (amdgpu/nouveau/nvidia).
pub fn read_hardware_temperatures() -> (f32, f32, f32, f32) {
    let mut cpu_temp = 0.0f32;
    let mut gpu_temp = 0.0f32;
    let mut gpu_junc = 0.0f32;
    let mut gpu_power = 0.0f32;

    // Scan hwmon 0 to 8
    for i in 0..8 {
        let base = format!("/sys/class/hwmon/hwmon{}", i);
        let name_path = format!("{}/name", base);
        if let Ok(name) = std::fs::read_to_string(&name_path) {
            let n = name.trim();
            if n == "k10temp" || n == "coretemp" || n == "acpitz" {
                if let Some(t) = read_sysfs_temp(&format!("{}/temp1_input", base)) {
                    cpu_temp = t;
                }
            } else if n == "amdgpu" || n.starts_with("nvidia") || n == "nouveau" {
                if let Some(t) = read_sysfs_temp(&format!("{}/temp1_input", base)) {
                    gpu_temp = t;
                }
                if let Some(t) = read_sysfs_temp(&format!("{}/temp2_input", base)) {
                    gpu_junc = t;
                }
                if let Some(p) = read_sysfs_power(&format!("{}/power1_average", base)) {
                    gpu_power = p;
                }
            }
        }
    }

    (cpu_temp, gpu_temp, gpu_junc, gpu_power)
}

/// Reads network bytes from `/proc/net/dev`.
pub fn read_network_bytes() -> NetworkBytes {
    let mut bytes = NetworkBytes::default();
    if let Ok(file) = File::open("/proc/net/dev") {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if line.contains(':') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 10 {
                    let iface = parts[0].trim_end_matches(':');
                    if iface != "lo" {
                        let rx: u64 = parts[1].parse().unwrap_or(0);
                        let tx: u64 = parts[9].parse().unwrap_or(0);
                        bytes.rx_bytes += rx;
                        bytes.tx_bytes += tx;
                    }
                }
            }
        }
    }
    bytes
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
        cpu_temp_c: 0.0,
        cpu_power_watts: 0.0,
        ram_used_mb: mem_used,
        ram_total_mb: mem_total,
        ram_usage_pct: mem_pct,
        swap_used_mb: swap_used,
        swap_total_mb: swap_total,
        gpu_usage_pct: 0.0,
        gpu_temp_c: 0.0,
        gpu_junction_temp_c: 0.0,
        gpu_power_watts: 0.0,
        net_rx_kbps: 0.0,
        net_tx_kbps: 0.0,
        disks: Vec::new(),
    })
}

/// Reads instantaneous CPU package power (Watts) using the global RAPL meter.
pub fn read_cpu_power_watts() -> f32 {
    if let Ok(mut lock) = GLOBAL_RAPL_METER.lock() {
        let meter = lock.get_or_insert_with(rapl::RaplPowerMeter::new);
        meter.sample()
    } else {
        0.0
    }
}

/// Reads combined system metrics (CPU, Memory, Swap, GPU, Hardware Temps, Disks).
pub fn read_system_metrics(prev_cpu: Option<CpuTicks>) -> (SystemMetrics, CpuTicks) {
    let curr_cpu = read_cpu_ticks().unwrap_or_default();
    let cpu_pct = match prev_cpu {
        Some(prev) => calculate_cpu_usage(prev, curr_cpu),
        None => 0.0,
    };

    let mut metrics = read_memory_metrics().unwrap_or(SystemMetrics {
        cpu_usage_pct: 0.0,
        cpu_temp_c: 0.0,
        cpu_power_watts: 0.0,
        ram_used_mb: 0,
        ram_total_mb: 0,
        ram_usage_pct: 0.0,
        swap_used_mb: 0,
        swap_total_mb: 0,
        gpu_usage_pct: 0.0,
        gpu_temp_c: 0.0,
        gpu_junction_temp_c: 0.0,
        gpu_power_watts: 0.0,
        net_rx_kbps: 0.0,
        net_tx_kbps: 0.0,
        disks: Vec::new(),
    });

    metrics.cpu_usage_pct = cpu_pct;
    metrics.gpu_usage_pct = read_gpu_metrics();

    let (cpu_t, gpu_t, gpu_junc, gpu_pwr) = read_hardware_temperatures();
    metrics.cpu_temp_c = cpu_t;
    metrics.cpu_power_watts = read_cpu_power_watts();
    metrics.gpu_temp_c = gpu_t;
    metrics.gpu_junction_temp_c = gpu_junc;
    metrics.gpu_power_watts = gpu_pwr;
    metrics.disks = storage::scan_mounted_partitions();

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
    fn test_hardware_temperatures() {
        let (cpu, gpu, junc, pwr) = read_hardware_temperatures();
        assert!(cpu >= 0.0);
        assert!(gpu >= 0.0);
        assert!(junc >= 0.0);
        assert!(pwr >= 0.0);
    }

    #[test]
    fn test_cpu_power_reading() {
        let power = read_cpu_power_watts();
        assert!(power >= 0.0);
    }

    #[test]
    fn test_network_bytes_reading() {
        let net = read_network_bytes();
        let _ = net.rx_bytes + net.tx_bytes;
    }

    #[test]
    fn test_read_system_metrics() {
        let (metrics, ticks) = read_system_metrics(None);
        assert!(ticks.total > 0);
        assert!(metrics.ram_total_mb > 0);
        assert!(metrics.cpu_power_watts >= 0.0);
        assert!(!metrics.disks.is_empty(), "Disks should be populated on Linux host");

        let _ticks_next = CpuTicks {
            total: ticks.total + 200,
            idle: ticks.idle + 100,
        };
        let (metrics2, _) = read_system_metrics(Some(ticks));
        assert!(metrics2.ram_total_mb > 0);
        assert!(metrics2.cpu_power_watts >= 0.0);
        assert!(!metrics2.disks.is_empty());
    }
}

