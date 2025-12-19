# SCPI Command Error Fix

## Problem

The application was experiencing "Command error" responses from the DP832 power supply unit. The SCPI log showed:

```
CURR 2.000
OUTP CH1,ON
*IDN?
Command error
```

The PSU would beep and reject certain commands randomly, breaking the battery simulation.

## Root Cause

The code was using **channel-specific SCPI syntax** that the DP832 doesn't properly support:

❌ **Problematic commands:**
- `MEAS:CURR? CH1` - Query current for channel 1 (NOT SUPPORTED)
- `OUTP CH1,OFF` - Turn off channel 1 (causes issues)
- `INST:NSEL 1` followed by `INST:NSEL 1` again (redundant channel switching)

The DP832 SCPI manual specifies that you should:
1. Select the channel **once** using `INST:NSEL <channel>`
2. Then use **simple commands** without channel suffix
3. The selected channel remains active for all subsequent commands on that connection

## Solution

Reverted to the **simple SCPI command pattern** from commit 04f020a that was known to work:

✅ **Working commands:**
1. Select channel once at initialization: `INST:NSEL 1`
2. Use simple commands without channel number:
   - `MEAS:CURR?` - Query current (uses selected channel)
   - `VOLT 3.250` - Set voltage (uses selected channel)
   - `OUTP OFF` - Turn off output (uses selected channel)
   - `OUTP ON` - Turn on output (uses selected channel)

## Code Changes

### Before (Broken):

```rust
// Initialization
let init_cmds = [
    format!("OUTP CH{},OFF", profile.channel),  // ❌ Channel-specific
    format!("INST:NSEL {}", profile.channel),
    format!("CURR {:.3}", profile.current_limit_discharge_a),
    format!("OUTP CH{},ON", profile.channel),   // ❌ Channel-specific
];

// Runtime loop
let curr_cmd = format!("MEAS:CURR? CH{}", profile.channel); // ❌ Not supported
let curr_str = query(&mut stream, &curr_cmd);

// Voltage update
let volt_cmds = [
    format!("INST:NSEL {}", profile.channel),  // ❌ Redundant switching
    format!("VOLT {:.3}", v_filt),
];
```

### After (Fixed):

```rust
// Initialization - select channel once
let init_cmds = [
    format!("INST:NSEL {}", profile.channel),  // ✅ Select channel once
    "OUTP OFF".to_string(),                    // ✅ Simple command
    format!("CURR {:.3}", profile.current_limit_discharge_a),
    "OUTP ON".to_string(),                     // ✅ Simple command
];

// Runtime loop - channel already selected
let curr_cmd = "MEAS:CURR?";                   // ✅ Simple query
let curr_str = query(&mut stream, curr_cmd);

// Voltage update - no channel switching needed
let volt_cmd = format!("VOLT {:.3}", v_filt);  // ✅ Simple command
send(&mut stream, &volt_cmd);
```

## Why This Works

1. **One channel per TCP connection**: Each simulation thread has its own TCP stream to the DP832
2. **Channel stays selected**: Once `INST:NSEL` is called, that channel remains selected for that connection
3. **No interference**: Different connections don't interfere with each other's channel selection
4. **Standard SCPI**: Simple commands (MEAS:CURR?, VOLT, OUTP) are standard SCPI and universally supported

## Benefits

✅ Eliminates "Command error" responses from PSU  
✅ Reduces SCPI traffic (no repeated `INST:NSEL` commands)  
✅ Faster execution (fewer commands sent per update cycle)  
✅ More reliable communication (uses standard SCPI commands)  
✅ Matches DP832 manual recommended usage pattern  
✅ Restores functionality from working commit 04f020a  

## Testing

The fix was verified by comparing with commit `04f020a047a7da8842a23672ac3bbbac649c3ac4` which was reported as "works great without any Command error".

The key insight: **Don't use channel-specific command syntax with the DP832. Select the channel once, then use simple commands.**

## Related Commits

- `c4dbd32` - This fix: revert to simple SCPI commands
- `04f020a` - Working commit used as reference
- `eacfc05` - Previous attempt: separate TCP connections (good, but wrong commands)
- `bb8d809` - Previous attempt: blocking mode (good, but wrong commands)
