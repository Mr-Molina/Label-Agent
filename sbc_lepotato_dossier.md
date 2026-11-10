# Hardware & OS Dossier: Libre Computer AML-S905X-CC ("Le Potato") (`sbc-lepotato` / `10.0.0.68`)

- **Audit Date**: `2026-11-09T19:27:00-05:00` (`WO-019`)
- **Status**: **ONLINE & ACTIVE** (Accessible via SSH on port `22/tcp` at `10.0.0.68` / `sbc-lepotato.home.local`)
- **Target Role**: Dedicated Edge Appliance Host for **Label-Agent** (pure-Rust voice-driven inventory & labeling appliance)
- **Primary Admin User**: `console-admin` (`uid=1000`, `gid=1000`, passwordless `sudo`)

---

## 1. Hardware & SoC Specifications

| Component | Specification | Live Audited Ground Truth & Notes |
| :--- | :--- | :--- |
| **Model** | **Libre Computer AML-S905X-CC ("Le Potato")** | Board identity: `aml-s905x-cc`, form-factor compatible with Raspberry Pi 3 |
| **SoC** | **Amlogic S905X (Meson GXL)** | 28nm HPC process, high-efficiency quad-core application processor |
| **CPU** | **4x ARM Cortex-A53** (`ARMv8-A`, 64-bit `aarch64`) | **1.416 GHz** max (`100 MHz` min); `48.00 BogoMIPS` per core; `scaling MHz: 71%` |
| **CPU Flags** | `fp asimd evtstrm aes pmull sha1 sha2 crc32 cpuid` | Hardware cryptographic acceleration (AES, SHA1, SHA2, PMULL) & NEON SIMD |
| **Vulnerabilities** | Meltdown, Spectre v2, Retbleed, MDS, etc.: **Not affected** | Hardware-level immunity to speculative execution flaws (Spectre v1 mitigated) |
| **Thermal Baseline** | **`49.0°C`** (`/sys/class/thermal/thermal_zone0/temp: 49000`) | Stable thermal margin under passive heatsink |
| **Memory (RAM)** | **2 GB DDR3/DDR4 SDRAM** | **`1.9 GiB` total** (`234 MiB` used, `1.3 GiB` free, `1.7 GiB` available) |
| **Swap** | **`969 MiB` active zram swap** (`/dev/zram0`) | High-speed in-RAM compressed swap (`0B` used, zero flash write amplification) |
| **Power Input** | **5V DC @ 2A – 2.5A Micro-USB** or 5V GPIO | Compatible with standard 5V power supply rails |

---

## 2. Storage & Filesystem Architecture

The appliance boots from a high-speed eMMC storage module (`/dev/mmcblk0`) with dedicated hardware boot partitions, offering superior I/O throughput and endurance compared to standard microSD cards:

- **Storage Device**: `/dev/mmcblk0` (`14.6 GiB` total)
- **Hardware Boot Partitions**: `/dev/mmcblk0boot0` (`4 MiB`), `/dev/mmcblk0boot1` (`4 MiB`)
- **Swap Device**: `/dev/zram0` (`969.7 MiB`, compressed RAM swap)

| Partition / Mount | Mount Point | Filesystem | Size | Used | Avail | Use % | Operational Notes |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `/dev/mmcblk0p1` | `/boot/efi` | `vfat` | `510 MiB` | `5.5 MiB` | `505 MiB` | `2%` | EFI System Partition (`ESP`) |
| `/dev/mmcblk0p2` | `/` (Root OS) | `btrfs` | `15 GiB` | `1.1 GiB` | `13 GiB` | `8%` | Btrfs root filesystem with snapshot capability |
| `zram0` | `[SWAP]` | `swap` | `969.7 MiB` | `0 B` | `969.7 MiB` | `0%` | Fast compressed in-memory swap |
| `tmpfs` | `/tmp` | `tmpfs` | In-RAM | `0 B` | Available | `0%` | Zero-flash-wear temporary spooling for label renders |

---

## 3. Operating System, Kernel & Userland Runtimes

| Property | Value / Status |
| :--- | :--- |
| **Operating System** | **Debian GNU/Linux 13.7 (`trixie`)** `aarch64` |
| **Kernel Version** | `6.12.73-g6b2d05eda916` (`#1 SMP PREEMPT_DYNAMIC Tue Feb 17 16:14:53 UTC 2026`) |
| **System Architecture** | `aarch64` (64-bit ARM, Little Endian) |
| **Hostname** | `sbc-lepotato.home.local` (short: `sbc-lepotato`) |
| **User Account** | `console-admin` (`UID=1000`, `GID=1000`) with passwordless `sudo` |
| **User Group Memberships**| `console-admin`, `adm`, `dialout`, `cdrom`, `sudo`, `audio`, `video`, `plugdev`, `games`, `users`, `input`, `render`, `netdev`, `lpadmin`, `gpio`, `i2c`, `spi`, `docker` |
| **Time Synchronization** | **`systemd-timesyncd` active & synchronized** (`RTC time: Tue 2026-11-10 00:27:25 UTC`) |
| **Python Runtime** | `Python 3.13.5` installed |
| **Git Tooling** | `git version 2.47.3` installed |
| **Init / Supervisor** | `systemd 257` (`systemd-timesyncd` active; `ntpsec` inactive; `cups` inactive) |

---

## 4. Network & Connectivity Profile

| Parameter | Live Telemetry & Configuration |
| :--- | :--- |
| **Wired Interface (`eth0`)** | Native Fast Ethernet controller (`100 Mbps` full-duplex) |
| **MAC Address** | `06:35:7d:f7:8d:b8` |
| **IPv4 Address** | **`10.0.0.68/24`** (Static DHCP reservation on local network gateway) |
| **IPv6 Addresses** | Global dynamic: `fd9d:2f57:def5:b9c3:c921:e433:f21e:ed2b/64`<br/>Link-local: `fe80::fa13:ab7f:defd:1901/64` |
| **Network Manager** | `NetworkManager.service` (active) + `networking.service` (active) |
| **Host Firewall (`ufw`)** | **`active`**; Inbound ports allowed: `22/tcp` (IPv4 & IPv6). Outbound traffic permitted. |

---

## 5. Bus & Peripheral Topology

### 5.1 USB Controller & Ports
- **Root Controller**: Linux Foundation 2.0 root hub (`1d6b:0002`)
- **Internal Hub**: Genesys Logic, Inc. Hub (`05e3:0610`) powering 4x physical USB 2.0 Type-A ports
- **Connected Peripherals (Baseline)**: None currently attached. The board is ready for:
  - **Dymo LabelWriter 450** (`0922:0020` USB VID:PID)
  - **USB Microphone / Audio Capture Dongle** (16 kHz 16-bit mono PCM)

### 5.2 Audio Subsystem
- **Current State**: `/proc/asound/cards` reports `--- no soundcards ---`.
- **ALSA Utilities**: `alsa-utils` (`arecord`, `aplay`) is not yet installed.
- **Required Action**: `sudo apt install -y alsa-utils libasound2` upon attaching USB microphone.

### 5.3 Printing Subsystem
- **Current State**: `cups` and `lpstat` are not currently installed (`inactive`).
- **Required Action**: `sudo apt install -y cups cups-client printer-driver-dymo` and enable `cups.socket` / `cups.service` when provisioning for hardware printing.

---

## 6. Comparison: Le Potato (`AML-S905X-CC`) vs. Retired Pi 3B+

| Dimension | Retired Raspberry Pi 3B+ | Libre Computer Le Potato (`AML-S905X-CC`) | Advantage |
| :--- | :--- | :--- | :--- |
| **System Memory** | 1.0 GB LPDDR2 (908 MiB usable) | **2.0 GB DDR3/DDR4 (1.9 GiB usable)** | **+100% RAM headroom** (ample space for label-agent + render cache) |
| **Storage Medium** | 32 GB microSDHC (high write await ~159 ms) | **16 GB eMMC (hardware boot0/boot1)** | **Significantly faster I/O await & wear resistance** |
| **Swap Strategy** | 200 MiB disk swapfile (`dphys-swapfile`) | **969 MiB compressed `zram` swap in RAM** | **Zero disk wear & microsecond swap speeds** |
| **Root Filesystem**| `ext4` | **`btrfs` on `/` (13 GiB free / 8% used)** | Modern snapshotting & data integrity |
| **CPU Architecture**| 4x Cortex-A53 @ 1.40 GHz (BCM2837B0) | **4x Cortex-A53 @ 1.416 GHz (S905X)** | Crypto extensions (`aes`, `sha1`, `sha2`, `pmull`) |
| **Ethernet Bus** | USB 2.0 attached Microchip LAN7515 | **Dedicated Native Ethernet Controller** | Isolated from USB bus bandwidth |
| **OS Distribution**| Debian 13.6 (`trixie`) `aarch64` | **Debian 13.7 (`trixie`) `aarch64`** | Fresh upstream Debian 13.7 baseline |
| **Time Sync** | `ntpsec` | **`systemd-timesyncd` (native, active)** | Lightweight, already synchronized out of the box |
