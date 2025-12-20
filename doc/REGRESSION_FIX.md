# Regression Fix: Command Error Issue

## Problem
After commit 04f020a (which worked perfectly), subsequent commits introduced intermittent "Command error" responses from the DP832 power supply. The application would display errors like:
- `CH1: ERROR - Failed to parse current 'Command error'`
- PSU beeping due to invalid commands
- Non-deterministic failures, especially after `*IDN?` queries

## Root Causes

### Primary Cause: Non-blocking TCP Mode (Fixed in bb8d809)
The first regression was introduced by switching from **blocking TCP mode** to **non-blocking mode** with manual timeout handling.

#### Working Version (commit 04f020a):
```rust
stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
```

#### Broken Versions (commits after 04f020a):
```rust
stream.set_nonblocking(true).unwrap();
// Plus complex manual timeout logic in query() function
```

The non-blocking mode changes included:
1. Manual timeout tracking with `Instant::now()` and `elapsed()`
2. Active polling with sleep delays in the read loop
3. Buffer draining logic to prevent response bleed
4. Variable timeouts for different command types
5. Extra delays between commands (200ms)
6. Explicit `flush()` calls

### Secondary Cause: Shared TCP Connection (Fixed in eacfc05)
Even after reverting to blocking mode, "Command error" responses persisted because:

#### Working Version (commit 04f020a):
- **Each channel thread** had its own **dedicated TcpStream**
- Independent communication per channel
- No race conditions possible

#### Broken Versions (after bb8d809):
- **Single shared** `ScpiConnection` protected by `Arc<Mutex<>>`
- All channels competed for the same TCP stream
- Race conditions when multiple threads accessed simultaneously
- Responses could be mixed between threads

## Solutions

### Solution 1: Revert to Blocking Mode (bb8d809)
1. **Removed non-blocking mode**: Changed back to `set_read_timeout(Some(Duration::from_secs(1)))`
2. **Simplified `query()` function**: Removed all manual timeout logic, delays, and larger buffers
3. **Removed `drain_buffer()` function**: Not needed with proper blocking mode
4. **Removed command delays**: Eliminated 200ms delays between initialization commands
5. **Removed flush()**: TCP streams auto-flush, this was redundant

### Solution 2: Separate TCP Connection Per Channel (eacfc05)
1. **Removed ScpiConnection struct**: Eliminated shared connection wrapper
2. **Create dedicated TcpStream** for each channel in main loop:
   ```rust
   let stream_clone = TcpStream::connect(&addr).unwrap();
   stream_clone.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
   ```
3. **Pass stream ownership** to each simulate_channel() thread
4. **Direct send()/query() calls** with SCPI logging maintained

## Key Differences

### send() function:
**Before (broken):**
```rust
fn send(stream: &mut TcpStream, cmd: &str) {
    let cmd = format!("{}\n", cmd);
    stream.write_all(cmd.as_bytes()).unwrap();
    stream.flush().unwrap();  // Unnecessary
}
```

**After (fixed):**
```rust
fn send(stream: &mut TcpStream, cmd: &str) {
    let cmd = format!("{}\n", cmd);
    stream.write_all(cmd.as_bytes()).unwrap();
}
```

### query() function:
**Before (broken):** 67 lines with delays, manual timeouts, larger buffers
**After (fixed):** 24 lines, simple blocking read with WouldBlock as termination

### Thread Architecture:
**Before (broken):**
```rust
// Single shared connection
let shared_conn = Arc::new(Mutex::new(conn));
for profile in profiles {
    let conn_clone = shared_conn.clone();
    std::thread::spawn(move || {
        simulate_channel(state, writers, conn_clone, profile, csv);
    });
}
```

**After (fixed):**
```rust
// Dedicated connection per channel
for profile in profiles {
    let stream_clone = TcpStream::connect(&addr).unwrap();
    stream_clone.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    std::thread::spawn(move || {
        simulate_channel(state, writers, stream_clone, profile, csv);
    });
}
```

## Lessons Learned

1. **Keep it simple**: The original blocking mode with OS-level timeout was sufficient and reliable
2. **Don't over-engineer**: Manual non-blocking I/O is complex and error-prone when blocking mode works fine
3. **Avoid shared mutable state across threads**: Each thread should have its own resources when possible
4. **TCP streams can handle multiple connections**: The DP832 supports multiple simultaneous TCP connections
5. **Test regressions**: When adding features, ensure core functionality still works
6. **Trust the OS**: Blocking sockets with timeouts are well-tested and reliable

## Testing
The fixes were validated by:
1. Comparing code with working commit 04f020a
2. Ensuring compilation succeeds
3. Removing all timing-related workarounds that were masking the root causes
4. Restoring the original multi-connection architecture

## Commits
- Regression fix (blocking mode): bb8d809
- Regression fix (separate connections): eacfc05
- Original working version: 04f020a
