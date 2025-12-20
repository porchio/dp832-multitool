=======================================
Remote Control Interface Improvements
=======================================

Overview
========

This document describes the recent improvements to the DP832 Remote Control interface.

Changes Implemented
===================

Log Windows
-----------

Two new log windows have been added to the remote control interface:

1. **Event Log** (left window)
   
   - Displays user actions and system events
   - Shows connection status, errors, and configuration changes
   - Automatically scrolls to show most recent entries
   - Stores up to 100 most recent messages
   - Can be cleared with ``L`` key

2. **SCPI Commands Log** (right window)
   
   - Displays all SCPI commands sent to the power supply
   - Useful for debugging communication issues
   - Automatically scrolls to show most recent entries
   - Stores up to 200 most recent messages
   - Can be cleared with ``S`` key

Both logs are automatically saved to timestamped files in the ``logs/`` directory:

- ``logs/event_YYYYMMDD_HHMMSS.log``
- ``logs/scpi_YYYYMMDD_HHMMSS.log``

User Interface Improvements
----------------------------

**Spacebar Toggle**

The channel output toggle has been changed from the ``O`` key to the ``SPACE`` key for improved ergonomics and consistency with common UI patterns.

**Updated Key Bindings**

- ``↑/↓`` - Select channel
- ``V`` - Set voltage
- ``C`` - Set current
- ``SPACE`` - Toggle channel output (changed from ``O``)
- ``R`` - Refresh measurements
- ``L`` - Clear event log (new)
- ``S`` - Clear SCPI log (new)
- ``Q`` - Quit

Communication Optimization
--------------------------

The SCPI communication has been optimized to minimize unnecessary channel switching on the physical power supply unit:

- Channel switching (``INST:NSEL``) only occurs when setting voltage or current
- Measurements use channel-specific queries (e.g., ``MEAS:VOLT? CH1``)
- Output state changes use channel-specific commands (e.g., ``OUTP ON,CH1``)

This reduces communication overhead and prevents the PSU from unnecessarily switching active channels during normal operation.

Technical Implementation
========================

SCPI Logging Architecture
--------------------------

The SCPI logging uses a channel-based communication pattern:

1. The ``DP832Controller`` holds a ``Sender<String>`` for logging
2. The ``RemoteControlUI`` holds a ``Receiver<String>`` to collect logs
3. Each SCPI command is sent through the channel before execution
4. The UI polls the receiver and processes log messages in the main loop

This non-blocking design ensures that logging doesn't impact command execution timing.

Log File Management
-------------------

Log files are created using the ``LogWriters`` utility from the common module:

- Files are created with timestamps when the application starts
- Each log entry is timestamped with millisecond precision
- Files are flushed after each write to ensure data persistence
- Logs directory is created automatically if it doesn't exist

Benefits
========

1. **Improved Debugging**: All SCPI commands are now visible and logged
2. **Better User Experience**: Spacebar is more intuitive for toggle operations
3. **Reduced PSU Wear**: Fewer channel switches on the physical unit
4. **Audit Trail**: Timestamped log files provide a complete history of operations
5. **Real-time Monitoring**: Live log windows show what's happening as it happens

Usage Examples
==============

Starting the Remote Control
----------------------------

.. code-block:: bash

   # Using config file
   cargo run --release --bin remote-control -- --config config.toml
   
   # Using command line arguments
   cargo run --release --bin remote-control -- --ip 192.168.1.100 --port 5555

Viewing Logs
------------

Log files are created in the ``logs/`` directory:

.. code-block:: bash

   # View event log
   tail -f logs/event_20240101_120000.log
   
   # View SCPI commands
   tail -f logs/scpi_20240101_120000.log

Troubleshooting
===============

If SCPI Commands Show "Command error"
--------------------------------------

Check the SCPI log window to see exactly what commands are being sent. Common issues:

- Verify the command syntax matches the DP832 manual
- Check for extra whitespace or special characters
- Ensure the PSU firmware is up to date

If Logs Are Not Saved
----------------------

- Verify the ``logs/`` directory exists and is writable
- Check disk space
- Ensure the application has write permissions

Future Enhancements
===================

Potential future improvements:

1. Configurable log retention (number of messages to keep in memory)
2. Log filtering (show only errors, warnings, etc.)
3. Export logs to different formats (CSV, JSON)
4. Search functionality within logs
5. Log playback for reproducing issues
