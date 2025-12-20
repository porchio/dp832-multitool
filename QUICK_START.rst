DP832 Multitool - Quick Start Guide
====================================

Installation
------------

.. code-block:: bash

   # Clone the repository (if not already done)
   git clone <repository-url>
   cd dp832-battery-sim

   # Build release version
   cargo build --release

The binaries will be in:

- ``target/release/battery-sim`` - Battery simulator
- ``target/release/remote-control`` - Remote control interface

Screenshots
-----------

Battery Simulator Interface
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. image:: screenshots/battery-sim-three-channels.png
   :alt: Battery Simulator - Three Channels
   :width: 100%
   :align: center

*Three-channel battery simulation showing real-time voltage, current, and power graphs with event and SCPI logs*

Remote Control Interface
~~~~~~~~~~~~~~~~~~~~~~~~~

.. image:: screenshots/remote-control-main.png
   :alt: Remote Control Interface
   :width: 100%
   :align: center

*Remote control interface with real-time channel monitoring and control*

Tools
-----

Remote Control
~~~~~~~~~~~~~~

Interactive terminal interface for complete control of the DP832 power supply.

Basic Usage
^^^^^^^^^^^

.. code-block:: bash

   remote-control --ip 192.168.1.140

With Configuration File
^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   remote-control --config config.toml

Keyboard Controls
^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1
   :widths: 20 80

   * - Key
     - Action
   * - ``↑/↓``
     - Select channel
   * - ``V``
     - Edit voltage setpoint
   * - ``C``
     - Edit current setpoint
   * - ``SPACE``
     - Toggle output on/off
   * - ``A``
     - Enable all channels
   * - ``R``
     - Refresh measurements
   * - ``L``
     - Clear event log
   * - ``S``
     - Clear SCPI log
   * - ``Q``
     - Quit

**See** :doc:`REMOTE_CONTROL` **for complete remote control documentation.**

Battery Simulator
~~~~~~~~~~~~~~~~~

Real-time battery simulation with advanced modeling.

Basic Usage
^^^^^^^^^^^

Single Channel
^^^^^^^^^^^^^^

.. code-block:: bash

   battery-sim --ip 192.168.1.140 -p profiles/lifepo4.json

Multiple Channels
^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Three different battery chemistries
   battery-sim --ip 192.168.1.140 \
     -p profiles/lifepo4.json \
     -p profiles/liion_18650.json \
     -p profiles/lipo_1s.json

With Configuration File
^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   battery-sim --config examples/three_channels.toml \
     -p profiles/lifepo4.json \
     -p profiles/liion_18650.json

Keyboard Controls
^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1
   :widths: 20 80

   * - Key
     - Action
   * - ``q``
     - Quit the application
   * - ``r``
     - Reset all SoC values to 100%
   * - ``l``
     - Clear event log window
   * - ``s``
     - Clear SCPI command log window

Configuration
-------------

Battery Profiles
~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 25 35 15 25

   * - Profile
     - Description
     - Voltage
     - Chemistry
   * - ``lifepo4.json``
     - LiFePO4 single cell
     - 3.2V
     - LiFePO4
   * - ``lifepo4_3s.json``
     - LiFePO4 3S pack
     - 9.6V
     - LiFePO4
   * - ``liion_18650.json``
     - 18650 Li-ion cell
     - 3.7V
     - Li-ion
   * - ``lipo_1s.json``
     - 1S LiPo battery
     - 3.7V
     - LiPo
   * - ``lead_acid_6v.json``
     - Lead-acid battery
     - 6V
     - Lead-acid
   * - ``nimh_aa.json``
     - NiMH AA cell
     - 1.2V
     - NiMH

Keyboard Controls
-----------------

.. list-table::
   :header-rows: 1
   :widths: 20 80

   * - Key
     - Action
   * - ``q``
     - Quit the application
   * - ``r``
     - Reset all SoC values to 100%
   * - ``l``
     - Clear event log window
   * - ``s``
     - Clear SCPI command log window

Configuration Options
~~~~~~~~~~~~~~~~~~~~~

Remote Control
^^^^^^^^^^^^^^

- ``--ip <IP>`` - DP832 IP address (default: 192.168.1.100)
- ``--port <PORT>`` - SCPI port (default: 5555)
- ``--config <FILE>`` - TOML configuration file

Battery Simulator
^^^^^^^^^^^^^^^^^

- ``--ip <IP>`` - DP832 IP address (default: 192.168.1.100)
- ``--port <PORT>`` - SCPI port (default: 5555)
- ``-p, --profile <FILE>`` - Battery profile JSON (can specify multiple)
- ``--config <FILE>`` - TOML configuration file
- ``--log <FILE>`` - CSV log file prefix

Environment Variables
~~~~~~~~~~~~~~~~~~~~~

- ``VERBOSE_SCPI=1`` - Enable verbose SCPI logging

Example Configurations
----------------------

Located in ``examples/``:

- ``single_channel.toml`` - Basic single-channel setup
- ``three_channels.toml`` - All three channels active
- ``chemistry_comparison.toml`` - Compare different chemistries
- ``development.toml`` - Development testing
- ``bench.toml`` - Quick bench testing

Output Files
------------

Logs Directory
~~~~~~~~~~~~~~

Created automatically at runtime:

- ``logs/event_YYYYMMDD_HHMMSS.log`` - Runtime events and messages
- ``logs/scpi_YYYYMMDD_HHMMSS.log`` - All SCPI commands sent/received
- ``logs/output_ch1.csv`` - Per-channel data logs (if --log specified)

UI Layout
---------

.. code-block:: text

   ┌─────────────────────────────────────────────────────────────┐
   │ DP832 Battery Simulator                                     │
   ├─────────────────────────────────────────────────────────────┤
   │ Channel 1                                                   │
   │ ┌───────┐  ┌──────────────────────────────────────────┐   │
   │ │ SoC   │  │ Voltage | Current | Power (graphs)       │   │
   │ │ Gauge │  │                                           │   │
   │ └───────┘  └──────────────────────────────────────────┘   │
   ├─────────────────────────────────────────────────────────────┤
   │ Channel 2                                                   │
   │ ...                                                         │
   ├─────────────────────────────────────────────────────────────┤
   │ Channel 3                                                   │
   │ ...                                                         │
   ├─────────────────────────────────────────────────────────────┤
   │ Event Log        │ SCPI Commands                           │
   │ CH1: Init...     │ → INST:NSEL 1                           │
   │ CH2: Current...  │ → VOLT 3.200                            │
   │                  │ → MEAS:CURR? CH1                        │
   └─────────────────────────────────────────────────────────────┘

Troubleshooting
---------------

Connection Issues
~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Test connection to DP832
   ping 192.168.1.140
   telnet 192.168.1.140 5555

Enable Verbose Logging
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   VERBOSE_SCPI=1 cargo run --release -- --ip 192.168.1.140 -p profiles/lifepo4.json

Check Log Files
~~~~~~~~~~~~~~~

.. code-block:: bash

   # View latest event log
   tail -f logs/event_*.log | tail -1

   # View latest SCPI log
   tail -f logs/scpi_*.log | tail -1

Common Issues
-------------

1. **"Connection refused"** - Check IP address and ensure DP832 is powered on
2. **"Command error"** - Check SCPI log for details, may need power supply restart
3. **Graphs not updating** - Check that channels are enabled on power supply
4. **Application exits immediately** - Check error in event log, may be profile loading issue

More Information
----------------

- **Full Documentation**: See ``README.rst``
- **Development History**: See ``DEVELOPMENT_SUMMARY.rst``
- **Feature Checklist**: See ``PROJECT_STATUS.rst``
- **Profile Details**: See ``profiles/README.rst``
- **Config Examples**: See ``examples/README.rst``

Quick Examples
--------------

Test Single LiFePO4 Cell
~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   cargo run --release -- --ip 192.168.1.140 -p profiles/lifepo4.json

Compare Three Battery Chemistries
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   cargo run --release -- --ip 192.168.1.140 \
     -p profiles/lifepo4.json \
     -p profiles/liion_18650.json \
     -p profiles/nimh_aa.json

Run with Logging
~~~~~~~~~~~~~~~~

.. code-block:: bash

   cargo run --release -- \
     --ip 192.168.1.140 \
     --log battery_test \
     -p profiles/lifepo4.json
   # Creates: battery_test_ch1.csv

----

**For detailed information, see the comprehensive documentation in README.rst**
