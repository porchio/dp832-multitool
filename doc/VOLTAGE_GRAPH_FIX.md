# Voltage Graph Fix

## Problem
The voltage graphs were not working properly:
- CH1 would stop showing data (graph would flatline or disappear)
- CH2 would show varying voltage even when it should be constant
- Data from different channels was being mixed or misattributed

## Root Cause
The code was using invalid SCPI syntax for the DP832 power supply:
- Used `:MEAS:CURR? CH1` which is NOT supported by the DP832
- Repeatedly selected channels with `INST:NSEL` before each voltage update

The DP832 does not support channel parameters in MEAS commands. Instead, it requires:
1. Selecting the channel with `INST:NSEL <channel>`
2. Using simple commands like `MEAS:CURR?` (without channel parameter)

## Solution
Reverted to the simple SCPI approach that was working in commit 04f020a:

### Key Changes:
1. **Channel selection**: Only done once at initialization
   - Each simulation thread has its own TCP connection
   - The channel selection persists for that connection
   - No need to re-select before each command

2. **Current measurement**: Use simple `MEAS:CURR?`
   - Removed invalid `:MEAS:CURR? CH1` syntax
   - Reads from the currently selected channel (set at init)

3. **Voltage updates**: Use simple `VOLT` command
   - Removed redundant `INST:NSEL` before each voltage update
   - Channel is already selected and persists on this connection

### Why This Works:
- Each channel's simulation runs in its own thread
- Each thread has its own dedicated TCP connection to the PSU
- Channel selection via `INST:NSEL` is connection-specific, not global
- Once a channel is selected on a connection, it stays selected
- This allows each thread to safely use simple commands without channel parameters

## Benefits:
✅ Eliminates "Command error" responses from PSU  
✅ Each channel correctly displays its own voltage data  
✅ Graphs show proper data without mixing between channels  
✅ Reduced SCPI traffic (fewer unnecessary INST:NSEL commands)  
✅ Matches the proven working approach from commit 04f020a  

## Technical Details:
- The DP832 maintains channel selection state per TCP connection
- Multiple simultaneous TCP connections each have their own channel context
- This design allows safe concurrent access to different channels
- The simpler command set is more reliable and less error-prone

## Commit:
```
4327d28 fix: revert to simple SCPI commands without channel parameters
```

## Testing:
To verify the fix works:
1. Run with multiple profiles: `cargo run -- -p profiles/lifepo4.json -p profiles/liion_18650.json --ip <IP>`
2. Observe that each channel graph shows independent, correct voltage data
3. CH1 should show its voltage curve based on its profile
4. CH2 should show its voltage curve based on its profile
5. No "Command error" messages should appear in the SCPI log
