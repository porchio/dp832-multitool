Migration Guide
===============

From Single Tool to Multitool
------------------------------

The DP832 project has been restructured from a single battery simulator into a comprehensive multitool with multiple applications.

What Changed
------------

Binary Names
~~~~~~~~~~~~

**Old:**

- Single binary: ``dp832_battery_sim``

**New:**

- Battery simulator: ``battery-sim``
- Remote control: ``remote-control``

Why the Change?
~~~~~~~~~~~~~~~

The project has evolved to support multiple use cases:

1. **Battery simulation** - Original functionality, now optimized
2. **Remote control** - New complete PSU control interface
3. **Future tools** - Architecture supports easy addition of new tools

Migration Steps
---------------

For Existing Users
~~~~~~~~~~~~~~~~~~

If you were using the battery simulator, simply replace the binary name:

**Before:**

.. code-block:: bash

   dp832_battery_sim --ip 192.168.1.100 -p profile.json

**After:**

.. code-block:: bash

   battery-sim --ip 192.168.1.100 -p profile.json

All command-line arguments remain the same. Configuration files are fully compatible.

For Scripts and Automation
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Update any scripts that invoke the binary:

.. code-block:: bash

   # Old
   ./target/release/dp832_battery_sim "$@"
   
   # New
   ./target/release/battery-sim "$@"

What Stayed the Same
--------------------

Fully Compatible
~~~~~~~~~~~~~~~~

- ✅ All battery profile JSON files
- ✅ All TOML configuration files
- ✅ Command-line arguments
- ✅ CSV log output format
- ✅ Runtime behavior
- ✅ SCPI communication
- ✅ Terminal UI appearance

No changes required to:

- Battery profiles
- Configuration files
- Monitoring scripts
- Automation workflows

New Capabilities
----------------

Remote Control Interface
~~~~~~~~~~~~~~~~~~~~~~~~~

The new ``remote-control`` tool provides:

- Interactive control of all three channels
- Live voltage/current monitoring
- Quick output enable/disable
- Set point adjustments
- Real-time power display

Quick start:

.. code-block:: bash

   remote-control --ip 192.168.1.100

Benefits of New Architecture
-----------------------------

Code Organization
~~~~~~~~~~~~~~~~~

- Cleaner module structure
- Better code reuse
- Easier to understand
- Simpler to extend

Performance
~~~~~~~~~~~

- Slightly smaller binaries
- Faster compilation for individual tools
- Better resource usage

Maintenance
~~~~~~~~~~~

- Independent tool updates
- Focused testing
- Clear responsibility boundaries

Troubleshooting
---------------

Binary Not Found
~~~~~~~~~~~~~~~~

After rebuilding, the binary names have changed. Update your PATH or scripts to use:

- ``battery-sim`` instead of ``dp832_battery_sim``
- ``remote-control`` for the new tool

Old Binary Still Present
~~~~~~~~~~~~~~~~~~~~~~~~~

After ``git pull`` and rebuild, you may have both old and new binaries:

.. code-block:: bash

   # Clean build to remove old artifacts
   cargo clean
   cargo build --release

Configuration Not Loading
~~~~~~~~~~~~~~~~~~~~~~~~~~

Configuration files work the same way. Ensure you're using the correct path:

.. code-block:: bash

   battery-sim --config ~/.config/dp832-battery/config.toml

Getting Help
------------

Each tool has built-in help:

.. code-block:: bash

   battery-sim --help
   remote-control --help

See Also
--------

- ``README.rst`` - Complete usage guide
- ``ARCHITECTURE.rst`` - Project structure documentation
- ``examples/`` - Example configuration files
- ``profiles/`` - Battery profile examples
