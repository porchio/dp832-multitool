DP832 Battery Simulator - Documentation Index
==============================================

Welcome to the DP832 Battery Simulator documentation.

Main Documentation
------------------

.. toctree::
   :maxdepth: 2

   README
   QUICK_START
   PROJECT_STATUS
   DEVELOPMENT_SUMMARY

Getting Started
---------------

New users should start with:

1. :doc:`README` - Complete user guide and feature documentation
2. :doc:`QUICK_START` - Quick start guide for immediate usage
3. :doc:`examples/README` - Example configurations
4. :doc:`profiles/README` - Battery chemistry profiles

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

.. code-block:: bash

   # Single channel
   dp832_battery_sim --ip 192.168.1.100 -p profiles/lifepo4.json

   # Three channels
   dp832_battery_sim --ip 192.168.1.100 \
     -p profiles/lifepo4.json \
     -p profiles/liion_18650.json \
     -p profiles/lipo_1s.json

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
- Remote monitoring/control interface
- Additional power supply models

License
-------

[Your license here]

Indices and Tables
------------------

* :ref:`genindex`
* :ref:`search`
