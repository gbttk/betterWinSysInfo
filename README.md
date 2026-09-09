![yeah I didn't change the project name](https://i.postimg.cc/SQ53Ysjm/New-Project.png)
**betterWinSysInfo** is a native command-line utility built in Rust, designed exclusively for the Microsoft Windows ecosystem. Its core objective is to perform deep hardware and software auditing, gathering detailed telemetry from critical components like the CPU, GPU, Motherboard, BIOS, RAM topology, and storage subsystem. It also features a heuristic engine to estimate peak power consumption and recommend power supply (PSU) wattage.

## Key Features:

* **Memory Topology:** Identifies total capacity, physical slots provided by the motherboard, occupied slots, active frequency, and SMBIOS memory type decoding.
* **Processor & Motherboard:** Extracts precise commercial models, physical cores, logical threads, cache sizes, BIOS vendor, and firmware release dates.
* **Graphics & Storage:** Catalogs dedicated (dGPU) and integrated (iGPU) video adapters, VRAM capacity, physical disks, and logical volume mapping.
* **Energy Diagnostics:** A heuristic engine that analyzes the CPU and GPU combination to calculate estimated peak wattage and recommend an ideal power supply unit with a 30% safety margin.
* **Output Formats:** ANSI/UTF-8 formatted tables in the terminal or structured JSON export for automated infrastructure pipelines.

## Architectural Choices:

* **Why Rust:** We chose Rust to generate a single, statically linked binary of approximately 1.2 MB. Unlike interpreted languages like Python that require heavy runtimes and take seconds to load, **bWinSysInfo** initializes in milliseconds. It relies on RAII for deterministic memory usage and the Option type to guarantee that missing WMI fields do not cause the application to crash.
* **Safe WMI Integration:** To extract deep hardware information without developing a kernel-mode driver, the architecture connects to the Windows Management Instrumentation (WMI) via the Component Object Model (COM). This allows the tool to run entirely in user-mode, avoiding security risks associated with vulnerable drivers in corporate environments.
* **Decoupled Structure:** The codebase is strictly divided into logical layers. The hardware telemetry module is completely independent of the UI layer. This separation ensures that the telemetry logic can be reused in the future if a graphical interface is introduced.

## Compatibility:

* Windows 10 and Windows 11 (all editions)
* Windows Server 2016, 2019, and 2022
* Hardware architectures: x86_64 and ARM64

## Usage:

* winsysinfo --summary : Displays a condensed system summary.
* winsysinfo --ram : Details memory module topology and occupation.
* winsysinfo --all : Executes the unrestricted report for all hardware components.
* winsysinfo --export report.json : Generates a structured JSON file for inventory integration.

Double-clicking the executable detects interactive execution automatically, generates the full report, and waits for the user to press ENTER before closing.

## Roadmap:
- [ ] A roadmap!
