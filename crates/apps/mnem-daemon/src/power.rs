use std::path::Path;
use std::process::Command;

/// Power source detection for macOS.
/// Returns true if the system is currently running on battery power.
pub fn is_on_battery() -> bool {
    // macOS: use pmset to check power source
    if let Ok(output) = Command::new("pmset").arg("-g").arg("batt").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.contains("Battery Power");
    }

    // Fallback: check sysfs on Linux
    let battery_path = Path::new("/sys/class/power_supply/BAT0/status");
    if battery_path.exists() {
        if let Ok(status) = std::fs::read_to_string(battery_path) {
            return status.trim() == "Discharging";
        }
    }

    false
}

/// Battery level as a percentage (0-100). Returns None if unavailable.
pub fn battery_level() -> Option<u8> {
    // macOS
    if let Ok(output) = Command::new("pmset").arg("-g").arg("batt").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse "XX%" from pmset output
        for word in stdout.split_whitespace() {
            if word.ends_with("%;") || word.ends_with('%') {
                let num_str = word.trim_end_matches(|c| c == '%' || c == ';');
                if let Ok(level) = num_str.parse::<u8>() {
                    return Some(level);
                }
            }
        }
    }

    // Linux fallback
    let capacity_path = Path::new("/sys/class/power_supply/BAT0/capacity");
    if capacity_path.exists() {
        if let Ok(content) = std::fs::read_to_string(capacity_path) {
            if let Ok(level) = content.trim().parse::<u8>() {
                return Some(level);
            }
        }
    }

    None
}

/// Adaptive settings based on current power state.
#[derive(Debug, Clone)]
pub struct PowerProfile {
    /// Compression level (1-22 for zstd). Lower = faster, less battery.
    pub compression_level: i32,
    /// Debounce interval for file events, in milliseconds. Higher = less CPU wake.
    pub debounce_ms: u64,
    /// Whether to run GC now. Deferred on battery.
    pub allow_gc: bool,
    /// Max parallel threads for initial scan.
    pub scan_parallelism: usize,
}

impl PowerProfile {
    /// Create a profile adapted to the current power state.
    pub fn detect() -> Self {
        let on_battery = is_on_battery();
        let level = battery_level();

        if on_battery {
            let critical = level.map_or(false, |l| l < 20);

            if critical {
                // Critical battery: minimum activity
                Self {
                    compression_level: 1,
                    debounce_ms: 5000,
                    allow_gc: false,
                    scan_parallelism: 1,
                }
            } else {
                // Normal battery: reduced activity
                Self {
                    compression_level: 3,
                    debounce_ms: 2000,
                    allow_gc: false,
                    scan_parallelism: 2,
                }
            }
        } else {
            // AC power: full performance
            Self {
                compression_level: 6,
                debounce_ms: 500,
                allow_gc: true,
                scan_parallelism: num_cpus(),
            }
        }
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}
