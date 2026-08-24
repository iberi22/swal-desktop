//! RAPL (Running Average Power Limit) Energy and CPU Package Power Profiler
//! Zero-allocation microjoule sysfs reader for Intel and AMD CPUs

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Standard candidate sysfs paths for RAPL energy microjoule counters.
const DEFAULT_RAPL_PATHS: &[&str] = &[
    "/sys/class/powercap/intel-rapl:0/energy_uj",
    "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj",
    "/sys/devices/virtual/powercap/intel-rapl/intel-rapl:0/energy_uj",
    "/sys/class/powercap/intel-rapl:0:0/energy_uj",
    "/sys/class/powercap/intel-rapl:1/energy_uj",
];

/// Reads a `u64` microjoule counter from a sysfs path into a stack-allocated buffer.
pub fn read_energy_uj_from_file(path: &Path) -> Option<u64> {
    let mut file = File::open(path).ok()?;
    let mut buf = [0u8; 32];
    let n = file.read(&mut buf).ok()?;
    let s = std::str::from_utf8(&buf[..n]).ok()?.trim();
    s.parse::<u64>().ok()
}

/// Detects the active RAPL package energy file on the host system.
pub fn detect_rapl_path() -> Option<PathBuf> {
    for &candidate in DEFAULT_RAPL_PATHS {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return Some(p);
        }
    }

    // Dynamic scan under /sys/class/powercap/
    if let Ok(entries) = std::fs::read_dir("/sys/class/powercap") {
        for entry in entries.flatten() {
            let path = entry.path();
            let energy_path = path.join("energy_uj");
            if energy_path.exists() {
                return Some(energy_path);
            }
        }
    }

    None
}

/// High-performance RAPL Power Meter calculating instantaneous CPU package power (Watts).
#[derive(Debug, Clone)]
pub struct RaplPowerMeter {
    energy_path: Option<PathBuf>,
    max_energy_range_uj: Option<u64>,
    last_energy_uj: Option<u64>,
    last_timestamp: Option<Instant>,
}

impl Default for RaplPowerMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl RaplPowerMeter {
    /// Creates a new `RaplPowerMeter` by detecting the primary RAPL sysfs path.
    pub fn new() -> Self {
        let energy_path = detect_rapl_path();
        let max_energy_range_uj = energy_path.as_ref().and_then(|p| {
            let max_path = p.parent()?.join("max_energy_range_uj");
            read_energy_uj_from_file(&max_path)
        });

        Self {
            energy_path,
            max_energy_range_uj,
            last_energy_uj: None,
            last_timestamp: None,
        }
    }

    /// Creates a `RaplPowerMeter` bound to an explicit sysfs energy file path.
    pub fn with_path<P: Into<PathBuf>>(path: P) -> Self {
        let energy_path = path.into();
        let max_energy_range_uj = energy_path.parent().and_then(|parent| {
            let max_path = parent.join("max_energy_range_uj");
            read_energy_uj_from_file(&max_path)
        });

        Self {
            energy_path: Some(energy_path),
            max_energy_range_uj,
            last_energy_uj: None,
            last_timestamp: None,
        }
    }

    /// Returns the active sysfs energy path if one was discovered or provided.
    pub fn energy_path(&self) -> Option<&Path> {
        self.energy_path.as_deref()
    }

    /// Returns the maximum energy range (wraparound point) in microjoules if available.
    pub fn max_energy_range_uj(&self) -> Option<u64> {
        self.max_energy_range_uj
    }

    /// Checks if RAPL readings are currently readable on this platform.
    pub fn is_available(&self) -> bool {
        self.read_current_energy_uj().is_some()
    }

    /// Reads the current raw microjoule counter from sysfs.
    pub fn read_current_energy_uj(&self) -> Option<u64> {
        let path = self.energy_path.as_ref()?;
        read_energy_uj_from_file(path)
    }

    /// Resets the internal state and timestamps.
    pub fn reset(&mut self) {
        self.last_energy_uj = None;
        self.last_timestamp = None;
    }

    /// Computes delta microjoules between two consecutive readings, handling counter wraparound.
    pub fn calculate_delta_uj(prev_uj: u64, curr_uj: u64, max_range: Option<u64>) -> u64 {
        if curr_uj >= prev_uj {
            curr_uj - prev_uj
        } else if let Some(max_range) = max_range {
            if max_range > prev_uj {
                (max_range - prev_uj).saturating_add(curr_uj)
            } else {
                curr_uj.wrapping_sub(prev_uj)
            }
        } else {
            curr_uj.wrapping_sub(prev_uj)
        }
    }

    /// Calculates power in Watts given microjoule delta and elapsed seconds:
    /// `watts = (delta_uj / 1_000_000.0) / elapsed_secs`
    pub fn calculate_power(delta_uj: u64, elapsed_secs: f32) -> f32 {
        if elapsed_secs <= 0.0 {
            return 0.0;
        }
        let joules = (delta_uj as f64) / 1_000_000.0;
        let watts = (joules / (elapsed_secs as f64)) as f32;
        if watts.is_finite() && watts >= 0.0 {
            watts
        } else {
            0.0
        }
    }

    /// Samples current microjoules and calculates instantaneous CPU power in Watts.
    /// Returns `0.0` on first invocation or when RAPL is unavailable.
    pub fn sample(&mut self) -> f32 {
        let current_uj = match self.read_current_energy_uj() {
            Some(uj) => uj,
            None => return 0.0,
        };
        let now = Instant::now();
        self.sample_with_values(current_uj, now)
    }

    /// Internal sampling helper allowing deterministic testing with explicit values.
    pub fn sample_with_values(&mut self, current_uj: u64, now: Instant) -> f32 {
        let power = match (self.last_energy_uj, self.last_timestamp) {
            (Some(prev_uj), Some(prev_time)) => {
                let elapsed = now.duration_since(prev_time).as_secs_f32();
                if elapsed > 0.0 {
                    let delta_uj = Self::calculate_delta_uj(prev_uj, current_uj, self.max_energy_range_uj);
                    Self::calculate_power(delta_uj, elapsed)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };

        self.last_energy_uj = Some(current_uj);
        self.last_timestamp = Some(now);
        power
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::Duration;

    #[test]
    fn test_rapl_power_math() {
        // 1,000,000 uJ in 1.0 s = 1.0 J / 1.0 s = 1.0 Watt
        let w1 = RaplPowerMeter::calculate_power(1_000_000, 1.0);
        assert!((w1 - 1.0).abs() < 1e-4);

        // 65,000,000 uJ in 1.0 s = 65.0 Watts (standard desktop TDP)
        let w2 = RaplPowerMeter::calculate_power(65_000_000, 1.0);
        assert!((w2 - 65.0).abs() < 1e-4);

        // 32,500,000 uJ in 0.5 s = 65.0 Watts
        let w3 = RaplPowerMeter::calculate_power(32_500_000, 0.5);
        assert!((w3 - 65.0).abs() < 1e-4);

        // 0 delta = 0.0 Watts
        let w0 = RaplPowerMeter::calculate_power(0, 1.0);
        assert_eq!(w0, 0.0);

        // 0 elapsed = 0.0 Watts
        let w_zero_time = RaplPowerMeter::calculate_power(1_000_000, 0.0);
        assert_eq!(w_zero_time, 0.0);
    }

    #[test]
    fn test_rapl_wraparound_delta() {
        // Normal monotonic increase
        let delta = RaplPowerMeter::calculate_delta_uj(100, 500, None);
        assert_eq!(delta, 400);

        // Wraparound with explicit max range
        let max_range = 1_000_000u64;
        let delta_wrap = RaplPowerMeter::calculate_delta_uj(999_900, 200, Some(max_range));
        assert_eq!(delta_wrap, 300);

        // Wraparound without explicit max range (wrapping_sub fallback)
        let delta_wrap_no_max = RaplPowerMeter::calculate_delta_uj(u64::MAX - 50, 49, None);
        assert_eq!(delta_wrap_no_max, 100);
    }

    #[test]
    fn test_rapl_sample_state_progression() {
        let mut meter = RaplPowerMeter::with_path("/dev/null");
        let t0 = Instant::now();

        // First sample should return 0.0 W because there is no previous reference
        let p0 = meter.sample_with_values(10_000_000, t0);
        assert_eq!(p0, 0.0);

        // Second sample after 1 second with +45,000,000 uJ -> 45.0 W
        let t1 = t0 + Duration::from_secs(1);
        let p1 = meter.sample_with_values(55_000_000, t1);
        assert!((p1 - 45.0).abs() < 1e-4);

        // Third sample after another 0.5s with +40,000,000 uJ -> 80.0 W
        let t2 = t1 + Duration::from_millis(500);
        let p2 = meter.sample_with_values(95_000_000, t2);
        assert!((p2 - 80.0).abs() < 1e-4);
    }

    #[test]
    fn test_rapl_file_reading() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_rapl_energy_{}.txt", std::process::id()));

        // Write microjoules to temp file
        let mut f = File::create(&file_path).expect("Failed to create temp rapl file");
        writeln!(f, "123456789").unwrap();
        drop(f);

        let read_val = read_energy_uj_from_file(&file_path);
        assert_eq!(read_val, Some(123456789));

        // Test with meter
        let meter = RaplPowerMeter::with_path(&file_path);
        assert!(meter.is_available());
        assert_eq!(meter.read_current_energy_uj(), Some(123456789));

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_rapl_fallback_nonexistent_file() {
        let mut meter = RaplPowerMeter::with_path("/nonexistent/sys/class/powercap/energy_uj");
        assert!(!meter.is_available());
        let power = meter.sample();
        assert_eq!(power, 0.0);
    }

    #[test]
    fn test_rapl_default_detector() {
        let mut meter = RaplPowerMeter::new();
        // Even if permission is denied or path is missing in sandbox/CI, sample() must safely return >= 0.0
        let p = meter.sample();
        assert!(p >= 0.0);
    }
}
