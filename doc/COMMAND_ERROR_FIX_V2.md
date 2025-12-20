# Command Error Fix - Second Pass

## Problem
Even after the initial SCPI command fix (commit c4dbd32), users were still experiencing intermittent "Command error" responses from the DP832 PSU. The errors were non-deterministic and often occurred after the initial `*IDN?` query.

## Root Causes Identified

### 1. Uninitialized TCP Connections
Each channel's TCP connection was not being properly initialized with `*CLS` to clear the error state. While the main connection used for `*IDN?` was cleared, the per-channel connections were not.

**Impact:** Channels could start with leftover errors from previous sessions or other connections.

### 2. Error Response Cascade
When the PSU returned "Command error" as a response to a query, the code attempted to parse it as a numeric value, which failed. The error state wasn't being cleared, causing subsequent commands to also fail.

**Impact:** A single error could cascade into multiple failures.

### 3. Initialization Sequence
The initialization commands were being sent in a loop over an array, which might have subtle timing or reference issues compared to the working version.

**Impact:** Potential for initialization to fail or be incomplete.

## Fixes Applied

### Fix 1: Initialize Each TCP Connection (Commit 0868e25)
```rust
// Before
let stream_clone = TcpStream::connect(&addr).unwrap();
stream_clone.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

// After
let mut stream_clone = TcpStream::connect(&addr).unwrap();
stream_clone.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
send(&mut stream_clone, "*CLS");  // Clear errors on this connection
```

**Why this helps:** Each channel's connection starts with a clean error state, independent of any other connection or previous session.

### Fix 2: Detect and Clear Error Responses (Commit 99b70e2)
```rust
// Check for error responses before parsing
let curr_result: Result<f64, String> = {
    let trimmed = curr_str.trim();
    if trimmed.contains("error") || trimmed.contains("Error") || trimmed.contains("ERROR") {
        // PSU returned error - clear it and retry
        log_message!(state, writers, "CH{}: PSU error response '{}' - clearing error state", 
                    profile.channel, trimmed);
        send(&mut stream, "*CLS");  // Clear error state
        Err(trimmed.to_string())
    } else {
        trimmed.parse().map_err(|_| trimmed.to_string())
    }
};
```

**Why this helps:** When an error response is detected, we:
1. Log it for debugging
2. Clear the PSU's error state with `*CLS`
3. Retry on the next iteration

This prevents error cascades and allows recovery from transient errors.

### Fix 3: Explicit Initialization Sequence (Commit caffffe)
```rust
// Before: Array-based initialization
let init_cmds = [
    format!("INST:NSEL {}", profile.channel),
    "OUTP OFF".to_string(),
    format!("CURR {:.3}", profile.current_limit_discharge_a),
    "OUTP ON".to_string(),
];
for cmd in &init_cmds {
    log_scpi!(state, writers, "CH{} → {}", profile.channel, cmd);
    send(&mut stream, cmd);
}

// After: Direct calls (matches working version 04f020a)
log_scpi!(state, writers, "CH{} → INST:NSEL {}", profile.channel, profile.channel);
send(&mut stream, &format!("INST:NSEL {}", profile.channel));

log_scpi!(state, writers, "CH{} → OUTP OFF", profile.channel);
send(&mut stream, "OUTP OFF");

log_scpi!(state, writers, "CH{} → CURR {:.3}", profile.channel, profile.current_limit_discharge_a);
send(&mut stream, &format!("CURR {:.3}", profile.current_limit_discharge_a));

log_scpi!(state, writers, "CH{} → OUTP ON", profile.channel);
send(&mut stream, "OUTP ON");
```

**Why this helps:** 
- Matches the exact pattern from the working commit 04f020a
- More explicit and easier to debug
- Detailed logging for each step

## Testing

The fixes can be verified by:

1. **Check SCPI Log Window:**
   - Should show all commands being sent
   - Look for any "Command error" responses
   - If errors occur, should see "*CLS" being sent to clear them

2. **Check Event Log Window:**
   - Should show "Initialized" message for each channel
   - If errors occur, should see "PSU error response" messages followed by retry
   - Should NOT see cascading errors

3. **Monitor PSU Behavior:**
   - PSU should NOT beep (indicating invalid commands)
   - All channels should turn on and start simulating
   - Voltage/current/power graphs should update smoothly

4. **Check Log Files:**
   - `logs/event_YYYYMMDD_HHMMSS.log` - Application events
   - `logs/scpi_YYYYMMDD_HHMMSS.log` - All SCPI commands sent/received

## Expected Behavior

With these fixes:
- ✅ Each channel connection is properly initialized
- ✅ Error responses are detected and cleared automatically
- ✅ Transient errors are handled with retry logic
- ✅ Initialization follows the proven working pattern
- ✅ All commands and responses are logged for debugging
- ✅ Maximum 5 consecutive errors before safety shutdown

## Comparison with Working Version (04f020a)

| Aspect | Working Version (04f020a) | Current Version |
|--------|---------------------------|-----------------|
| SCPI Commands | Simple (no channel suffix) | ✅ Same |
| TCP Connections | One per channel | ✅ Same |
| Channel Selection | Once at init | ✅ Same |
| Error Handling | Basic (unwrap_or) | ✅ Enhanced (detect & clear) |
| Connection Init | No *CLS | ✅ Now includes *CLS |
| Logging | Console only | ✅ UI + Files |
| Initialization | Direct calls | ✅ Now matches exactly |

## What's Different from 04f020a?

The current version has all the functionality of 04f020a, plus:
1. **Enhanced error handling** - Detects and recovers from PSU errors
2. **Connection initialization** - Each connection cleared with *CLS
3. **Detailed logging** - SCPI log and Event log windows
4. **Persistent logs** - Timestamped log files
5. **Better diagnostics** - Can see exactly what's happening

## If Issues Persist

If you still see "Command error" responses:

1. **Check the SCPI log** - See exactly what commands are failing
2. **Check the timing** - Multiple channels might be overwhelming the PSU
3. **Try single channel** - Test with just one profile to isolate issues
4. **Check DP832 firmware** - Ensure PSU has latest firmware
5. **Network issues** - Check for network latency or packet loss

## Next Steps

1. Rebuild the application: `cargo build --release`
2. Run with logging enabled (automatic)
3. Monitor both log windows during operation
4. Check generated log files in `logs/` directory
5. Report any remaining issues with log file contents
