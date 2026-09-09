use serde::{Deserialize, Serialize};
use wmi::WMIConnection;

// Win32_PhysicalMemoryArray gives motherboard slot counts and max capacity
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_PhysicalMemoryArray")]
#[serde(rename_all = "PascalCase")]
struct WmiMemoryArray {
    memory_devices: Option<u32>,
    max_capacity: Option<u64>,
}

// Win32_PhysicalMemory gives details for each inserted RAM stick
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_PhysicalMemory")]
#[serde(rename_all = "PascalCase")]
struct WmiPhysicalMemory {
    capacity: Option<u64>,
    configured_clock_speed: Option<u32>,
    speed: Option<u32>,
    device_locator: Option<String>,
    bank_label: Option<String>,
    manufacturer: Option<String>,
    part_number: Option<String>,
    smbios_memory_type: Option<u32>,
    memory_type: Option<u32>,
    form_factor: Option<u32>,
}

/// individual physical RAM stick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStick {
    pub slot: String,
    pub bank: String,
    pub capacity_gb: f64,
    pub configured_frequency_mhz: u32,
    pub rated_speed_mhz: u32,
    pub memory_type: String,
    pub form_factor: String,
    pub manufacturer: String,
    pub part_number: String,
}

/// RAM topology and installed memory sticks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTopology {
    pub total_slots: u32,
    pub slots_used: u32,
    pub slots_available: u32,
    pub total_installed_gb: f64,
    pub max_supported_capacity_gb: Option<f64>,
    pub sticks: Vec<MemoryStick>,
    pub channel_mode: String,
}

impl MemoryTopology {
    pub fn collect(wmi_con: &WMIConnection) -> Result<Self, Box<dyn std::error::Error>> {
        let arrays: Vec<WmiMemoryArray> = wmi_con.query().unwrap_or_default();
        let raw_mems: Vec<WmiPhysicalMemory> = wmi_con.query().unwrap_or_default();

        // total slots from memory array (fallback to detected sticks if BIOS reports 0)
        let total_slots = arrays
            .first()
            .and_then(|a| a.memory_devices)
            .unwrap_or(raw_mems.len() as u32);

        // max capacity is given in KB by WMI, convert to GB
        let max_supported_capacity_gb = arrays
            .first()
            .and_then(|a| a.max_capacity)
            .map(|kb| (kb as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0);

        let slots_used = raw_mems.len() as u32;
        let slots_available = if total_slots >= slots_used {
            total_slots - slots_used
        } else {
            0 // clamp to 0 if BIOS reports weird slot numbers
        };

        let mut total_installed_bytes: u64 = 0;

        let sticks: Vec<MemoryStick> = raw_mems
            .into_iter()
            .enumerate()
            .map(|(idx, m)| {
                let cap_bytes = m.capacity.unwrap_or(0);
                total_installed_bytes += cap_bytes;
                let cap_gb = cap_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

                let raw_type = m.smbios_memory_type.or(m.memory_type).unwrap_or(0);
                let mem_type = Self::decode_memory_type(raw_type);
                let form_factor = Self::decode_form_factor(m.form_factor.unwrap_or(0));

                MemoryStick {
                    slot: m.device_locator.unwrap_or_else(|| format!("Slot {}", idx + 1)),
                    bank: m.bank_label.unwrap_or_else(|| "N/A".to_string()),
                    capacity_gb: (cap_gb * 10.0).round() / 10.0,
                    configured_frequency_mhz: m.configured_clock_speed.or(m.speed).unwrap_or(0),
                    rated_speed_mhz: m.speed.unwrap_or(0),
                    memory_type: mem_type,
                    form_factor,
                    manufacturer: m.manufacturer.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "Generic / OEM".to_string()),
                    part_number: m.part_number.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "N/A".to_string()),
                }
            })
            .collect();

        let total_installed_gb = (total_installed_bytes as f64 / (1024.0 * 1024.0 * 1024.0) * 10.0).round() / 10.0;

        // practical channel mode guess based on populated slot count
        let channel_mode = match slots_used {
            0 => "No memory detected",
            1 => "Single-Channel (1 stick installed)",
            2 => "Dual-Channel (2 sticks installed)",
            3 => "Tri-Channel / Flex Mode",
            4 => "Quad-Channel / Dual-Channel 4 DIMMs",
            _ => "Multi-Channel",
        }
        .to_string();

        Ok(MemoryTopology {
            total_slots,
            slots_used,
            slots_available,
            total_installed_gb,
            max_supported_capacity_gb,
            sticks,
            channel_mode,
        })
    }

    // decode SMBIOS 3.x memory type constants
    pub fn decode_memory_type(raw_type: u32) -> String {
        match raw_type {
            20 => "DDR",
            21 => "DDR2",
            22 => "DDR2 FB-DIMM",
            24 => "DDR3",
            26 => "DDR4",
            30 => "LPDDR4",
            34 => "DDR5",
            35 => "LPDDR5",
            36 => "HBM",
            37 => "HBM2",
            _ => "DDR / Unknown",
        }
        .to_string()
    }

    // 8 = desktop DIMM, 12 = laptop SODIMM
    pub fn decode_form_factor(form_factor: u32) -> String {
        match form_factor {
            8 => "DIMM (Desktop)",
            12 => "SODIMM (Notebook)",
            _ => "DIMM",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_memory_type() {
        assert_eq!(MemoryTopology::decode_memory_type(24), "DDR3");
        assert_eq!(MemoryTopology::decode_memory_type(26), "DDR4");
        assert_eq!(MemoryTopology::decode_memory_type(34), "DDR5");
        assert_eq!(MemoryTopology::decode_memory_type(999), "DDR / Unknown");
    }

    #[test]
    fn test_decode_form_factor() {
        assert_eq!(MemoryTopology::decode_form_factor(8), "DIMM (Desktop)");
        assert_eq!(MemoryTopology::decode_form_factor(12), "SODIMM (Notebook)");
        assert_eq!(MemoryTopology::decode_form_factor(0), "DIMM");
    }
}

