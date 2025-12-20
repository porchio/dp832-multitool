DP832 Multitool - Documentation Index
=====================================

Welcome to the DP832 Multitool documentation.

This toolkit provides comprehensive tools for the Rigol DP832 power supply:

- **Battery Simulator** - Real-time battery behavior simulation
- **Remote Control** - Complete remote interface for all PSU functions

.. image:: screenshots/battery-sim-three-channels.png
   :alt: Battery Simulator with Three Channels
   :width: 100%
   :align: center

.. image:: screenshots/remote-control-main.png
   :alt: Remote Control Interface
   :width: 100%
   :align: center

Main Documentation
------------------

.. toctree::
   :maxdepth: 2

   README
   QUICK_START
   REMOTE_CONTROL
   ARCHITECTURE
   MIGRATION
   PROJECT_STATUS
   DEVELOPMENT_SUMMARY

Getting Started
---------------

New users should start with:

1. :doc:`README` - Complete user guide and feature documentation
2. :doc:`QUICK_START` - Quick start guide for immediate usage
3. :doc:`REMOTE_CONTROL` - Remote control interface guide
4. :doc:`ARCHITECTURE` - Project structure and design
5. :doc:`MIGRATION` - Migration guide from single-tool version
6. :doc:`examples/README` - Example configurations
7. :doc:`profiles/README` - Battery chemistry profiles

Project Information
-------------------

For developers and contributors:

- :doc:`PROJECT_STATUS` - Current project status and feature checklist
- :doc:`DEVELOPMENT_SUMMARY` - Complete development history
- GitHub repository: (add your repository URL here)

Component Documentation
-----------------------

.. toctree::
   :maxdepth: 1

   examples/README
   profiles/README
   screenshots/README

Quick Links
-----------

Installation
~~~~~~~~~~~~

.. code-block:: bash

   git clone <repository-url>
   cd dp832-battery-sim
   cargo build --release

Basic Usage
~~~~~~~~~~~

**Battery Simulator:**

.. code-block:: bash

   # Single channel
   battery-sim --ip 192.168.1.100 -p profiles/lifepo4.json

   # Three channels
   battery-sim --ip 192.168.1.100 \
     -p profiles/lifepo4.json \
     -p profiles/liion_18650.json \
     -p profiles/lipo_1s.json

**Remote Control:**

.. code-block:: bash

   # Connect and control PSU
   remote-control --ip 192.168.1.100

Features
--------

Multi-Channel Support
~~~~~~~~~~~~~~~~~~~~~

- 3 independent channels
- Per-channel configuration
- Split-view display

Advanced Battery Modeling
~~~~~~~~~~~~~~~~~~~~~~~~~~

- State of Charge (SoC) tracking
- Internal resistance modeling
- RC time constants
- Customizable OCV curves
- Cutoff protection
- Current limiting

Rich Terminal UI
~~~~~~~~~~~~~~~~

- Real-time voltage/current/power graphs
- Live SoC gauges
- Event and SCPI command logs
- Keyboard controls

Robust Communication
~~~~~~~~~~~~~~~~~~~~

- Optimized SCPI communication
- Adaptive timeouts
- Error recovery
- Detailed logging

Troubleshooting
---------------

Common Issues
~~~~~~~~~~~~~

**Connection refused**
   Check IP address and ensure DP832 is powered on

**Command error from PSU**
   Check SCPI log for details, may need power supply restart

**Graphs not updating**
   Ensure channels are enabled on power supply

**Application exits immediately**
   Check error in event log, may be profile loading issue

Enable Verbose Logging
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   VERBOSE_SCPI=1 dp832_battery_sim -p profiles/lifepo4.json

Contributing
------------

Contributions welcome! Areas of interest:

- Additional battery chemistry profiles
- More sophisticated battery models (thermal effects, aging)
- Additional tools (data logger, calibration, etc.)
- Additional power supply models
- Documentation improvements

License
-------

[Your license here]

Indices and Tables
------------------

* :ref:`genindex`
* :ref:`search`
