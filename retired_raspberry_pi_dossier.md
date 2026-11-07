# Hardware & OS Dossier: Retired Raspberry Pi 3 Model B+ (`sbc-lyrion` / `10.0.0.10`)

- **Decommissioned Date**: `2026-11-05T16:36:54-05:00` (`WO-SBC-DECOMMISSION-10.0.0.10-015`)
- **Physical Status**: Cleanly synced and powered off (`sudo sync && sudo poweroff`), stripped to a minimal SSH-only headless Debian 13 (`trixie`) `aarch64` baseline.
- **Ready for Repurposing**: Can either be booted immediately as a clean headless Debian 13 `arm64` node (accessible via SSH on port `22/tcp` at `10.0.0.10`) or reflashed with a new microSD image.

---

## 1. Hardware & SoC Specifications

| Component | Specification | Audited Telemetry / Notes |
| :--- | :--- | :--- |
| **Model** | **Raspberry Pi 3 Model B Plus (3B+) Rev 1.3** | Hardware Revision Code `a020d3` (Sony UK manufacturing) |
| **SoC** | **Broadcom BCM2837B0** | 40nm process with metal heat-spreader cap |
| **CPU** | **4x ARM Cortex-A53** (`ARMv8-A`, 64-bit `aarch64`) | **1.40 GHz** max (`600 MHz` min); `38.40 BogoMIPS` per core; governor pinned to **`performance`** (`1.40 GHz`) |
| **Core Voltage** | **`1.3250 V`** | Clean 5V DC micro-USB rail |
| **Power & Throttling Health** | **`throttled=0x0`** (`vcgencmd get_throttled`) | **0** historical undervoltage events, **0** frequency-capping or thermal-throttling events |
| **Thermal Baseline** | **`40.7°C – 41.9°C`** | Well below the `60°C` soft limit and `80°C` hard limit (`bcm2835_thermal`) |
| **GPU** | **Broadcom VideoCore IV** @ 400 MHz (3D @ 300 MHz) | OpenGL ES 2.0, hardware H.264 1080p60 decode / 1080p30 encode (`vc4-drm`) |
| **Memory (RAM)** | **1 GB LPDDR2 SDRAM** (900 MHz) | **906–908 MiB** usable by Linux kernel |
| **Swap** | **200 MiB** `/var/swap` (`dphys-swapfile`) | Configured with `vm.swappiness = 10` |
| **Power Draw** | **~2.5W – 3.5W** idle / **~6.5W – 7.5W** full load | 5.1V DC @ 2.5A Micro-USB B, 5V GPIO (Pins 2 & 4), or 4-pin PoE HAT header |

---

## 2. Storage & Partition Layout (`32 GB` microSDHC)

- **Physical Media**: 32 GB Class 10 microSDHC card (`/dev/mmcblk0`)
- **Audited Block I/O Performance**:
  - **Average Read Await**: `10.19 ms`
  - **Average Write Await**: `159.35 ms`
- **Full Pre-Decommission Backup Archive (External)**:
  - `10.0.0.50:/mnt/storage/lyrion/backups/sbc-lyrion-full-backup-20261105.tar.gz` (`339 MB`)

| Partition / Mount | Mount Point | Filesystem | Total Size | Used | Available | Use % | Mount Options |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `/dev/mmcblk0p1` | `/boot` | `vfat` (FAT32) | `255 MiB` | `34 MiB` | `222 MiB` | `14%` | `rw,relatime` |
| `/dev/mmcblk0p2` | `/` (Root OS) | `ext4` | `29.6 GiB` | `5.0 GiB` | `23.0 GiB` | `18%` | `rw,noatime` |
| `tmpfs` | `/tmp` | `tmpfs` | `454 MiB` | `0 MiB` | `454 MiB` | `0%` | In-RAM temporary storage (reduces SD wear) |
| `tmpfs` | `/dev/shm` | `tmpfs` | `454 MiB` | `0 MiB` | `454 MiB` | `0%` | Shared memory |
| `tmpfs` | `/run` | `tmpfs` | `182 MiB` | `5.5 MiB` | `176 MiB` | `3%` | Runtime state |

---

## 3. Operating System, Kernel Tuning & Credentials

| Property | Value |
| :--- | :--- |
| **Distribution** | **Debian GNU/Linux 13.6 (`trixie`)** `aarch64` |
| **Active Kernel** | `6.12.43+rpt-rpi-v8` (`aarch64` SMP PREEMPT) |
| **Systemd Target** | `multi-user.target` (Headless console, zero GUI/X11/Wayland packages) |
| **Hostname** | `sbc-lyrion` (FQDN: `sbc-lyrion.molinadeveloping.com`) |
| **Primary Admin User** | `console-admin` (`UID=1000`, `GID=1000`, passwordless `sudo` via `/etc/sudoers.d/`) |
| **SSH Access** | Port `22/tcp` (`OpenSSH_10.0p1 Debian-7+deb13u4`), Ed25519/RSA key auth enabled for `console-admin` and `root` |
| **Active Kernel Sysctl Tunings** | `vm.swappiness = 10`, `net.core.rmem_max = 8388608` (8 MB socket receive buffer), `net.ipv4.tcp_congestion_control = bbr` (TCP BBR congestion control) |
| **Installed Runtimes & Tools** | `Python 3.13`, `Perl 5.40`, `Bash 5.2`, `systemd 257`, `NetworkManager`, `ufw`, `sqlite3`, `flac`, `sox`, `faad`, `lame`, `wavpack`, `git`, `curl`, `rsync` |

---

## 4. Network & Wireless Interfaces

| Interface | Controller / Driver | MAC Address | Configuration / Status |
| :--- | :--- | :--- | :--- |
| **`enxb827eb4b7845` (Wired RJ45)** | Microchip **LAN7515** Gigabit Ethernet over internal USB 2.0 High-Speed bus (480 Mbps raw / ~300 Mbps usable; kernel driver `lan78xx`) | **`b8:27:eb:4b:78:45`** | MTU `1500`; `0` FCS/alignment errors (`0.056%` historical TCP retransmission rate). EdgeRouter X (`10.0.0.1`) holds static DHCP reservation **`10.0.0.10/24`** (`sbc-lyrion`) and DNS `sbc-lyrion.molinadeveloping.com -> 10.0.0.10`. |
| **`wlan0` (Wi-Fi)** | Broadcom/Cypress **BCM43455** Dual-Band **802.11b/g/n/ac** (2.4 GHz & 5 GHz, `brcmfmac` driver) | **`b8:27:eb:c8:d2:90`** | Administratively disabled (`wpa_supplicant.service` masked) to eliminate wireless interrupt jitter; can be unmasked for Wi-Fi deployments. |
| **`hci0` (Bluetooth)** | Broadcom/Cypress **BCM43455** **Bluetooth 4.2 / BLE** (over UART `/dev/ttyAMA0`) | `b8:27:eb:e1:d2:f0` | `bluetooth.service` and `hciuart.service` currently masked; can be unmasked for BLE/Bluetooth projects. |

---

## 5. Physical I/O, Buses & Expansion Header

- **40-Pin GPIO Header (J8)**:
  - **Power Pins**: 2x `5V` (Pins 2, 4), 2x `3.3V` (Pins 1, 17), 8x `GND` (Pins 6, 9, 14, 20, 25, 30, 34, 39)
  - **I2C Buses**: `I2C1` (GPIO 2 SDA / Pin 3, GPIO 3 SCL / Pin 5) + `I2C0` (HAT ID EEPROM on Pins 27, 28)
  - **SPI Buses**: `SPI0` (GPIO 7–11 / Pins 19, 21, 23, 24, 26) + `SPI1` auxiliary
  - **UART**: PL011 (`ttyAMA0`) & mini-UART (`ttyS0`) on GPIO 14 TXD (Pin 8) / GPIO 15 RXD (Pin 10)
  - **PWM & PCM/I2S**: Hardware PWM on GPIO 12, 13, 18, 19; PCM/I2S digital audio on GPIO 18–21 (compatible with HiFiBerry / I2S DAC HATs)
- **USB Ports**: 4x USB 2.0 Type-A host ports (shared internal USB 2.0 root hub via LAN7515; max combined USB peripheral current `1.2A`)
- **Video & Audio Outputs**:
  - **HDMI**: Full-size HDMI 1.3a port (up to 1920x1200 @ 60 Hz + HDMI digital audio)
  - **3.5mm TRRS Jack**: Stereo analog audio (`snd_bcm2835` ALSA card) + composite PAL/NTSC video
- **Camera & Display Ribbon Connectors**:
  - **CSI-2**: 15-pin MIPI Camera Serial Interface
  - **DSI**: 15-pin MIPI Display Serial Interface (official 7" touchscreen compatible)
- **PoE Header**: 4-pin 0.1" header near RJ45 jack for 802.3af/at PoE HATs

---

## 6. Final Pre-Shutdown Service & Firewall State (`WO-SBC-DECOMMISSION-10.0.0.10-015`)

### 6.1 Active Network Listeners (`ss -tulpn`)
Only **OpenSSH (`sshd`)** is listening on the network:
- `tcp LISTEN 0.0.0.0:22` (`sshd`)
- `tcp LISTEN [::]:22` (`sshd`)

### 6.2 Host Firewall (`ufw status numbered`)
- **Default Policy**: `deny (incoming)`, `allow (outgoing)`, `disabled (routed)`
- **Active Rules**:
  - `[1] 22/tcp ALLOW IN Anywhere`
  - `[2] 22/tcp (v6) ALLOW IN Anywhere (v6)`

### 6.3 Masked Superfluous Services (Baseline Snapshot in `/root/pre-decommission-10.0.0.10-20261105/`)
All non-essential services were stopped, disabled, and masked prior to shutdown so the board boots cleanly into an SSH-only state (`0` failed units):
- **Application Services Masked**: `lyrionmusicserver.service`, `nginx.service`, `lyrion-backup.service`, `lyrion-backup.timer`
- **Network Discovery / Wireless Services Masked**: `avahi-daemon.service`, `avahi-daemon.socket`, `wpa_supplicant.service`, `ModemManager.service`, `bluetooth.service`, `hciuart.service`
- **Peripheral / Print / RPC / Time Services Masked**: `triggerhappy.service`, `triggerhappy.socket`, `cups.service`, `cups.socket`, `cups.path`, `rpcbind.service`, `rpcbind.socket`, `nfs-blkmap.service`, `ntpsec.service`, `ntpsec-systemd-netif.path`, `ntpsec-rotate-stats.timer`

---

## 7. Quick-Start Commands for Your New Project

If you boot this Raspberry Pi as-is (without reflashing the microSD card), it will come online at `10.0.0.10` (`console-admin@10.0.0.10`) running Debian 13 (`trixie`) `arm64` with only SSH active.

### A. Purge Dormant Lyrion/NGINX Files (Reclaims ~1.2 GB on `/`)
```bash
sudo systemctl unmask lyrionmusicserver.service nginx.service
sudo apt-get purge -y lyrionmusicserver nginx nginx-common nginx-light
sudo apt-get autoremove --purge -y
sudo rm -rf /var/lib/squeezeboxserver /etc/nginx /etc/ssl/molinadeveloping.com \
            /usr/local/bin/lyrion-maintenance-backup.sh \
            /etc/systemd/system/lyrion-backup.service \
            /etc/systemd/system/lyrion-backup.timer \
            /home/console-admin/media
sudo systemctl daemon-reload
```

### B. Re-Enable NTP Time Sync, Wi-Fi, Bluetooth, or mDNS (`.local`) If Needed
```bash
# Re-enable NTP clock synchronization:
sudo systemctl unmask ntpsec.service ntpsec-systemd-netif.path ntpsec-rotate-stats.timer
sudo systemctl enable --now ntpsec.service ntpsec-rotate-stats.timer

# Re-enable Wi-Fi (wlan0 — MAC b8:27:eb:c8:d2:90):
sudo systemctl unmask wpa_supplicant.service
sudo systemctl enable --now wpa_supplicant.service

# Re-enable Bluetooth / BLE (hci0):
sudo systemctl unmask hciuart.service bluetooth.service
sudo systemctl enable --now hciuart.service bluetooth.service

# Re-enable mDNS (<hostname>.local advertisement):
sudo systemctl unmask avahi-daemon.socket avahi-daemon.service
sudo systemctl enable --now avahi-daemon.socket avahi-daemon.service
```

### C. Rename the Host for the New Product
```bash
sudo hostnamectl set-hostname <new-product-hostname>
```
