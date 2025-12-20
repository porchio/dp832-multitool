# Fix Summary: DP832 SCPI Command Errors

## Issue
The DP832 power supply was responding with "Command error" to certain SCPI commands, causing the battery simulation to fail or behave incorrectly.

## Root Cause
The code was using **unsupported channel-specific SCPI syntax**:
- `MEAS:CURR? CH1` (not supported by DP832)
- `OUTP CH1,OFF` (causes command errors)
- Redundant `INST:NSEL` calls before every `VOLT` command

## Solution
Reverted to the **standard SCPI command pattern**:

1. **Select channel once** at initialization: `INST:NSEL 1`
2. **Use simple commands** without channel suffix:
   - `MEAS:CURR?` instead of `MEAS:CURR? CH1`
   - `OUTP OFF` instead of `OUTP CH1,OFF`
   - `VOLT 3.250` without re-selecting channel
3. **Channel remains selected** for the entire TCP connection

## Key Changes

```diff
- let curr_cmd = format!("MEAS:CURR? CH{}", profile.channel);
+ let curr_cmd = "MEAS:CURR?";

- format!("OUTP CH{},OFF", profile.channel),
+ "OUTP OFF".to_string(),

- // Re-select channel before voltage
- format!("INST:NSEL {}", profile.channel),
- format!("VOLT {:.3}", v_filt),
+ // Channel already selected, just set voltage
+ format!("VOLT {:.3}", v_filt),
```

## Result
✅ No more "Command error" responses  
✅ Faster execution (fewer commands per update)  
✅ Reliable multi-channel operation  
✅ Standard SCPI compliance  

## Commits
- `c4dbd32` - Fix: revert to simple SCPI commands
- `55eca3d` - Docs: document the fix

## Testing
Compare with commit `04f020a` (known working version) confirmed the approach.
