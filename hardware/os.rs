use serde::{Deserialize, Serialize};
use wmi::WMIConnection;
use chrono::Local;

// Win32_OperatingSystem gives OS edition, build number and boot timestamps
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_OperatingSystem")]
#[serde(rename_all = "PascalCase")]
struct WmiOs {
    caption: Option<String>,
    version: Option<String>,
    build_number: Option<String>,
    os_architecture: Option<String>,
    install_date: Option<String>,
    last_boot_up_time: Option<String>,
}

// Win32_ComputerSystem gives PC manufacturer and product model
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_ComputerSystem")]
#[serde(rename_all = "PascalCase")]
struct WmiComputerSystem {
    model: Option<String>,
    manufacturer: Option<String>,
    system_type: Option<String>,
}

/// operating system and machine information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub os_name: String,
    pub version: String,
    pub build_number: String,
    pub architecture: String,
    pub computer_manufacturer: String,
    pub computer_model: String,
    pub system_type: String,
    pub install_date: String,
    pub uptime_formatted: String,
}

impl OsInfo {
    pub fn collect(wmi_con: &WMIConnection) -> Result<Self, Box<dyn std::error::Error>> {
        let os_list: Vec<WmiOs> = wmi_con.query().unwrap_or_default();
        let cs_list: Vec<WmiComputerSystem> = wmi_con.query().unwrap_or_default();

        // take the first entry (only 1 running OS and host system)
        let os = os_list.into_iter().next().unwrap_or(WmiOs {
            caption: None,
            version: None,
            build_number: None,
            os_architecture: None,
            install_date: None,
            last_boot_up_time: None,
        });

        let cs = cs_list.into_iter().next().unwrap_or(WmiComputerSystem {
            model: None,
            manufacturer: None,
            system_type: None,
        });

        // install date comes as CIM format YYYYMMDDHHMMSS...
        let install_date_formatted = if let Some(ref d) = os.install_date {
            if d.len() >= 8 {
                format!("{}/{}/{}", &d[6..8], &d[4..6], &d[0..4])
            } else {
                d.clone()
            }
        } else {
            "Unknown".to_string()
        };

        // calculate uptime: WMI gives last bootup timestamp like 20260908123000...
        let uptime_str = if let Some(ref boot_time) = os.last_boot_up_time {
            if boot_time.len() >= 14 {
                let dt_str = &boot_time[0..14];
                if let Ok(boot_dt) = chrono::NaiveDateTime::parse_from_str(dt_str, "%Y%m%d%H%M%S") {
                    let now = Local::now().naive_local();
                    let duration = now.signed_duration_since(boot_dt);
                    let days = duration.num_days();
                    let hours = duration.num_hours() % 24;
                    let mins = duration.num_minutes() % 60;
                    if days > 0 {
                        format!("{}d {}h {}m", days, hours, mins)
                    } else {
                        format!("{}h {}m", hours, mins)
                    }
                } else {
                    "N/A".to_string()
                }
            } else {
                "N/A".to_string()
            }
        } else {
            "N/A".to_string()
        };

        Ok(OsInfo {
            os_name: os.caption.map(|s| s.trim().to_string()).unwrap_or_else(|| "Microsoft Windows".to_string()),
            version: os.version.unwrap_or_else(|| "N/A".to_string()),
            build_number: os.build_number.unwrap_or_else(|| "N/A".to_string()),
            architecture: os.os_architecture.unwrap_or_else(|| "64-bit".to_string()),
            computer_manufacturer: cs.manufacturer.map(|s| s.trim().to_string()).unwrap_or_else(|| "OEM".to_string()),
            computer_model: cs.model.map(|s| s.trim().to_string()).unwrap_or_else(|| "PC Desktop/Notebook".to_string()),
            system_type: cs.system_type.unwrap_or_else(|| "x64-based PC".to_string()),
            install_date: install_date_formatted,
            uptime_formatted: uptime_str,
        })
    }
}
