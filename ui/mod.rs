pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod motherboard;
pub mod os;
pub mod power;
pub mod storage;

use serde::{Deserialize, Serialize};
use wmi::WMIConnection;

use self::cpu::CpuInfo;
use self::gpu::GpuInfo;
use self::memory::MemoryTopology;
use self::motherboard::MotherboardInfo;
use self::os::OsInfo;
use self::power::PowerDiagnostic;
use self::storage::StorageInfo;

/// consolidated system report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReport {
    pub os: OsInfo,
    pub motherboard: MotherboardInfo,
    pub cpus: Vec<CpuInfo>,
    pub memory: MemoryTopology,
    pub gpus: Vec<GpuInfo>,
    pub storage: StorageInfo,
    pub power: PowerDiagnostic,
}

impl SystemReport {
    pub fn collect() -> Result<Self, Box<dyn std::error::Error>> {
        // reuse a single WMI connection for querying all hardware subsystems
        let wmi_con = WMIConnection::new()?;

        let os = OsInfo::collect(&wmi_con)?;
        let motherboard = MotherboardInfo::collect(&wmi_con)?;
        let cpus = CpuInfo::collect(&wmi_con)?;
        let memory = MemoryTopology::collect(&wmi_con)?;
        let gpus = GpuInfo::collect(&wmi_con)?;
        let storage = StorageInfo::collect(&wmi_con)?;
        // power diagnostics needs data from the other components to estimate peak load
        let power = PowerDiagnostic::calculate(&wmi_con, &cpus, &gpus, &memory, &storage);

        Ok(SystemReport {
            os,
            motherboard,
            cpus,
            memory,
            gpus,
            storage,
            power,
        })
    }
}
