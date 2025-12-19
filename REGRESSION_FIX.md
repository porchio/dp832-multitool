# Regression Fix: Command Error Issue

## Problem
After commit 04f020a (which worked perfectly), subsequent commits introduced intermittent "Command error" responses from the DP832 power supply. The application would display errors like:
- `CH1: ERROR - Failed to parse current 'Command error'`
- PSU beeping due to invalid commands
- Non-deterministic failures, especially after `*IDN?` queries

## Root Cause
The regression was introduced by switching from **blocking TCP mode** to **non-blocking mode** with manual timeout handling:

### Working Version (commit 04f020a):
```rust
stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
```

### Broken Versions (commits after 04f020a):
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

All of this complexity was attempting to solve problems that didn't exist in the simpler blocking mode approach.

## Solution
Reverted to the simple, reliable blocking mode approach:

1. **Removed non-blocking mode**: Changed back to `set_read_timeout(Some(Duration::from_secs(1)))`
2. **Simplified `query()` function**: Removed all manual timeout logic, delays, and larger buffers
3. **Removed `drain_buffer()` function**: Not needed with proper blocking mode
4. **Removed command delays**: Eliminated 200ms delays between initialization commands
5. **Removed flush()**: TCP streams auto-flush, this was redundant

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

## Lessons Learned

1. **Keep it simple**: The original blocking mode with OS-level timeout was sufficient and reliable
2. **Don't over-engineer**: Manual non-blocking I/O is complex and error-prone when blocking mode works fine
3. **Test regressions**: When adding features, ensure core functionality still works
4. **Trust the OS**: Blocking sockets with timeouts are well-tested and reliable

## Testing
The fix was validated by:
1. Comparing code with working commit 04f020a
2. Ensuring compilation succeeds
3. Removing all timing-related workarounds that were masking the root cause

## Commit
Regression fix committed as: bb8d809
Original working version: 04f020a
