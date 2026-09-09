mod hardware;
mod ui;

use clap::Parser;
use hardware::SystemReport;
use std::fs::File;
use std::io::Write;
use ui::TablePrinter;

#[derive(Parser, Debug)]
#[command(
    name = "betterWinSysInfo",
    author = "gbttk",
    version = "0.1.0",
    about = "Hardware audit and complete diagnostics for Windows (CPU, GPU, Motherboard, RAM, Disks, Power)"
)]
struct Cli {
    /// show all detailed hardware sections
    #[arg(short, long)]
    all: bool,

    /// show only system executive summary
    #[arg(short, long)]
    summary: bool,

    /// show in-depth cpu details
    #[arg(long)]
    cpu: bool,

    /// show RAM topology (frequency, used/total slots, sticks)
    #[arg(short, long)]
    ram: bool,

    /// show video gpu and monitor details
    #[arg(short, long)]
    gpu: bool,

    /// show motherboard and BIOS details
    #[arg(short, long)]
    motherboard: bool,

    /// show physical and logical storage units
    #[arg(short = 'd', long)]
    storage: bool,

    /// show electrical consumption diagnostics, TDP...
    #[arg(short, long)]
    power: bool,

    /// export output in pure json format to terminal
    #[arg(short, long)]
    json: bool,

    /// save complete structured report to a JSON file
    #[arg(short, long, value_name = "FILE_PATH")]
    export: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // gather telemetry from WMI (usually takes ~1 sec on cold start)
    println!("Collecting Windows system telemetry information via WMI/CIM...");
    let report = match SystemReport::collect() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Critical error collecting system data: {e}");
            std::process::exit(1);
        }
    };

    // if solicited json output to file
    if let Some(ref path) = cli.export {
        match serde_json::to_string_pretty(&report) {
            Ok(json_str) => match File::create(path).and_then(|mut f| f.write_all(json_str.as_bytes())) {
                Ok(_) => println!("Report successfully exported to: {}", path),
                Err(e) => eprintln!("Error saving file to {}: {}", path, e),
            },
            Err(e) => eprintln!("Error serializing report to JSON: {}", e),
        }
    }

    // if the user already solicited json output in standard output
    if cli.json {
        match serde_json::to_string_pretty(&report) {
            Ok(json_str) => println!("{json_str}"),
            Err(e) => eprintln!("Error converting to JSON: {e}"),
        }
        return;
    }

    // if no flag (like when double clicking the .exe)
    let has_specific_flag = cli.all
        || cli.summary
        || cli.cpu
        || cli.ram
        || cli.gpu
        || cli.motherboard
        || cli.storage
        || cli.power;

    if !has_specific_flag {
        // Friendly mode for double-click: display full report and wait for ENTER
        TablePrinter::print_all(&report);
        println!("\n=======================================================");
        println!(" Hardware diagnostics completed successfully :)");
        println!(" Press [ENTER] to close this window...");
        println!("=======================================================");
        let mut pause_buf = String::new();
        let _ = std::io::stdin().read_line(&mut pause_buf);
        return;
    }

    // otherwise, print only the sections requested via CLI flags
    if cli.all {
        TablePrinter::print_all(&report);
        return;
    }

    if cli.summary {
        TablePrinter::print_summary(&report);
    }

    if cli.motherboard {
        TablePrinter::print_motherboard(&report);
    }

    if cli.cpu {
        TablePrinter::print_cpu(&report);
    }

    if cli.ram {
        TablePrinter::print_memory(&report);
    }

    if cli.gpu {
        TablePrinter::print_gpu(&report);
    }

    if cli.storage {
        TablePrinter::print_storage(&report);
    }

    if cli.power {
        TablePrinter::print_power(&report);
    }
}
