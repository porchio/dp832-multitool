# Example Configurations

This directory contains example TOML configuration files for various use cases.

## Available Examples

### 1. Single Channel Testing (`single_channel.toml`)

Basic single-channel setup for testing one battery.

```bash
dp832_battery_sim --config examples/single_channel.toml
```

**Use case**: Simple testing of a single battery profile.

---

### 2. Three Channel Testing (`three_channels.toml`)

Configuration for running all three channels simultaneously.

```bash
dp832_battery_sim --config examples/three_channels.toml \
  -p profiles/lifepo4.json \
  -p profiles/liion_18650.json \
  -p profiles/lipo_1s.json
```

**Use case**: 
- Testing three different batteries simultaneously
- Comparing multiple battery chemistries
- Maximum utilization of DP832 capabilities

---

### 3. Chemistry Comparison (`chemistry_comparison.toml`)

Side-by-side comparison of different battery chemistries.

```bash
dp832_battery_sim --config examples/chemistry_comparison.toml \
  -p profiles/lifepo4.json \
  -p profiles/liion_18650.json
```

**Use case**:
- Direct comparison between LiFePO4 and Li-ion
- Performance analysis
- Discharge characteristic studies

---

### 4. Development Testing (`development.toml`)

Quick setup for development and testing.

```bash
dp832_battery_sim --config examples/development.toml
```

**Use case**:
- Development testing
- Quick iterations
- Feature validation

---

### 5. Bench Testing (`bench.toml`)

The original bench testing configuration.

```bash
dp832_battery_sim --config examples/bench.toml
```

**Use case**: Production bench testing setup.

---

## Configuration File Format

All configuration files use TOML format:

```toml
[device]
ip = "192.168.1.100"      # DP832 IP address
port = 5555               # SCPI port (default: 5555)

[battery]
profile = "profiles/lifepo4.json"   # Default battery profile

[logging]
csv = "logs/test.csv"     # CSV log file path (optional)
```

## Usage Patterns

### Basic Usage (Config File Only)
```bash
dp832_battery_sim --config examples/single_channel.toml
```

### Override Profile from Command Line
```bash
dp832_battery_sim --config examples/single_channel.toml \
  -p profiles/liion_18650.json
```

### Multi-Channel with Config File
```bash
dp832_battery_sim --config examples/three_channels.toml \
  -p profiles/lifepo4.json \
  -p profiles/liion_18650.json \
  -p profiles/lipo_1s.json
```

### Override IP Address
```bash
dp832_battery_sim --config examples/single_channel.toml \
  --ip 192.168.1.50
```

### Multi-Channel with Logging
```bash
dp832_battery_sim \
  --ip 192.168.1.100 \
  -p profiles/lifepo4.json \
  -p profiles/liion_18650.json \
  --log logs/comparison_test.csv
```

## Command Line Arguments

Command line arguments override config file settings:

| Argument | Description | Example |
|----------|-------------|---------|
| `--config` | Path to config file | `--config examples/single_channel.toml` |
| `--ip` | DP832 IP address | `--ip 192.168.1.100` |
| `--port` | SCPI port | `--port 5555` |
| `-p, --profile` | Battery profile(s) | `-p profiles/lifepo4.json` |
| `--log` | CSV log file | `--log logs/test.csv` |

## Creating Custom Configurations

1. Copy an existing example that matches your use case
2. Modify the `[device]` section with your DP832 IP address
3. Set the default `[battery]` profile
4. Configure logging path if needed
5. Save with a descriptive name in the `examples/` directory

## Tips

- **IP Address**: Update the IP address to match your DP832's network configuration
- **Logging**: CSV files are created per-channel automatically (e.g., `test_ch1.csv`, `test_ch2.csv`)
- **Config File Location**: You can also place `config.toml` in `~/.config/dp832-battery/` for system-wide defaults
- **No Config**: You can run without a config file by specifying all parameters on the command line

## Multi-Channel CSV Logging

When using `--log test.csv` with multiple channels, the simulator automatically creates separate log files:

```
logs/test_ch1.csv    # Channel 1 data
logs/test_ch2.csv    # Channel 2 data  
logs/test_ch3.csv    # Channel 3 data
```

Each CSV contains:
- Timestamp
- State of Charge (SoC)
- Voltage
- Current
- Power

## Troubleshooting

**Cannot connect to DP832**:
- Verify IP address in config file
- Check network connectivity: `ping <ip_address>`
- Ensure DP832 SCPI server is enabled
- Check firewall settings

**Profile not found**:
- Verify profile path is relative to working directory
- Use absolute paths if needed
- Check that JSON files are valid

**Multiple channels using same channel number**:
- Each profile must specify a unique channel (1, 2, or 3)
- Edit the `"channel"` field in each profile JSON file
