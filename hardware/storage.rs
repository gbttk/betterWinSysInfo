use serde::{Deserialize, Serialize};
use wmi::WMIConnection;

// Win32_DiskDrive represents physical drives (NVMe, SATA SSD, HDD, external USB)
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_DiskDrive")]
#[serde(rename_all = "PascalCase")]
struct WmiDiskDrive {
    model: Option<String>,
    size: Option<u64>,
    interface_type: Option<String>,
    media_type: Option<String>,
    partitions: Option<u32>,
}

// Win32_LogicalDisk represents mounted volumes/partitions (C:, D:, etc.)
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_LogicalDisk")]
#[serde(rename_all = "PascalCase")]
struct WmiLogicalDisk {
    device_id: Option<String>,
    volume_name: Option<String>,
    file_system: Option<String>,
    size: Option<u64>,
    free_space: Option<u64>,
}

/// physical hard drive or SSD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalDisk {
    pub model: String,
    pub capacity_gb: f64,
    pub interface: String,
    pub media_type: String,
    pub partitions: u32,
}

/// logical drive letter / partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalVolume {
    pub drive_letter: String,
    pub label: String,
    pub file_system: String,
    pub total_gb: f64,
    pub free_gb: f64,
    pub used_gb: f64,
    pub percent_used: f64,
}

/// physical and logical storage units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub physical_disks: Vec<PhysicalDisk>,
    pub logical_volumes: Vec<LogicalVolume>,
}

impl StorageInfo {
    pub fn collect(wmi_con: &WMIConnection) -> Result<Self, Box<dyn std::error::Error>> {
        let raw_disks: Vec<WmiDiskDrive> = wmi_con.query().unwrap_or_default();
        let raw_vols: Vec<WmiLogicalDisk> = wmi_con.query().unwrap_or_default();

        let physical_disks = raw_disks
            .into_iter()
            .map(|d| {
                let size_bytes = d.size.unwrap_or(0);
                // disk makers advertise capacity using 1000^3 (marketing gigabytes)
                let gb = (size_bytes as f64 / (1000.0 * 1000.0 * 1000.0) * 10.0).round() / 10.0;

                PhysicalDisk {
                    model: d.model.map(|s| s.trim().to_string()).unwrap_or_else(|| "Unknown Disk".to_string()),
                    capacity_gb: gb,
                    interface: d.interface_type.map(|s| s.trim().to_string()).unwrap_or_else(|| "N/A".to_string()),
                    media_type: d.media_type.map(|s| s.trim().to_string()).unwrap_or_else(|| "N/A".to_string()),
                    partitions: d.partitions.unwrap_or(1),
                }
            })
            .collect();

        let logical_volumes = raw_vols
            .into_iter()
            // ignore empty card readers or unmounted optical drives
            .filter(|v| v.size.unwrap_or(0) > 0)
            .map(|v| {
                let total_bytes = v.size.unwrap_or(0);
                let free_bytes = v.free_space.unwrap_or(0);
                // saturating_sub prevents underflow if windows reports free > total by accident
                let used_bytes = total_bytes.saturating_sub(free_bytes);

                // for partitions, Windows Explorer calculates with 1024^3 (GiB)
                let total_gb = (total_bytes as f64 / (1024.0 * 1024.0 * 1024.0) * 10.0).round() / 10.0;
                let free_gb = (free_bytes as f64 / (1024.0 * 1024.0 * 1024.0) * 10.0).round() / 10.0;
                let used_gb = (used_bytes as f64 / (1024.0 * 1024.0 * 1024.0) * 10.0).round() / 10.0;
                
                let percent_used = if total_bytes > 0 {
                    ((used_bytes as f64 / total_bytes as f64) * 1000.0).round() / 10.0
                } else {
                    0.0
                };

                LogicalVolume {
                    drive_letter: v.device_id.unwrap_or_else(|| "?:".to_string()),
                    label: v.volume_name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "Unnamed".to_string()),
                    file_system: v.file_system.unwrap_or_else(|| "RAW".to_string()),
                    total_gb,
                    free_gb,
                    used_gb,
                    percent_used,
                }
            })
            .collect();

        Ok(StorageInfo {
            physical_disks,
            logical_volumes,
        })
    }
}
