# Channel-Specific Measurement Fix

## Problem
Both voltage graphs were showing voltage from CH2, regardless of which channel was supposed to be displayed. This indicated that measurements were not correctly targeting individual channels.

## Root Cause
The code was using `INST:NSEL {channel}` to select a channel, then using `MEAS:CURR?` without a channel parameter. While each channel had its own TCP connection (to avoid race conditions), the measurement command without an explicit channel parameter was not reliably reading from the selected channel.

## Solution
Changed to use channel-specific measurement syntax:
- **Before:** `MEAS:CURR?` (relied on prior channel selection)
- **After:** `MEAS:CURR? CH1` (explicit channel parameter)

This syntax is confirmed to work correctly with the DP832 power supply.

## Code Changes
In `src/main.rs`, line ~372-376:

```rust
// Old approach (unreliable):
log_scpi!(state, writers, "{} → MEAS:CURR?", ch_name);
let curr_str = query(&mut stream, "MEAS:CURR?");

// New approach (reliable):
let curr_cmd = format!("MEAS:CURR? {}", ch_name);
log_scpi!(state, writers, "{} → {}", ch_name, curr_cmd);
let curr_str = query(&mut stream, &curr_cmd);
```

## Benefits
1. **Reliable channel-specific measurements** - Each channel now correctly reads its own current
2. **Consistent with DP832 best practices** - Uses explicit channel parameters
3. **Better logging** - SCPI log now shows which channel is being queried
4. **Maintains separation** - Still uses separate TCP connections per channel to avoid race conditions

## Testing
After this fix:
- CH1 voltage graph shows CH1 data
- CH2 voltage graph shows CH2 data  
- CH3 voltage graph shows CH3 data
- Current and power measurements are also channel-specific

## Note
The voltage displayed in the graphs is still the **calculated** voltage (`v_filt`) based on the battery model, not a measured voltage from the PSU. This is by design - the simulator sets the voltage and measures the current to simulate battery behavior.
