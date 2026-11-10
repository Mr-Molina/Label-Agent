# Libre Computer Le Potato (AML-S905X-CC) Deployment Runbook: `label-agent`

## 1. Node Profile & Hardware Ground Truth
Reference: [`sbc_lepotato_dossier.md`](/home/console-admin/Github/Label-Agent/sbc_lepotato_dossier.md)

| Attribute | Specification | Operational Constraint / Deployment Notes |
| :--- | :--- | :--- |
| **Hardware Model** | **Libre Computer AML-S905X-CC ("Le Potato")** | Board revision `aml-s905x-cc`, 4x ARM Cortex-A53 (`aarch64`) @ 1.416 GHz |
| **RAM / Swap** | **2.0 GB DDR3/DDR4 (1.9 GiB usable)** / **969 MiB zram** | `MemoryMax=256M`, `MemoryHigh=192M` enforced in systemd unit |
| **Storage** | **16 GB eMMC Module (`/dev/mmcblk0`)** | Root `/dev/mmcblk0p2` on `btrfs` (`13 GiB` free, 8% used); hardware `boot0`/`boot1` |
| **Spooling RAM Disk** | `tmpfs` mounted at `/tmp` | All QR code rendering & label compilation in RAM (`PrivateTmp=no`) |
| **Operating System**| **Debian GNU/Linux 13.7 (`trixie`) `aarch64`** | Kernel `6.12.73-g6b2d05eda916`, headless console target |
| **Network IP** | **`10.0.0.68/24` on `eth0`** | Static DHCP reservation on LAN router (`sbc-lepotato.home.local`) |
| **Hardware Audio** | USB Microphone / ALSA capture | User `console-admin` already in `audio` (`gid=29`) |
| **Hardware Printing** | Dymo LabelWriter 450 (USB) / CUPS spooler | User `console-admin` already in `lpadmin` (`gid=115`) |
| **Clock Synchronization** | `systemd-timesyncd` (active & synchronized) | Validated UTC clock for HTTPS/TLS certificate verification |

---

## 2. Pre-Deployment Provisioning Steps

All steps below are executed via SSH on `console-admin@10.0.0.68`:

### Step 1: Package Installation (CUPS, ALSA Utilities & Printer Drivers)
Install the required print server and audio capture utilities:

```bash
sudo apt update
sudo apt install -y cups cups-client printer-driver-dymo alsa-utils libasound2
```

---

### Step 2: Configure and Start the CUPS Printing Service
Enable and start the CUPS service and socket:

```bash
# Enable and start CUPS
sudo systemctl enable --now cups.socket cups.service

# Verify service is active
systemctl is-active cups.service cups.socket

# Verify scheduler response
lpstat -r
```

When the Dymo LabelWriter 450 is connected via USB:
```bash
# Verify USB enumeration (ID 0922:0020 DYMO Corporation)
lsusb | grep -i -E "dymo|0922"

# Verify detected CUPS printer queue
lpstat -p -d
```

---

### Step 3: Verify Time Synchronization (`systemd-timesyncd`)
Clock synchronization is mandatory for HTTPS TLS handshake validation with `Inventory-Server`:

```bash
# Check synchronization status
timedatectl timesync-status
timedatectl status
```
Ensure `System clock synchronized: yes` and `NTP service: active`.

---

### Step 4: Verify Audio Subsystem
When the USB microphone is attached to one of the 4 USB ports:

```bash
# List ALSA recording devices
arecord -l

# Test a 2-second recording capture
arecord -D default -f S16_LE -r 16000 -c 1 -d 2 /tmp/test-mic.wav
ls -l /tmp/test-mic.wav && rm -f /tmp/test-mic.wav
```

---

### Step 5: Cross-Compile and Deploy Appliance Binary

On the development workstation (`console-admin@workstation`):

1. **Cross-Compile for ARM64 (`aarch64-unknown-linux-gnu`)**:
   ```bash
   cd /home/console-admin/Github/Label-Agent
   rustup target add aarch64-unknown-linux-gnu
   cargo build --release --target aarch64-unknown-linux-gnu --bin label-agent
   ```

2. **Deploy Executable to Le Potato (`10.0.0.68`)**:
   ```bash
   scp target/aarch64-unknown-linux-gnu/release/label-agent console-admin@10.0.0.68:/tmp/label-agent.new
   ssh console-admin@10.0.0.68 "sudo install -m 0755 /tmp/label-agent.new /usr/local/bin/label-agent && rm /tmp/label-agent.new"
   ```

3. **Install and Enable Systemd Service Unit**:
   ```bash
   scp deploy/systemd/label-agent.service console-admin@10.0.0.68:/tmp/label-agent.service
   ssh console-admin@10.0.0.68 "sudo install -m 0644 /tmp/label-agent.service /etc/systemd/system/label-agent.service && \
     sudo systemctl daemon-reload && \
     sudo systemctl enable --now label-agent.service"
   ```

---

## 3. Post-Deployment Verification Checklist

Verify live operational health on `sbc-lepotato` (`10.0.0.68`):

- [ ] **Daemon Process**: `systemctl status label-agent.service` reports `active (running)`.
- [ ] **Memory Footprint**: `systemctl show label-agent.service -p MemoryCurrent` reports `< 25M` (well below `MemoryHigh=192M` and `MemoryMax=256M`).
- [ ] **RAM Spool Directory**: `/tmp/label-agent` directory exists and mounts under `tmpfs` for zero-wear disk writes.
- [ ] **CUPS Spooler Operational**: `lpstat -r` returns `scheduler is running`.
- [ ] **Printer Detection**: `lpstat -p` lists `Dymo_LabelWriter_450` (or configured CUPS queue).
- [ ] **Audio Capture Device**: `arecord -l` detects the USB capture card.
- [ ] **Clock Sync**: `timedatectl status` confirms `System clock synchronized: yes`.
- [ ] **Inventory API Connectivity**: Outbound HTTPS requests to `Inventory-Server` succeed without TLS validation errors.
- [ ] **Firewall**: `sudo ufw status numbered` maintains port `22/tcp` allow rule; outbound traffic unobstructed.

---

## 4. Rollback and Disaster Recovery

In the event of binary regression or hardware faults:
```bash
# Stop daemon
sudo systemctl stop label-agent.service

# Inspect systemd logs
sudo journalctl -u label-agent.service --no-pager -n 100

# Roll back to previous binary if present
if [ -f /usr/local/bin/label-agent.prev ]; then
    sudo mv /usr/local/bin/label-agent.prev /usr/local/bin/label-agent
    sudo systemctl restart label-agent.service
fi
```
