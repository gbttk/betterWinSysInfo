use serde::{Deserialize, Serialize};
use wmi::WMIConnection;
use crate::hardware::cpu::CpuInfo;
use crate::hardware::gpu::GpuInfo;
use crate::hardware::memory::MemoryTopology;
use crate::hardware::storage::StorageInfo;

// queries Win32_Battery for laptop battery status
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_Battery")]
#[serde(rename_all = "PascalCase")]
struct WmiBattery {
    estimated_charge_remaining: Option<u16>,
    battery_status: Option<u16>,
}

// queries root\wmi for ACPI thermal zones
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "MSAcpi_ThermalZoneTemperature")]
#[serde(rename_all = "PascalCase")]
struct WmiThermalZone {
    instance_name: Option<String>,
    current_temperature: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalSensor {
    pub name: String,
    pub temperature_celsius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryDiagnostic {
    pub is_present: bool,
    pub charge_percent: Option<u16>,
    pub status: String,
}

/// power consumption diagnostics and PSU sizing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerDiagnostic {
    pub estimated_cpu_tdp_w: u32,
    pub estimated_gpu_tdp_w: u32,
    pub estimated_platform_w: u32,
    pub estimated_peak_system_draw_w: u32,
    pub recommended_psu_wattage_w: u32,
    pub profile_category: String,
    pub battery: BatteryDiagnostic,
    pub thermal_sensors: Vec<ThermalSensor>,
    pub thermal_status_note: String,
}

impl PowerDiagnostic {
    pub fn calculate(
        wmi_con: &WMIConnection,
        cpus: &[CpuInfo],
        gpus: &[GpuInfo],
        memory: &MemoryTopology,
        storage: &StorageInfo,
    ) -> Self {
        // 1. battery check
        let raw_batteries: Vec<WmiBattery> = wmi_con.query().unwrap_or_default();
        let battery = if let Some(bat) = raw_batteries.into_iter().next() {
            let status = match bat.battery_status.unwrap_or(0) {
                1 => "Discharging",
                2 => "Connected to AC power",
                3 => "Fully Charged",
                4 => "Low battery",
                5 => "Critical battery",
                6 => "Charging",
                _ => "In use",
            }
            .to_string();

            BatteryDiagnostic {
                is_present: true,
                charge_percent: bat.estimated_charge_remaining,
                status,
            }
        } else {
            BatteryDiagnostic {
                is_present: false,
                charge_percent: None,
                status: "No battery detected (Desktop PC)".to_string(),
            }
        };

        // 2. ACPI thermal diagnostics via root\wmi namespace
        let mut thermal_sensors = Vec::new();
        let thermal_status_note = match WMIConnection::with_namespace_path("root\\wmi") {
            Ok(wmi_thermal) => {
                let thermals: Result<Vec<WmiThermalZone>, _> = wmi_thermal.query();
                match thermals {
                    Ok(tzs) if !tzs.is_empty() => {
                        for tz in tzs {
                            if let Some(raw_kelvin) = tz.current_temperature {
                                // raw reading is in tenths of Kelvin (deci-Kelvin)
                                let celsius = (raw_kelvin as f64 / 10.0) - 273.15;
                                // ignore crazy readings (below 0C or burning silicon)
                                if (0.0..=120.0).contains(&celsius) {
                                    thermal_sensors.push(ThermalSensor {
                                        name: tz.instance_name.unwrap_or_else(|| "ACPI Zone".to_string()),
                                        temperature_celsius: (celsius * 10.0).round() / 10.0,
                                    });
                                }
                            }
                        }
                        "ACPI sensors read successfully".to_string()
                    }
                    _ => "ACPI thermal zones not exposed by OEM BIOS (common in desktops without motherboard driver)".to_string(),
                }
            }
            Err(_) => "Access to root\\wmi unavailable".to_string(),
        };

        // 3. CPu tdp estimative
        let cpu_tdp = cpus.first().map(|cpu| {
            let name = cpu.model.to_uppercase();
            if name.contains("I9-") || name.contains("RYZEN 9") || name.contains("THREADRIPPER") {
                180
            } else if name.contains("I7-") || name.contains("RYZEN 7") {
                if name.contains("4790") || name.contains("3770") || name.contains("2600") {
                    84 // Haswell / Ivy Bridge classics
                } else if name.contains('K') || name.contains('X') {
                    125
                } else {
                    95
                }
            } else if name.contains("I5-") || name.contains("RYZEN 5") {
                if name.contains('K') || name.contains('X') {
                    95
                } else {
                    65
                }
            } else if name.contains("I3-") || name.contains("RYZEN 3") || name.contains("PENTIUM") || name.contains("CELERON") {
                54
            } else if name.contains('U') || name.contains('H') || name.contains("HQ") {
                35 // notebook/mobile
            } else {
                // fallback formula if model string is weird: ~15W per core + base overhead
                cpu.physical_cores * 15 + 20
            }
        }).unwrap_or(65);

        // 4. GPU tdp estimative
        let mut gpu_tdp = 0;
        for gpu in gpus {
            let name = gpu.name.to_uppercase();
            if !gpu.is_dedicated {
                // igpu consumes from the cpu
                gpu_tdp += 5;
            } else if name.contains("4090") || name.contains("7900 XTX") {
                gpu_tdp += 450;
            } else if name.contains("4080") || name.contains("7900 XT") || name.contains("3090") || name.contains("3080") {
                gpu_tdp += 320;
            } else if name.contains("4070") || name.contains("3070") || name.contains("6800") {
                gpu_tdp += 220;
            } else if name.contains("4060") || name.contains("3060") || name.contains("6700") || name.contains("2060") {
                gpu_tdp += 160;
            } else if name.contains("1660") || name.contains("1650") || name.contains("RX 580") || name.contains("RX 570") {
                gpu_tdp += 120;
            } else if name.contains("1050") || name.contains("RX 550") || name.contains("1030") {
                gpu_tdp += 65;
            } else {
                gpu_tdp += 75; // dedicated/entry-level/old gpu
            }
        }

        // 5. platform components consumption
        let ram_wattage = (memory.slots_used * 3) as u32; // ~3W by DDR3/DDR4/DDR5
        let storage_wattage = (storage.physical_disks.len() as u32) * 6; // ~6W by SSD/HDD
        let motherboard_and_fans_w = 45; // motherboard, VRM, fans and USB devices

        let estimated_platform_w = ram_wattage + storage_wattage + motherboard_and_fans_w;
        let estimated_peak_system_draw_w = cpu_tdp + gpu_tdp + estimated_platform_w;

        let recommended_psu_wattage_w = Self::calculate_recommended_psu(estimated_peak_system_draw_w);
        let profile_category = Self::categorize_profile(estimated_peak_system_draw_w);

        PowerDiagnostic {
            estimated_cpu_tdp_w: cpu_tdp,
            estimated_gpu_tdp_w: gpu_tdp,
            estimated_platform_w,
            estimated_peak_system_draw_w,
            recommended_psu_wattage_w,
            profile_category,
            battery,
            thermal_sensors,
            thermal_status_note,
        }
    }

    // recommends a PSU with ~30% safety headroom so the unit stays in its 50-80% efficiency sweet spot
    pub fn calculate_recommended_psu(peak_system_draw_w: u32) -> u32 {
        let raw_recommended = (peak_system_draw_w as f64 * 1.30).round() as u32;
        // round up to standard commercial PSU wattage ratings
        match raw_recommended {
            0..=300 => 350,
            301..=400 => 450,
            401..=500 => 550,
            501..=600 => 650,
            601..=720 => 750,
            721..=850 => 850,
            851..=1000 => 1000,
            _ => 1200,
        }
    }

    // categorizes system profile based on peak wattage draw
    pub fn categorize_profile(peak_system_draw_w: u32) -> String {
        if peak_system_draw_w <= 160 {
            "Efficient Profile / Office or Mini-PC".to_string()
        } else if peak_system_draw_w <= 350 {
            "Intermediate Profile / Mainstream".to_string()
        } else if peak_system_draw_w <= 550 {
            "High-Performance Profile / Gaming".to_string()
        } else {
            "Extreme Enthusiast Profile / Workstation".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommended_psu() {
        // 146W peak: 146 * 1.3 = 189.8 -> 350W
        assert_eq!(PowerDiagnostic::calculate_recommended_psu(146), 350);
        // 300W peak: 300 * 1.3 = 390 -> 450W
        assert_eq!(PowerDiagnostic::calculate_recommended_psu(300), 450);
        // 500W peak: 500 * 1.3 = 650 -> 750W
        assert_eq!(PowerDiagnostic::calculate_recommended_psu(500), 750);
    }

    #[test]
    fn test_categorize_profile() {
        assert_eq!(PowerDiagnostic::categorize_profile(120), "Efficient Profile / Office or Mini-PC");
        assert_eq!(PowerDiagnostic::categorize_profile(250), "Intermediate Profile / Mainstream");
        assert_eq!(PowerDiagnostic::categorize_profile(450), "High-Performance Profile / Gaming");
        assert_eq!(PowerDiagnostic::categorize_profile(700), "Extreme Enthusiast Profile / Workstation");
    }
}
