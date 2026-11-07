# Raspberry Pi 3 Model B+ Deployment Runbook: `label-agent`

## 1. Node Profile & Hardware Ground Truth
Reference: [`retired_raspberry_pi_dossier.md`](/home/console-admin/Github/Label-Agent/retired_raspberry_pi_dossier.md)

| Attribute | Specification | Operational Constraint |
| :--- | :--- | :--- |
| **Hardware** | Raspberry Pi 3 Model B+ Rev 1.3 (`a020d3`) | 4x ARM Cortex-A53 (`aarch64`) @ 1.40 GHz |
| **RAM / Swap** | 1 GB LPDDR2 (908 MiB usable) / 200 MiB swap | `MemoryMax=256M`, `MemoryHigh=192M` enforced |
| **Storage** | 32 GB Class 10 microSDHC (`/dev/mmcblk0p2`) | 23.0 GiB available; write await is 159 ms |
| **RAM Disk** | `tmpfs` mounted at `/tmp` (454 MiB) | Zero-SD-wear spooling (`PrivateTmp=no`) |
| **Operating System**| Debian GNU/Linux 13.6 (`trixie`) `aarch64` | Kernel `6.12.43+rpt-rpi-v8`, headless console |
| **Network IP** | `10.0.0.10/24` on `enxb827eb4b7845` | Static DHCP lease on EdgeRouter X (`10.0.0.1`) |
| **Target Hostname**| `sbc-label-agent` | Transitioning from decommissioned `sbc-lyrion` |
| **Connected Peripherals** | Dymo LabelWriter 450 (USB), USB microphone | Required user groups: `audio`, `lp` |

---

## 2. Pre-Deployment Provisioning Steps

Execute commands over SSH on the target node (`console-admin@10.0.0.10`):

### Step 1: Purge Dormant Lyrion & NGINX Services
Reclaims approximately `+1.2 GB` of storage on root partition `/dev/mmcblk0p2`:

```bash
# Unmask legacy music server services
sudo systemctl unmask lyrionmusicserver.service nginx.service

# Purge packages and unused dependencies
sudo apt-get purge -y lyrionmusicserver nginx nginx-common nginx-light
sudo apt-get autoremove --purge -y

# Clean up legacy directories and maintenance timers
sudo rm -rf /var/lib/squeezeboxserver /etc/nginx /etc/ssl/molinadeveloping.com \
            /usr/local/bin/lyrion-maintenance-backup.sh \
            /etc/systemd/system/lyrion-backup.service \
            /etc/systemd/system/lyrion-backup.timer \
            /home/console-admin/media

sudo systemctl daemon-reload
```

Verify storage reclamation:
```bash
df -h /
```

---

### Step 2: Unmask and Enable Required Services (NTP & CUPS)
1. **NTPsec Clock Synchronization** (Mandatory for TLS/HTTPS certificate validation when querying `Inventory-Server`):
   ```bash
   sudo systemctl unmask ntpsec.service ntpsec-systemd-netif.path ntpsec-rotate-stats.timer
   sudo systemctl enable --now ntpsec.service ntpsec-rotate-stats.timer
   ntpq -p
   ```

2. **CUPS Printing Subsystem** (Required for hardware label output on `Dymo_LabelWriter_450`):
   ```bash
   sudo systemctl unmask cups.service cups.socket cups.path
   sudo systemctl enable --now cups.socket cups.service
   systemctl is-active cups.service
   ```

3. **Verify Dymo Printer Attachment via CUPS**:
   ```bash
   lpstat -p -d
   lsusb | grep -i dymo
   ```

---

### Step 3: Set Appliance Hostname
Update system identity from legacy `sbc-lyrion` to `sbc-label-agent`:

```bash
sudo hostnamectl set-hostname sbc-label-agent

# Ensure /etc/hosts resolves local address correctly
sudo sed -i 's/sbc-lyrion/sbc-label-agent/g' /etc/hosts
hostnamectl status
```

---

### Step 4: Cross-Compile and Deploy Appliance Binary

On development workstation (`console-admin@workstation`):

1. **Cross-Compile for ARM64**:
   ```bash
   cd /home/console-admin/Github/Label-Agent
   rustup target add aarch64-unknown-linux-gnu
   cargo build --release --target aarch64-unknown-linux-gnu --bin label-agent
   ```

2. **Deploy Executable to Pi 3B+**:
   ```bash
   scp target/aarch64-unknown-linux-gnu/release/label-agent console-admin@10.0.0.10:/tmp/label-agent.new
   ssh console-admin@10.0.0.10 "sudo install -m 0755 /tmp/label-agent.new /usr/local/bin/label-agent && rm /tmp/label-agent.new"
   ```

3. **Install and Activate Systemd Service**:
   ```bash
   scp deploy/systemd/label-agent.service console-admin@10.0.0.10:/tmp/label-agent.service
   ssh console-admin@10.0.0.10 "sudo install -m 0644 /tmp/label-agent.service /etc/systemd/system/label-agent.service && \
     sudo systemctl daemon-reload && \
     sudo systemctl enable --now label-agent.service"
   ```

---

## 3. Post-Deployment Verification Checklist

Verify live operational health on `sbc-label-agent` (`10.0.0.10`):

- [ ] **Daemon Running**: `systemctl status label-agent.service` reports `active (running)`.
- [ ] **Memory Footprint**: `systemctl show label-agent.service -p MemoryCurrent` reports `< 25M` (well below `MemoryHigh=192M`).
- [ ] **Tmpfs Spool Directory**: `/tmp/label-agent` exists on `tmpfs` and receives render buffers.
- [ ] **Audio Capture**: User `console-admin` can read ALSA capture devices (`arecord -l`).
- [ ] **CUPS Spool Access**: `lpstat -r` confirms scheduler is running and permissions allow printing.
- [ ] **NTP Synchronization**: `timedatectl timesync-status` shows synchronized state.
- [ ] **UFW Firewall Integrity**: `sudo ufw status numbered` allows outbound traffic to `Inventory-Server` on port `443/tcp` or internal inventory endpoint.

---

## 4. Rollback and Disaster Recovery

In the event of binary regression or hardware faults:
```bash
# Emergency daemon halt
sudo systemctl stop label-agent.service

# Read crash journals
sudo journalctl -u label-agent.service --no-pager -n 100

# Revert to previous binary backup if available
if [ -f /usr/local/bin/label-agent.prev ]; then
    sudo mv /usr/local/bin/label-agent.prev /usr/local/bin/label-agent
    sudo systemctl restart label-agent.service
fi
```
