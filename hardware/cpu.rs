use serde::{Deserialize, Serialize};
use wmi::WMIConnection;

// queries Win32_Processor via WMI to fetch cpu topology, clocks and caches
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_Processor")]
#[serde(rename_all = "PascalCase")]
struct WmiProcessor {
    name: Option<String>,
    manufacturer: Option<String>,
    number_of_cores: Option<u32>,
    number_of_logical_processors: Option<u32>,
    max_clock_speed: Option<u32>,
    current_clock_speed: Option<u32>,
    socket_designation: Option<String>,
    l2_cache_size: Option<u32>,
    l3_cache_size: Option<u32>,
    load_percentage: Option<u16>,
    architecture: Option<u16>,
}

/// cpu specs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub manufacturer: String,
    pub physical_cores: u32,
    pub logical_threads: u32,
    pub max_clock_mhz: u32,
    pub current_clock_mhz: Option<u32>,
    pub socket: String,
    pub l2_cache_kb: Option<u32>,
    pub l3_cache_kb: Option<u32>,
    pub load_percentage: Option<u16>,
    pub architecture: String,
}

impl CpuInfo {
    pub fn collect(wmi_con: &WMIConnection) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let raw_procs: Vec<WmiProcessor> = wmi_con.query()?;
        
        let cpus = raw_procs
            .into_iter()
            .map(|p| {
                // architecture enum from MS docs (9 = x64 is 99% of desktop machines)
                let arch = match p.architecture.unwrap_or(9) {
                    0 => "x86 (32-bit)",
                    1 => "MIPS",
                    2 => "Alpha",
                    3 => "PowerPC",
                    5 => "ARM",
                    6 => "ia64 (Itanium)", // if someone runs this on itanium, send help
                    9 => "x64 (64-bit)",
                    12 => "ARM64",
                    _ => "Unknown",
                }
                .to_string();

                CpuInfo {
                    model: p.name.map(|s| s.trim().to_string()).unwrap_or_else(|| "Unknown CPU".to_string()),
                    manufacturer: p.manufacturer.map(|s| s.trim().to_string()).unwrap_or_else(|| "Unknown".to_string()),
                    // fallback to 1 core/thread in case hypervisors or weird VMs hide it
                    physical_cores: p.number_of_cores.unwrap_or(1),
                    logical_threads: p.number_of_logical_processors.unwrap_or(1),
                    // clocks are in MHz, caches in KB (wmi standard)
                    max_clock_mhz: p.max_clock_speed.unwrap_or(0),
                    current_clock_mhz: p.current_clock_speed,
                    socket: p.socket_designation.map(|s| s.trim().to_string()).unwrap_or_else(|| "N/A".to_string()),
                    l2_cache_kb: p.l2_cache_size,
                    l3_cache_kb: p.l3_cache_size,
                    // note: this is an instantaneous snapshot of CPU usage from the WMI query moment
                    load_percentage: p.load_percentage,
                    architecture: arch,
                }
            })
            .collect();

        Ok(cpus)
    }
}
