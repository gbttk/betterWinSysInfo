use serde::{Deserialize, Serialize};
use wmi::WMIConnection;

// Win32_VideoController lists all display adapters (discrete and integrated)
#[derive(Deserialize, Debug, Clone)]
#[serde(rename = "Win32_VideoController")]
#[serde(rename_all = "PascalCase")]
struct WmiVideoController {
    name: Option<String>,
    driver_version: Option<String>,
    driver_date: Option<String>,
    adapter_ram: Option<u64>,
    video_processor: Option<String>,
    current_horizontal_resolution: Option<u32>,
    current_vertical_resolution: Option<u32>,
    current_refresh_rate: Option<u32>,
}

/// GPU details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
    pub vram_gb: f64,
    pub driver_version: String,
    pub driver_date: String,
    pub video_processor: String,
    pub resolution: String,
    pub refresh_rate_hz: Option<u32>,
    pub is_dedicated: bool,
}

impl GpuInfo {
    pub fn collect(wmi_con: &WMIConnection) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let raw_gpus: Vec<WmiVideoController> = wmi_con.query().unwrap_or_default();

        let gpus = raw_gpus
            .into_iter()
            .map(|g| {
                let name = g.name.map(|s| s.trim().to_string()).unwrap_or_else(|| "Unknown GPU".to_string());
                
                // sooo... in older windows drivers, AdapterRAM can suffer overflow or come zero
                let bytes = g.adapter_ram.unwrap_or(0);
                let mb = bytes / (1024 * 1024);
                let gb = (mb as f64 / 1024.0 * 10.0).round() / 10.0;

                // driver date format: YYYYMMDD...
                let driver_date = if let Some(ref d) = g.driver_date {
                    if d.len() >= 8 {
                        format!("{}/{}/{}", &d[6..8], &d[4..6], &d[0..4])
                    } else {
                        d.clone()
                    }
                } else {
                    "N/A".to_string()
                };

                // if resolution is 0x0, monitor is sleeping, unplugged, or this adapter has no active display
                let resolution = match (g.current_horizontal_resolution, g.current_vertical_resolution) {
                    (Some(w), Some(h)) if w > 0 && h > 0 => format!("{}x{}", w, h),
                    _ => "Disconnected / Secondary".to_string(),
                };

                // simple check to distinguish iGPU vs dedicated dGPU
                // "basic display" = windows just gave up on finding a real driver
                let lower_name = name.to_lowercase();
                let is_dedicated = !lower_name.contains("intel(r) hd") 
                    && !lower_name.contains("intel(r) uhd")
                    && !lower_name.contains("intel(r) iris")
                    && !lower_name.contains("radeon(tm) graphics")
                    && !lower_name.contains("vega 8")
                    && !lower_name.contains("vega 7")
                    && !lower_name.contains("basic display");

                GpuInfo {
                    name,
                    vram_mb: mb,
                    vram_gb: gb,
                    driver_version: g.driver_version.unwrap_or_else(|| "N/A".to_string()),
                    driver_date,
                    video_processor: g.video_processor.unwrap_or_else(|| "N/A".to_string()),
                    resolution,
                    refresh_rate_hz: g.current_refresh_rate,
                    is_dedicated,
                }
            })
            .collect();

        Ok(gpus)
    }
}
