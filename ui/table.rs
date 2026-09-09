use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use crate::hardware::SystemReport;

/// CLI table formatter using comfy-table
pub struct TablePrinter;

impl TablePrinter {
    /// summary
    pub fn print_summary(report: &SystemReport) {
        println!("\n=======================================================");
        println!("             WINSYSINFO - SYSTEM SUMMARY            ");
        println!("=======================================================");

        let mut table = Table::new();
        table.load_style(UTF8_FULL);
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Component").fg(Color::Cyan),
            Cell::new("Main Details").fg(Color::Green),
        ]);

        table.add_row(vec![
            Cell::new("Operating System"),
            Cell::new(format!("{} (Build {}) [{}]", report.os.os_name, report.os.build_number, report.os.architecture)),
        ]);

        table.add_row(vec![
            Cell::new("Computer / Manufacturer"),
            Cell::new(format!("{} - Model: {}", report.os.computer_manufacturer, report.os.computer_model)),
        ]);

        table.add_row(vec![
            Cell::new("Uptime"),
            Cell::new(&report.os.uptime_formatted),
        ]);

        table.add_row(vec![
            Cell::new("Motherboard"),
            Cell::new(format!("{} {} (BIOS: {})", report.motherboard.manufacturer, report.motherboard.product, report.motherboard.bios_version)),
        ]);

        if let Some(cpu) = report.cpus.first() {
            table.add_row(vec![
                Cell::new("Processor (CPU)"),
                Cell::new(format!("{} ({} Cores / {} Threads @ {} MHz)", cpu.model, cpu.physical_cores, cpu.logical_threads, cpu.max_clock_mhz)),
            ]);
        }

        table.add_row(vec![
            Cell::new("RAM Memory"),
            Cell::new(format!("{:.1} GB installed | {} of {} slots used | {}", 
                report.memory.total_installed_gb, report.memory.slots_used, report.memory.total_slots, report.memory.channel_mode)),
        ]);

        // list each GPU adapter (handles laptops with both iGPU and dGPU)
        for (idx, gpu) in report.gpus.iter().enumerate() {
            let label = if idx == 0 { "Video Card (GPU)" } else { "Secondary GPU" };
            table.add_row(vec![
                Cell::new(label),
                Cell::new(format!("{} ({:.1} GB VRAM) - Resolution: {}", gpu.name, gpu.vram_gb, gpu.resolution)),
            ]);
        }

        table.add_row(vec![
            Cell::new("Storage"),
            Cell::new(format!("{} physical disk(s) | {} logical volume(s)", report.storage.physical_disks.len(), report.storage.logical_volumes.len())),
        ]);

        table.add_row(vec![
            Cell::new("Power Consumption & Recommended PSU"),
            Cell::new(format!("Estimated Peak: ~{}W | Recommended PSU: {}W 80-Plus ({})", 
                report.power.estimated_peak_system_draw_w, report.power.recommended_psu_wattage_w, report.power.profile_category)),
        ]);

        println!("{table}");
    }

    /// BIOS and motherboard
    pub fn print_motherboard(report: &SystemReport) {
        println!("\n=======================================================");
        println!("             MOTHERBOARD AND BIOS                          ");
        println!("=======================================================");

        let mut table = Table::new();
        table.load_style(UTF8_FULL);
        table.set_header(vec![
            Cell::new("Property").fg(Color::Cyan),
            Cell::new("Value").fg(Color::Yellow),
        ]);

        table.add_row(vec![Cell::new("Motherboard Manufacturer"), Cell::new(&report.motherboard.manufacturer)]);
        table.add_row(vec![Cell::new("Model / Product"), Cell::new(&report.motherboard.product)]);
        table.add_row(vec![Cell::new("Version / Revision"), Cell::new(&report.motherboard.version)]);
        table.add_row(vec![Cell::new("Serial Number"), Cell::new(&report.motherboard.serial_number)]);
        table.add_row(vec![Cell::new("BIOS Vendor"), Cell::new(&report.motherboard.bios_vendor)]);
        table.add_row(vec![Cell::new("BIOS Version"), Cell::new(&report.motherboard.bios_version)]);
        table.add_row(vec![Cell::new("BIOS Release Date"), Cell::new(&report.motherboard.bios_date)]);

        println!("{table}");
    }

    /// CPU
    pub fn print_cpu(report: &SystemReport) {
        println!("\n=======================================================");
        println!("             PROCESSOR (CPU)                         ");
        println!("=======================================================");

        // each physical CPU socket gets its own specification table
        for (idx, cpu) in report.cpus.iter().enumerate() {
            let mut table = Table::new();
            table.load_style(UTF8_FULL);
            table.set_header(vec![
                Cell::new(format!("CPU #{}", idx + 1)).fg(Color::Cyan),
                Cell::new("Specification").fg(Color::Yellow),
            ]);

            table.add_row(vec![Cell::new("Model"), Cell::new(&cpu.model)]);
            table.add_row(vec![Cell::new("Manufacturer"), Cell::new(&cpu.manufacturer)]);
            table.add_row(vec![Cell::new("Architecture"), Cell::new(&cpu.architecture)]);
            table.add_row(vec![Cell::new("Physical Cores"), Cell::new(cpu.physical_cores.to_string())]);
            table.add_row(vec![Cell::new("Logical Processors (Threads)"), Cell::new(cpu.logical_threads.to_string())]);
            table.add_row(vec![Cell::new("Max Clock"), Cell::new(format!("{} MHz ({:.2} GHz)", cpu.max_clock_mhz, cpu.max_clock_mhz as f64 / 1000.0))]);
            
            if let Some(curr) = cpu.current_clock_mhz {
                table.add_row(vec![Cell::new("Current Clock"), Cell::new(format!("{} MHz", curr))]);
            }

            table.add_row(vec![Cell::new("Socket"), Cell::new(&cpu.socket)]);

            if let Some(l2) = cpu.l2_cache_kb {
                table.add_row(vec![Cell::new("Cache L2"), Cell::new(format!("{} KB", l2))]);
            }
            if let Some(l3) = cpu.l3_cache_kb {
                table.add_row(vec![Cell::new("Cache L3"), Cell::new(format!("{} KB ({:.1} MB)", l3, l3 as f64 / 1024.0))]);
            }
            if let Some(load) = cpu.load_percentage {
                table.add_row(vec![Cell::new("Current CPU Usage"), Cell::new(format!("{}%", load))]);
            }

            println!("{table}");
        }
    }

    /// RAM memory
    pub fn print_memory(report: &SystemReport) {
        println!("\n=======================================================");
        println!("             RAM MEMORY AND SLOT TOPOLOGY          ");
        println!("=======================================================");

        println!(
            "Total Installed: {:.1} GB | Capacity In Use: {} of {} slots used ({} available) | Topology: {}",
            report.memory.total_installed_gb,
            report.memory.slots_used,
            report.memory.total_slots,
            report.memory.slots_available,
            report.memory.channel_mode
        );

        let mut table = Table::new();
        table.load_style(UTF8_FULL);
        table.set_header(vec![
            Cell::new("Slot").fg(Color::Cyan),
            Cell::new("Bank").fg(Color::Cyan),
            Cell::new("Capacity").fg(Color::Green),
            Cell::new("Active Freq.").fg(Color::Yellow),
            Cell::new("Rated Freq.").fg(Color::Yellow),
            Cell::new("Type").fg(Color::Magenta),
            Cell::new("Form Factor"),
            Cell::new("Manufacturer"),
            Cell::new("Part Number"),
        ]);

        // list each physical DIMM stick
        for stick in &report.memory.sticks {
            table.add_row(vec![
                Cell::new(&stick.slot),
                Cell::new(&stick.bank),
                Cell::new(format!("{:.1} GB", stick.capacity_gb)),
                Cell::new(format!("{} MHz", stick.configured_frequency_mhz)),
                Cell::new(format!("{} MHz", stick.rated_speed_mhz)),
                Cell::new(&stick.memory_type),
                Cell::new(&stick.form_factor),
                Cell::new(&stick.manufacturer),
                Cell::new(&stick.part_number),
            ]);
        }

        println!("{table}");
    }

    /// GPU
    pub fn print_gpu(report: &SystemReport) {
        println!("\n=======================================================");
        println!("             VIDEO CARD (GPU)                      ");
        println!("=======================================================");

        // one table per detected GPU adapter
        for (idx, gpu) in report.gpus.iter().enumerate() {
            let mut table = Table::new();
            table.load_style(UTF8_FULL);
            table.set_header(vec![
                Cell::new(format!("Video Adapter #{}", idx + 1)).fg(Color::Cyan),
                Cell::new("Value").fg(Color::Yellow),
            ]);

            table.add_row(vec![Cell::new("Device Name"), Cell::new(&gpu.name)]);
            table.add_row(vec![
                Cell::new("Type"), 
                Cell::new(if gpu.is_dedicated { "Dedicated (dGPU)" } else { "Integrated (iGPU)" })
            ]);
            table.add_row(vec![Cell::new("Video Memory (VRAM)"), Cell::new(format!("{:.1} GB ({} MB)", gpu.vram_gb, gpu.vram_mb))]);
            table.add_row(vec![Cell::new("Driver Version"), Cell::new(&gpu.driver_version)]);
            table.add_row(vec![Cell::new("Driver Date"), Cell::new(&gpu.driver_date)]);
            table.add_row(vec![Cell::new("Video Processor"), Cell::new(&gpu.video_processor)]);
            table.add_row(vec![Cell::new("Monitor Resolution"), Cell::new(&gpu.resolution)]);

            if let Some(hz) = gpu.refresh_rate_hz {
                table.add_row(vec![Cell::new("Refresh Rate"), Cell::new(format!("{} Hz", hz))]);
            }

            println!("{table}");
        }
    }

    /// storage
    pub fn print_storage(report: &SystemReport) {
        println!("\n=======================================================");
        println!("             STORAGE DISKS AND VOLUMES         ");
        println!("=======================================================");

        // 1. physical drive hardware
        println!("\n[Detected Physical Disks]");
        let mut disk_table = Table::new();
        disk_table.load_style(UTF8_FULL);
        disk_table.set_header(vec![
            Cell::new("Model").fg(Color::Cyan),
            Cell::new("Capacity").fg(Color::Green),
            Cell::new("Interface").fg(Color::Yellow),
            Cell::new("Media Type"),
            Cell::new("Partitions"),
        ]);

        for d in &report.storage.physical_disks {
            disk_table.add_row(vec![
                Cell::new(&d.model),
                Cell::new(format!("{:.1} GB", d.capacity_gb)),
                Cell::new(&d.interface),
                Cell::new(&d.media_type),
                Cell::new(d.partitions.to_string()),
            ]);
        }
        println!("{disk_table}");

        // 2. logical filesystem volumes
        println!("\n[Logical Volumes and Partitions]");
        let mut vol_table = Table::new();
        vol_table.load_style(UTF8_FULL);
        vol_table.set_header(vec![
            Cell::new("Letter").fg(Color::Cyan),
            Cell::new("Label"),
            Cell::new("System"),
            Cell::new("Total Capacity").fg(Color::Green),
            Cell::new("Used Space").fg(Color::Yellow),
            Cell::new("Free Space").fg(Color::Green),
            Cell::new("% Used").fg(Color::Red),
        ]);

        for v in &report.storage.logical_volumes {
            vol_table.add_row(vec![
                Cell::new(&v.drive_letter),
                Cell::new(&v.label),
                Cell::new(&v.file_system),
                Cell::new(format!("{:.1} GB", v.total_gb)),
                Cell::new(format!("{:.1} GB", v.used_gb)),
                Cell::new(format!("{:.1} GB", v.free_gb)),
                Cell::new(format!("{:.1}%", v.percent_used)),
            ]);
        }
        println!("{vol_table}");
    }

    /// power
    pub fn print_power(report: &SystemReport) {
        println!("\n=======================================================");
        println!("             POWER DIAGNOSTICS AND ESTIMATE       ");
        println!("=======================================================");

        // summary of estimated wattages and recommended PSU
        let mut table = Table::new();
        table.load_style(UTF8_FULL);
        table.set_header(vec![
            Cell::new("Diagnostic Parameter").fg(Color::Cyan),
            Cell::new("Estimate / Status").fg(Color::Yellow),
        ]);

        table.add_row(vec![
            Cell::new("Estimated Processor (CPU) TDP"),
            Cell::new(format!("~{} Watts", report.power.estimated_cpu_tdp_w)),
        ]);

        table.add_row(vec![
            Cell::new("Estimated Graphics (GPU) TDP"),
            Cell::new(format!("~{} Watts", report.power.estimated_gpu_tdp_w)),
        ]);

        table.add_row(vec![
            Cell::new("Platform Consumption (Motherboard/RAM/Disks/Fans)"),
            Cell::new(format!("~{} Watts", report.power.estimated_platform_w)),
        ]);

        table.add_row(vec![
            Cell::new("Estimated Peak Total Consumption"),
            Cell::new(format!("~{} Watts", report.power.estimated_peak_system_draw_w)),
        ]);

        table.add_row(vec![
            Cell::new("Recommended Power Supply Unit (PSU)"),
            Cell::new(format!("{}W 80-Plus (with 30% safety margin against peaks)", report.power.recommended_psu_wattage_w)),
        ]);

        table.add_row(vec![
            Cell::new("System Classification"),
            Cell::new(&report.power.profile_category),
        ]);

        // check if laptop battery is present
        if report.power.battery.is_present {
            let charge = report.power.battery.charge_percent.map(|c| format!("{}%", c)).unwrap_or_else(|| "N/A".to_string());
            table.add_row(vec![
                Cell::new("Battery Status (Laptop)"),
                Cell::new(format!("Charge: {} | Status: {}", charge, report.power.battery.status)),
            ]);
        } else {
            table.add_row(vec![
                Cell::new("Battery Status"),
                Cell::new("No battery installed (Direct Desktop Power)"),
            ]);
        }

        // thermal readout if ACPI thermal zones were exposed by BIOS
        if !report.power.thermal_sensors.is_empty() {
            for s in &report.power.thermal_sensors {
                table.add_row(vec![
                    Cell::new(format!("Temperature [{}]", s.name)),
                    Cell::new(format!("{:.1} °C", s.temperature_celsius)),
                ]);
            }
        } else {
            table.add_row(vec![
                Cell::new("Thermal Sensors"),
                Cell::new(&report.power.thermal_status_note),
            ]);
        }

        println!("{table}");
    }

    /// print all
    pub fn print_all(report: &SystemReport) {
        Self::print_summary(report);
        Self::print_motherboard(report);
        Self::print_cpu(report);
        Self::print_memory(report);
        Self::print_gpu(report);
        Self::print_storage(report);
        Self::print_power(report);
    }
}
