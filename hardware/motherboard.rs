use serde::{Deserialize, Serialize};
use wmi::WMIConnection;

// Win32_BaseBoard holds physical motherboard info
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_BaseBoard")]
#[serde(rename_all = "PascalCase")]
struct WmiBaseBoard {
    manufacturer: Option<String>,
    product: Option<String>,
    version: Option<String>,
    serial_number: Option<String>,
}

// Win32_BIOS holds firmware details
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_BIOS")]
#[serde(rename_all = "PascalCase")]
struct WmiBios {
    manufacturer: Option<String>,
    smbios_bios_version: Option<String>,
    release_date: Option<String>,
}

/// motherboard and BIOS details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub version: String,
    pub serial_number: String,
    pub bios_vendor: String,
    pub bios_version: String,
    pub bios_date: String,
}

impl MotherboardInfo {
    pub fn collect(wmi_con: &WMIConnection) -> Result<Self, Box<dyn std::error::Error>> {
        let boards: Vec<WmiBaseBoard> = wmi_con.query().unwrap_or_default();
        let bios_list: Vec<WmiBios> = wmi_con.query().unwrap_or_default();

        // grab first entry (pretty much every machine only has one motherboard and BIOS)
        let board = boards.into_iter().next().unwrap_or(WmiBaseBoard {
            manufacturer: None,
            product: None,
            version: None,
            serial_number: None,
        });

        let bios = bios_list.into_iter().next().unwrap_or(WmiBios {
            manufacturer: None,
            smbios_bios_version: None,
            release_date: None,
        });

        // WMI dates come in CIM format like YYYYMMDD... format it nicely to DD/MM/YYYY
        let bios_date_formatted = if let Some(ref d) = bios.release_date {
            if d.len() >= 8 {
                format!("{}/{}/{}", &d[6..8], &d[4..6], &d[0..4])
            } else {
                d.clone()
            }
        } else {
            "N/A".to_string()
        };

        // OEM boards or cheap prebuilts often have blank fields, so fall back to sane defaults
        Ok(MotherboardInfo {
            manufacturer: board.manufacturer.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "Unknown".to_string()),
            product: board.product.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "Generic Model / OEM".to_string()),
            version: board.version.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "N/A".to_string()),
            serial_number: board.serial_number.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "N/A".to_string()),
            bios_vendor: bios.manufacturer.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "Unknown".to_string()),
            bios_version: bios.smbios_bios_version.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "N/A".to_string()),
            bios_date: bios_date_formatted,
        })
    }
}

