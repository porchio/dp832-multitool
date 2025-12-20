===========================
DP832 Remote Control Guide
===========================

Overview
========

The DP832 Remote Control provides a complete terminal-based interface for controlling and monitoring the Rigol DP832 triple-output power supply. It offers real-time measurements, interactive control, and comprehensive logging capabilities.

Screenshots
===========

Main Interface
--------------

.. image:: screenshots/remote-control-main.png
   :alt: Remote Control Main Interface
   :width: 100%
   :align: center

*Remote control interface showing all three channels with real-time measurements and status indicators*

Editing Values
--------------

.. image:: screenshots/remote-control-editing.png
   :alt: Remote Control - Editing Values
   :width: 100%
   :align: center

*Voltage or current editing dialog with selected channel highlighted*

All Channels Active
-------------------

.. image:: screenshots/remote-control-all-channels-on.png
   :alt: Remote Control - All Channels Active
   :width: 100%
   :align: center

*All three channels enabled with live measurements and power calculations*

Features
========

Real-Time Monitoring
--------------------

- **Live measurements** for all three channels simultaneously
- **Voltage, current, and power** display with color coding
- **Output status** indicators (ON/OFF) with visual feedback
- **Auto-refresh** every 2 seconds (configurable)

Interactive Control
-------------------

- **Voltage and current setting** via intuitive keyboard interface
- **Channel output control** with spacebar toggle
- **Bulk operations** - enable all channels at once with 'A' key
- **Channel selection** with arrow keys
- **Visual feedback** for selected channel (highlighted)

Comprehensive Logging
---------------------

- **Event Log**: Runtime events, user actions, and system messages
- **SCPI Log**: All commands sent to and responses from the PSU
- **Persistent storage**: Logs automatically saved to timestamped files
- **Auto-scrolling**: Most recent entries always visible
- **In-memory buffering**: 100 events, 200 SCPI commands

Installation
============

Prerequisites
-------------

- Rust toolchain (1.70+)
- DP832 power supply on the network
- Terminal with good ANSI support

Build
-----

.. code-block:: bash

   git clone <repository-url>
   cd dp832-battery-sim
   cargo build --release

The binary will be located at: ``target/release/remote-control``

Quick Start
===========

Basic Usage
-----------

.. code-block:: bash

   # Connect to PSU at specific IP
   remote-control --ip 192.168.1.100
   
   # Use default port (5555)
   remote-control --ip 192.168.1.100 --port 5555
   
   # Use configuration file
   remote-control --config config.toml

Configuration File
------------------

Create a ``config.toml`` file:

.. code-block:: toml

   [device]
   ip = "192.168.1.100"
   port = 5555

Then run:

.. code-block:: bash

   remote-control --config config.toml

User Interface
==============

Layout
------

.. code-block:: text

   ╔═══════════════════════════════════════╗
   ║  DP832 Remote Control                 ║
   ╚═══════════════════════════════════════╝
   ┌─────────────────────────────────────────┐
   │ Channel Status                          │
   ├────┬─────────┬─────────┬────────────────┤
   │ CH │ Volt Set│ Curr Set│ Voltage │...   │
   ├────┼─────────┼─────────┼─────────┼──────┤
   │ 1  │  3.300 V│  2.000 A│  3.298 V│ ● ON │ ← Selected
   │ 2  │  5.000 V│  1.000 A│  5.001 V│ ○ OFF│
   │ 3  │ 12.000 V│  0.500 A│ 11.998 V│ ○ OFF│
   └────┴─────────┴─────────┴─────────┴──────┘
   ┌─────────────────────────────────────────┐
   │ Commands                                │
   │  ↑/↓  Select    V  Voltage   C  Current │
   │  SPC  Toggle    A  All ON    R  Refresh │
   │   L   Clr Event S  Clr SCPI  Q  Quit    │
   └─────────────────────────────────────────┘
   ┌──────────────────┬──────────────────────┐
   │ Event Log        │ SCPI Commands        │
   │ CH1: Init OK     │ → *CLS               │
   │ CH1 enabled      │ → *IDN?              │
   │ Setting voltage  │ ← RIGOL...           │
   │                  │ → MEAS:VOLT? CH1     │
   └──────────────────┴──────────────────────┘
   ┌─────────────────────────────────────────┐
   │ ● Ready. Use ↑/↓ to select channel...   │
   └─────────────────────────────────────────┘

Keyboard Controls
-----------------

Navigation
~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 15 85

   * - Key
     - Action
   * - ``↑``
     - Select previous channel (move selection up)
   * - ``↓``
     - Select next channel (move selection down)

Channel Control
~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 15 85

   * - Key
     - Action
   * - ``V``
     - Edit voltage for selected channel
   * - ``C``
     - Edit current for selected channel
   * - ``SPACE``
     - Toggle output ON/OFF for selected channel
   * - ``A``
     - Enable all channels at once

System Commands
~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 15 85

   * - Key
     - Action
   * - ``R``
     - Refresh measurements immediately (also updates every 2 seconds automatically)
   * - ``L``
     - Clear event log window
   * - ``S``
     - Clear SCPI command log window
   * - ``Q``
     - Quit the application

Editing Values
~~~~~~~~~~~~~~

When editing voltage or current:

.. list-table::
   :header-rows: 1
   :widths: 15 85

   * - Key
     - Action
   * - ``0-9 .``
     - Enter numeric value
   * - ``BACKSPACE``
     - Delete last character
   * - ``ENTER``
     - Confirm and apply the new value
   * - ``ESC``
     - Cancel editing without applying changes

Visual Indicators
-----------------

Selected Channel
~~~~~~~~~~~~~~~~

The currently selected channel is highlighted with:

- **Blue background**
- **Bold white text**
- Makes it clear which channel will be affected by commands

Output Status
~~~~~~~~~~~~~

Each channel shows its output status:

- **● ON** - Green, bold (output is enabled)
- **○ OFF** - Dark gray (output is disabled)

Measurements
~~~~~~~~~~~~

- **Set values**: Regular text (what you configured)
- **Actual values**: Green text (what the PSU is measuring)
- **Power**: Magenta text (calculated from V × I)

Workflow Examples
=================

Setting Up a Test
-----------------

1. **Start the remote control**:

   .. code-block:: bash

      remote-control --ip 192.168.1.100

2. **Select channel 1** (if not already selected):

   Press ``↓`` or ``↑`` until Channel 1 is highlighted

3. **Set voltage**:

   - Press ``V``
   - Type ``3.3``
   - Press ``ENTER``

4. **Set current limit**:

   - Press ``C``
   - Type ``2.0``
   - Press ``ENTER``

5. **Enable output**:

   Press ``SPACE``

6. **Monitor**:

   Watch the actual voltage, current, and power update in real-time

Multi-Channel Setup
-------------------

For setting up multiple channels quickly:

1. Configure Channel 1 (see above)
2. Press ``↓`` to select Channel 2
3. Press ``V``, enter voltage, press ``ENTER``
4. Press ``C``, enter current, press ``ENTER``
5. Repeat for Channel 3
6. Press ``A`` to enable all channels at once

Safety Shutdown
---------------

To quickly disable all outputs:

1. Press ``1`` to select Channel 1
2. Press ``SPACE`` to disable it
3. Press ``2`` to select Channel 2
4. Press ``SPACE`` to disable it
5. Press ``3`` to select Channel 3
6. Press ``SPACE`` to disable it

Or simply quit the application (``Q``) and manually turn off the PSU.

Logging
=======

Persistent Log Files
--------------------

All events and SCPI commands are automatically saved to timestamped files in the ``logs/`` directory:

**Event Log**: ``logs/event_YYYYMMDD_HHMMSS.log``

- Connection status
- User actions (channel enabled, voltage set, etc.)
- Errors and warnings
- System events

**SCPI Log**: ``logs/scpi_YYYYMMDD_HHMMSS.log``

- All commands sent to the PSU (marked with ``→``)
- All responses received from the PSU (marked with ``←``)
- Useful for debugging communication issues

Log Format
----------

Both logs use the same timestamp format:

.. code-block:: text

   2024-12-20 14:23:45.123 | Event message here
   2024-12-20 14:23:45.124 | → MEAS:VOLT? CH1
   2024-12-20 14:23:45.156 | ← 3.298

Viewing Logs
------------

During Runtime
~~~~~~~~~~~~~~

- **In the UI**: Logs are displayed in the bottom half of the interface
- **Event log**: Left window
- **SCPI log**: Right window
- **Auto-scrolling**: Most recent entries always visible

After Runtime
~~~~~~~~~~~~~

.. code-block:: bash

   # View the latest event log
   ls -lt logs/event_*.log | head -1 | xargs cat
   
   # View the latest SCPI log
   ls -lt logs/scpi_*.log | head -1 | xargs cat
   
   # Follow logs in real-time (in another terminal)
   tail -f logs/event_$(date +%Y%m%d)_*.log
   tail -f logs/scpi_$(date +%Y%m%d)_*.log

Log Management
--------------

Clearing In-Memory Logs
~~~~~~~~~~~~~~~~~~~~~~~~

While the application is running:

- Press ``L`` to clear the event log window
- Press ``S`` to clear the SCPI log window

Note: This only clears the in-memory display. The log files are not affected.

Managing Log Files
~~~~~~~~~~~~~~~~~~

Log files accumulate over time. You can manage them:

.. code-block:: bash

   # List all log files
   ls -lh logs/
   
   # Remove logs older than 30 days
   find logs/ -name "*.log" -mtime +30 -delete
   
   # Compress old logs
   gzip logs/*_202411*.log

Technical Details
=================

SCPI Communication
------------------

The remote control uses optimized SCPI communication:

**No Unnecessary Channel Switching**

Traditional approach (slow):

.. code-block:: text

   → INST:NSEL 1        # Switch to CH1
   → MEAS:VOLT?         # Measure voltage
   ← 3.298
   → INST:NSEL 2        # Switch to CH2
   → MEAS:VOLT?         # Measure voltage
   ← 5.001

Optimized approach (fast):

.. code-block:: text

   → MEAS:VOLT? CH1     # Measure CH1 directly
   ← 3.298
   → MEAS:CURR? CH1     # Measure CH1 current
   ← 1.234
   → MEAS:VOLT? CH2     # Measure CH2 directly
   ← 5.001

**Benefits:**

- Faster communication (fewer commands)
- No PSU front panel flashing (channel doesn't change)
- Reduced risk of "Command error"
- Less wear on PSU channel selection mechanism

Supported SCPI Commands
------------------------

The remote control uses these DP832 SCPI commands:

Initialization
~~~~~~~~~~~~~~

.. code-block:: text

   *CLS              # Clear status
   *IDN?             # Query device identification

Measurements (Query Only)
~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   MEAS:VOLT? CH1    # Measure voltage on channel 1
   MEAS:CURR? CH1    # Measure current on channel 1
   MEAS:VOLT? CH2    # Measure voltage on channel 2
   MEAS:CURR? CH2    # Measure current on channel 2
   MEAS:VOLT? CH3    # Measure voltage on channel 3
   MEAS:CURR? CH3    # Measure current on channel 3

Settings (Query and Set)
~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   APPL? CH1                  # Query channel 1 settings
   APPL CH1,3.3,2.0           # Set CH1 to 3.3V, 2.0A limit
   OUTP? CH1                  # Query CH1 output state
   OUTP CH1,ON                # Enable CH1 output
   OUTP CH1,OFF               # Disable CH1 output
   OUTP ALL,ON                # Enable all channels
   OUTP ALL,OFF               # Disable all channels

Update Frequency
----------------

- **Automatic updates**: Every 2 seconds
- **Manual refresh**: Press ``R`` key anytime
- **Immediate updates**: After any user change (voltage, current, output toggle)

This balanced approach provides responsive feedback without overwhelming the PSU with queries.

Error Handling
--------------

The remote control handles various error conditions:

Communication Errors
~~~~~~~~~~~~~~~~~~~~

- **Timeout**: If the PSU doesn't respond within 1 second
- **Connection lost**: If the TCP connection is broken
- **Parse errors**: If the PSU response is not valid

All errors are:

1. Displayed in the status bar
2. Logged to the event log
3. Saved to the event log file

"Command error" Response
~~~~~~~~~~~~~~~~~~~~~~~~

If the PSU responds with "Command error":

1. The error is logged in both event and SCPI logs
2. The operation is aborted
3. User is notified via status message

Common causes:

- Invalid parameter value
- Command not supported by PSU firmware
- Command sent while PSU is in a restricted state

Recovery:

- Check the SCPI log to see the exact command that failed
- Verify the command syntax in the DP832 manual
- Try power-cycling the PSU
- Update PSU firmware if available

Troubleshooting
===============

Connection Issues
-----------------

Cannot Connect to PSU
~~~~~~~~~~~~~~~~~~~~~

**Symptoms**: Application fails to start with connection error

**Solutions**:

1. Verify PSU IP address:

   .. code-block:: bash

      ping 192.168.1.100

2. Check PSU is powered on and on the network

3. Verify port number (default is 5555):

   .. code-block:: bash

      telnet 192.168.1.100 5555

4. Check firewall settings

5. Ensure no other application is connected to the PSU

Measurements Not Updating
~~~~~~~~~~~~~~~~~~~~~~~~~~

**Symptoms**: Values in the table don't change

**Solutions**:

1. Press ``R`` to manually refresh
2. Check SCPI log for errors
3. Verify channel is enabled on PSU
4. Check network connectivity (ping the PSU)

Display Issues
--------------

Garbled UI
~~~~~~~~~~

**Symptoms**: Terminal displays incorrect characters or boxes

**Solutions**:

1. Use a terminal with good ANSI support:
   
   - Linux: gnome-terminal, konsole, xterm
   - macOS: iTerm2, Terminal.app
   - Windows: Windows Terminal

2. Resize terminal to at least 80×24 characters

3. Try setting terminal type:

   .. code-block:: bash

      export TERM=xterm-256color
      remote-control --ip 192.168.1.100

Logs Not Scrolling
~~~~~~~~~~~~~~~~~~

**Symptoms**: Can't see recent log entries

**Solutions**:

- The logs auto-scroll automatically
- If it appears stuck, try:
  
  1. Press ``L`` or ``S`` to clear the log
  2. Resize the terminal window
  3. Restart the application

PSU Behavior Issues
-------------------

PSU Beeps on Command
~~~~~~~~~~~~~~~~~~~~

**Symptoms**: PSU beeps and shows "Command error" on display

**Solutions**:

1. Check SCPI log to see the failing command
2. Common causes:
   
   - Setting voltage beyond channel limits (CH1/CH2: 32V, CH3: 6V)
   - Setting current beyond channel limits (CH1/CH2: 3A, CH3: 3A)
   - Invalid syntax

3. Verify values are within range for the channel

Channel Switching on PSU
~~~~~~~~~~~~~~~~~~~~~~~~~

**Symptoms**: PSU front panel shows active channel changing

**Solutions**:

This should NOT happen with the current version. If it does:

1. Check you're using the latest version
2. Review SCPI log - should see ``MEAS:VOLT? CH1`` not ``INST:NSEL``
3. Report as a bug

Performance Issues
------------------

Slow Response
~~~~~~~~~~~~~

**Symptoms**: Delay between key press and action

**Solutions**:

1. Check network latency to PSU:

   .. code-block:: bash

      ping -c 10 192.168.1.100

2. Use wired connection instead of WiFi
3. Reduce network traffic
4. Check PSU is not overloaded with other connections

High CPU Usage
~~~~~~~~~~~~~~

**Symptoms**: Application uses lots of CPU

**Solutions**:

This shouldn't happen. If it does:

1. Check no infinite loop in logs
2. Restart the application
3. Report as a bug

Advanced Usage
==============

Integration with Scripts
------------------------

The remote control can be part of automated test sequences:

.. code-block:: bash

   #!/bin/bash
   # Start remote control in background (for monitoring only)
   # Note: Remote control is interactive, so this is mainly for demonstration
   
   # For actual automation, use direct SCPI commands via netcat/telnet
   echo "APPL CH1,3.3,2.0" | nc 192.168.1.100 5555
   echo "OUTP CH1,ON" | nc 192.168.1.100 5555
   sleep 5
   echo "MEAS:VOLT? CH1" | nc 192.168.1.100 5555
   echo "OUTP CH1,OFF" | nc 192.168.1.100 5555

Multiple Instances
------------------

You can run multiple remote control instances if needed (e.g., monitoring while another tool controls):

.. code-block:: bash

   # Terminal 1: Interactive control
   remote-control --ip 192.168.1.100
   
   # Terminal 2: Monitoring only (separate connection)
   # Note: Only one instance should control at a time

However, be aware that:

- Both instances will generate separate log files
- SCPI commands from both will be interleaved
- It's better to use a single instance for control

Configuration Tips
------------------

Create a Project-Specific Config
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

For each project, create a dedicated config file:

.. code-block:: toml

   # project1.toml
   [device]
   ip = "192.168.1.100"
   port = 5555

Then use:

.. code-block:: bash

   remote-control --config project1.toml

SSH Remote Access
~~~~~~~~~~~~~~~~~

The terminal UI works great over SSH:

.. code-block:: bash

   # On remote machine with PSU
   ssh user@remote-machine
   cd dp832-battery-sim
   remote-control --ip 192.168.1.100

Ensure your terminal supports ANSI escape sequences for best results.

Development & Contributing
==========================

Building from Source
--------------------

.. code-block:: bash

   git clone <repository-url>
   cd dp832-battery-sim
   cargo build --release

Running in Development Mode
----------------------------

.. code-block:: bash

   # With debug output
   RUST_LOG=debug cargo run --bin remote-control -- --ip 192.168.1.100
   
   # With SCPI verbose logging
   VERBOSE_SCPI=1 cargo run --bin remote-control -- --ip 192.168.1.100

Testing
-------

.. code-block:: bash

   # Run unit tests
   cargo test
   
   # Run specific module tests
   cargo test remote_control

Code Structure
--------------

.. code-block:: text

   src/remote_control/
   ├── mod.rs           # Module exports
   ├── controller.rs    # DP832Controller - PSU communication
   ├── config.rs        # Configuration structures
   └── ui.rs            # RemoteControlUI - Terminal interface

Architecture Documentation
---------------------------

See ``ARCHITECTURE.rst`` for detailed information about:

- Module structure
- Communication patterns
- Design decisions
- Future extensions

Reporting Issues
----------------

If you encounter a bug:

1. Check the SCPI log for the failing command
2. Check the event log for error messages
3. Save both log files
4. Note the PSU firmware version (shown on startup)
5. Create an issue with:
   
   - Description of the problem
   - Steps to reproduce
   - Log files
   - PSU firmware version

Feature Requests
----------------

To request a new feature:

1. Check existing issues
2. Describe the use case
3. Explain expected behavior
4. Suggest implementation if possible

FAQ
===

**Q: Can I control multiple power supplies?**

A: Not with a single instance. Each instance connects to one PSU. You can run multiple instances with different ``--ip`` arguments.

**Q: Does this work with other Rigol PSU models?**

A: It's designed for the DP832. It may work with DP831, DP832A, and similar models that use the same SCPI command set, but this is untested.

**Q: Can I automate tests with this tool?**

A: The remote control is interactive. For automation, use direct SCPI commands via scripts or use the tool as a monitoring interface while scripts control the PSU.

**Q: How do I change the update frequency?**

A: Currently hardcoded to 2 seconds. This can be made configurable if needed (feature request).

**Q: Why does the SCPI log show some commands twice?**

A: Some commands are logged when queued and when executed. This is expected behavior for debugging.

**Q: Can I save the current PSU state to a file?**

A: Not currently. This would be a useful feature (feature request).

**Q: Do the log files grow forever?**

A: The in-memory logs are limited (100 events, 200 SCPI commands), but log files grow continuously. Implement log rotation or manual cleanup as needed.

**Q: What's the difference between event log and SCPI log?**

A: 

- **Event log**: High-level actions and messages (user-friendly)
- **SCPI log**: Low-level commands and responses (debugging)

**Q: Can I customize the colors?**

A: Not currently. This could be added as a configuration option (feature request).

Reference
=========

Command Summary
---------------

.. list-table::
   :header-rows: 1
   :widths: 10 20 70

   * - Key
     - Category
     - Function
   * - ↑/↓
     - Navigation
     - Select channel
   * - V
     - Edit
     - Set voltage for selected channel
   * - C
     - Edit
     - Set current for selected channel
   * - SPACE
     - Control
     - Toggle output for selected channel
   * - A
     - Control
     - Enable all channels
   * - R
     - System
     - Refresh measurements
   * - L
     - System
     - Clear event log
   * - S
     - System
     - Clear SCPI log
   * - Q
     - System
     - Quit application

File Locations
--------------

.. list-table::
   :header-rows: 1
   :widths: 40 60

   * - Item
     - Location
   * - Binary
     - ``target/release/remote-control``
   * - Config file
     - ``config.toml`` (or specified with ``--config``)
   * - Event logs
     - ``logs/event_YYYYMMDD_HHMMSS.log``
   * - SCPI logs
     - ``logs/scpi_YYYYMMDD_HHMMSS.log``

SCPI Command Reference
----------------------

See the DP832 Programming Guide for complete SCPI command reference. The remote control primarily uses:

- ``*CLS`` - Clear status
- ``*IDN?`` - Identification query
- ``MEAS:VOLT? CHn`` - Measure voltage
- ``MEAS:CURR? CHn`` - Measure current
- ``APPL? CHn`` - Query settings
- ``APPL CHn,V,I`` - Set voltage and current
- ``OUTP? CHn`` - Query output state
- ``OUTP CHn,ON/OFF`` - Set output state
- ``OUTP ALL,ON/OFF`` - Set all outputs

See Also
========

- ``README.rst`` - Complete project documentation
- ``ARCHITECTURE.rst`` - Technical architecture details
- ``QUICK_START.rst`` - Quick start guide
- ``REMOTE_CONTROL_IMPROVEMENTS.rst`` - Recent improvements
- DP832 Programming Guide - Official SCPI command reference (Rigol)
