Project Architecture
====================

Overview
--------

The DP832 Multitool is structured as a Rust library with multiple binary applications. This modular architecture allows code reuse and clean separation of concerns.

Project Structure
-----------------

.. code-block:: text

   dp832-battery-sim/
   ├── src/
   │   ├── lib.rs                 # Library entry point
   │   ├── scpi.rs                # SCPI communication primitives
   │   ├── common.rs              # Shared types and utilities
   │   ├── battery_sim/           # Battery simulator module
   │   │   ├── mod.rs
   │   │   ├── model.rs           # Battery physics model
   │   │   ├── config.rs          # Configuration structures
   │   │   └── ui.rs              # Terminal UI for battery sim
   │   ├── remote_control/        # Remote control module
   │   │   ├── mod.rs
   │   │   ├── controller.rs      # DP832 control logic
   │   │   ├── config.rs          # Configuration structures
   │   │   └── ui.rs              # Terminal UI for remote control
   │   └── bin/
   │       ├── battery-sim.rs     # Battery simulator binary
   │       └── remote-control.rs  # Remote control binary
   ├── profiles/                  # Battery profile JSON files
   ├── examples/                  # Example configuration files
   └── logs/                      # Runtime logs (generated)

Core Modules
------------

SCPI Communication (scpi.rs)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Low-level SCPI communication primitives:

- ``send(stream, cmd)`` - Send a SCPI command
- ``query(stream, cmd)`` - Send a query and read response

These functions handle:

- Command formatting (newline termination)
- Response reading and parsing
- Timeout handling
- Error detection

Common Module (common.rs)
~~~~~~~~~~~~~~~~~~~~~~~~~~

Shared types and utilities used across both tools:

**Types:**

- ``DeviceConfig`` - Device connection configuration
- ``ChannelState`` - Runtime state for a power supply channel
- ``RuntimeState`` - Overall application state
- ``LogWriters`` - File logging infrastructure

**Functions:**

- ``load_optional_config()`` - Load TOML configuration files

Battery Simulator Module
~~~~~~~~~~~~~~~~~~~~~~~~~

Located in ``src/battery_sim/``, implements realistic battery simulation:

**model.rs**

- ``BatteryProfile`` - Battery characteristics and parameters
- ``OcvPoint`` - Open-circuit voltage curve data point
- ``interpolate_ocv()`` - Interpolate voltage from SoC

**config.rs**

- Configuration file structures specific to battery simulation

**ui.rs**

- Rich terminal interface with:
  
  - Real-time graphs (voltage, current, power)
  - SoC gauge display
  - Dual log windows (events and SCPI)
  - Split-view for multiple channels

Remote Control Module
~~~~~~~~~~~~~~~~~~~~~~

Located in ``src/remote_control/``, provides full PSU remote control:

**controller.rs**

- ``DP832Controller`` - Main controller class
- ``ChannelState`` - Per-channel state tracking
- Methods for:
  
  - Voltage/current setting
  - Output control
  - Measurement reading
  - State updates

**ui.rs**

- Interactive terminal interface with:
  
  - Channel selection
  - Live measurements display
  - Interactive editing
  - Real-time updates

Communication Architecture
--------------------------

Battery Simulator
~~~~~~~~~~~~~~~~~

The battery simulator uses a **separate TCP connection per channel** to avoid race conditions and SCPI command interference:

.. code-block:: text

   DP832 Device
   ├── TCP Connection 1 → Channel 1 Simulator Thread
   ├── TCP Connection 2 → Channel 2 Simulator Thread
   └── TCP Connection 3 → Channel 3 Simulator Thread

Each connection:

1. Selects its channel once at initialization (``INST:NSEL``)
2. Uses simple SCPI commands without channel parameter
3. Maintains channel selection throughout lifetime
4. Clears errors independently (``*CLS``)

This architecture eliminates the "Command error" issues that occurred when multiple threads shared a single connection.

Remote Control
~~~~~~~~~~~~~~

The remote control uses a **single TCP connection** with explicit channel selection before each operation. This is appropriate since operations are serialized through the UI thread.

Key Design Decisions
--------------------

Why Separate Binaries?
~~~~~~~~~~~~~~~~~~~~~~~

1. **Clean separation of concerns** - Each tool has distinct purpose
2. **Smaller binaries** - Users can deploy only what they need
3. **Independent evolution** - Tools can evolve independently
4. **Code reuse** - Common code in library prevents duplication

Why Rust?
~~~~~~~~~

1. **Performance** - Zero-cost abstractions for real-time simulation
2. **Safety** - Memory safety prevents crashes during long simulations
3. **Concurrency** - Safe multi-threading for multi-channel operation
4. **Ecosystem** - Excellent libraries for TUI, networking, serialization

Why Terminal UI (TUI)?
~~~~~~~~~~~~~~~~~~~~~~~

1. **Low resource usage** - No GUI framework overhead
2. **SSH friendly** - Works over remote connections
3. **Scriptable** - Easy integration with automation
4. **Fast development** - Rich TUI libraries (ratatui)
5. **Cross-platform** - Works on Linux, macOS, Windows

Future Extensions
-----------------

The modular architecture makes it easy to add new tools:

Potential Future Tools
~~~~~~~~~~~~~~~~~~~~~~

- **Data logger** - Record and analyze PSU measurements
- **Load testing** - Automated load test sequences  
- **Calibration** - Automated calibration procedures
- **Web interface** - HTTP/WebSocket server for browser control

Adding a New Tool
~~~~~~~~~~~~~~~~~

1. Create new module under ``src/`` (e.g., ``src/data_logger/``)
2. Add module declaration in ``src/lib.rs``
3. Create binary in ``src/bin/`` (e.g., ``src/bin/data-logger.rs``)
4. Add binary section to ``Cargo.toml``
5. Reuse common code from ``scpi.rs`` and ``common.rs``

Development Guidelines
----------------------

Code Organization
~~~~~~~~~~~~~~~~~

- **Library code** (``src/*.rs``, ``src/*/``) - Reusable components
- **Binary code** (``src/bin/*.rs``) - Application entry points
- **One binary per tool** - Keep binaries small and focused

Error Handling
~~~~~~~~~~~~~~

- Use ``Result<T, E>`` for fallible operations
- Provide meaningful error messages
- Log errors to both UI and file

Testing
~~~~~~~

- Unit tests for model calculations
- Integration tests for SCPI communication
- Manual testing with real hardware

Dependencies
~~~~~~~~~~~~

Keep dependencies minimal and well-maintained:

- ``clap`` - Command-line argument parsing
- ``ratatui`` - Terminal user interface
- ``crossterm`` - Cross-platform terminal control
- ``serde`` - Serialization/deserialization
- ``toml`` - TOML configuration parsing
- ``csv`` - CSV logging
- ``chrono`` - Timestamp generation

Build System
------------

The project uses Cargo with multiple binary targets:

.. code-block:: toml

   [[bin]]
   name = "battery-sim"
   path = "src/bin/battery-sim.rs"

   [[bin]]
   name = "remote-control"
   path = "src/bin/remote-control.rs"

Build Commands
~~~~~~~~~~~~~~

.. code-block:: bash

   # Build all binaries
   cargo build --release

   # Build specific binary
   cargo build --release --bin battery-sim
   cargo build --release --bin remote-control

   # Run specific binary
   cargo run --bin battery-sim -- --help
   cargo run --bin remote-control -- --help
