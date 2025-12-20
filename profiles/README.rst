Battery Profiles
================

This directory contains battery profile definitions for various battery chemistries and configurations.

Available Profiles
------------------

Lithium-based Chemistries
~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 20 25 10 12 15 18

   * - File
     - Description
     - Channel
     - Capacity
     - Voltage Range
     - Use Case
   * - ``lifepo4.json``
     - LiFePO4 1S cell
     - 1
     - 3.2 Ah
     - 2.5-3.65V
     - Standard LiFePO4 testing
   * - ``lifepo4_3s.json``
     - LiFePO4 3S pack
     - 3
     - 10.0 Ah
     - 8.0-11.1V
     - Higher voltage pack testing
   * - ``liion_18650.json``
     - Li-ion 18650 cell
     - 2
     - 2.5 Ah
     - 2.8-4.2V
     - Standard Li-ion testing
   * - ``lipo_1s.json``
     - LiPo 1S cell
     - 3
     - 1.2 Ah
     - 3.0-4.2V
     - High-discharge LiPo testing

Other Chemistries
~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 20 25 10 12 15 18

   * - File
     - Description
     - Channel
     - Capacity
     - Voltage Range
     - Use Case
   * - ``nimh_aa.json``
     - NiMH AA cell
     - 1
     - 2.0 Ah
     - 0.9-1.45V
     - NiMH testing
   * - ``lead_acid_6v.json``
     - Lead-Acid 6V
     - 2
     - 4.5 Ah
     - 5.25-7.2V
     - Lead-acid battery testing

Profile Format
--------------

Each profile is a JSON file with the following structure:

.. code-block:: json

   {
     "name": "Battery Name",
     "channel": 1,                         // DP832 channel (1, 2, or 3)
     
     "capacity_ah": 3.2,                   // Battery capacity in Ah
     "internal_resistance_ohm": 0.015,     // Internal resistance in Ω
     
     "current_limit_discharge_a": 2.0,     // Maximum discharge current in A
     "current_limit_charge_a": 2.0,        // Maximum charge current in A
     
     "cutoff_voltage": 2.5,                // Minimum voltage (discharge cutoff)
     "max_voltage": 3.65,                  // Maximum voltage (charge limit)
     
     "rc_time_constant_ms": 300,           // RC time constant for voltage smoothing
     "update_interval_ms": 100,            // Update rate in milliseconds
     
     "ocv_curve": [                        // Open Circuit Voltage vs SoC curve
       { "soc": 1.00, "voltage": 3.40 },
       { "soc": 0.90, "voltage": 3.32 },
       { "soc": 0.50, "voltage": 3.30 },
       { "soc": 0.20, "voltage": 3.25 },
       { "soc": 0.00, "voltage": 2.80 }
     ]
   }

Key Parameters Explained
-------------------------

- **channel**: Which DP832 channel to use (1, 2, or 3). Make sure this matches your physical setup.

- **capacity_ah**: The battery capacity in Ampere-hours. This determines how fast the SoC decreases during discharge.

- **internal_resistance_ohm**: The battery's internal resistance. Affects voltage drop under load.

- **rc_time_constant_ms**: Smoothing time constant for voltage response. Higher values = slower response to load changes.

- **ocv_curve**: The relationship between State of Charge (0.0 to 1.0) and Open Circuit Voltage. The simulator interpolates between these points.

Creating Custom Profiles
-------------------------

1. Copy an existing profile that matches your battery chemistry
2. Modify the parameters to match your specific battery
3. Adjust the OCV curve based on your battery's discharge curve
4. Set the appropriate channel number
5. Save with a descriptive filename

Usage Examples
--------------

Single Battery
~~~~~~~~~~~~~~

.. code-block:: bash

   dp832_battery_sim -p profiles/lifepo4.json

Multiple Batteries (Different Chemistries)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   dp832_battery_sim \
     -p profiles/lifepo4.json \
     -p profiles/liion_18650.json \
     -p profiles/lipo_1s.json

Chemistry Comparison
~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   dp832_battery_sim \
     -p profiles/lifepo4.json \
     -p profiles/liion_18650.json

Notes
-----

- Ensure each profile targets a different channel when using multiple profiles
- The OCV curve should be ordered from highest SoC (1.0) to lowest (0.0)
- Voltage ranges should be appropriate for your DP832 channel capabilities
- Current limits should not exceed DP832 channel specifications

Contributing
------------

When adding new profiles, please:

1. Use descriptive names
2. Include accurate OCV curves from datasheets
3. Set realistic internal resistance values
4. Test the profile before committing
