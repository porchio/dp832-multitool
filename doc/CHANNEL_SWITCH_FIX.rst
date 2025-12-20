===============================================
Remote Control Channel Switching Optimization
===============================================

:Date: 2024
:Status: Completed
:Commit: 42af9b9

Overview
========

The remote control interface was unnecessarily switching the active channel on the DP832 power supply
during routine measurements and updates. This caused performance issues and constantly changed which
channel was selected on the physical device.

Problem
=======

The ``update_channel()`` function was using ``INST:NSEL`` command to switch the active channel just to
read voltage and current setpoints. This meant:

* Every measurement cycle switched channels multiple times
* The PSU's display would constantly change active channels
* Increased SCPI communication overhead
* Slower UI responsiveness

Previous Implementation
=======================

.. code-block:: rust

    // Old code that switched channel
    send(&mut self.stream, &format!("INST:NSEL {}", channel));
    let v_set_str = query(&mut self.stream, "VOLT?");
    let i_set_str = query(&mut self.stream, "CURR?");

This required selecting each channel before querying its setpoints.

Solution
========

Changed to use the ``APPL?`` command which can query setpoints for a specific channel without
switching the active channel:

.. code-block:: rust

    // New code - no channel switching needed
    let appl_str = query(&mut self.stream, &format!("APPL? {}", ch_name));
    let parts: Vec<&str> = appl_str.split(',').collect();
    // Parse: "CH1,3.300,2.000,ON"

The ``APPL?`` command returns all settings for a channel in one query:
``<channel>,<voltage>,<current>,<state>``

Benefits
========

1. **No Channel Switching During Measurements**: The PSU active channel only changes when the user
   explicitly sets voltage or current

2. **Better Performance**: Reduced SCPI command count per update cycle

3. **Improved User Experience**: The PSU display remains stable instead of constantly switching

4. **Cleaner Code**: Single command replaces multiple commands + channel switch

Commands That Do NOT Switch Channel
====================================

The following commands work with channel-specific syntax and don't require switching:

* ``MEAS:VOLT? CH1`` - Measure actual voltage
* ``MEAS:CURR? CH1`` - Measure actual current  
* ``OUTP? CH1`` - Query output state
* ``APPL? CH1`` - Query voltage/current setpoints

Commands That DO Switch Channel
================================

Only these commands in the remote control now switch the active channel:

* ``INST:NSEL {channel}`` + ``VOLT {value}`` - When setting voltage
* ``INST:NSEL {channel}`` + ``CURR {value}`` - When setting current

Testing
=======

The application should now:

* Display stable measurements without switching channels
* Only switch the PSU's active channel when user presses 'V' or 'C' to change settings
* Respond faster to user input due to reduced SCPI overhead
* Show correct setpoint values for all three channels

Related Files
=============

* ``src/remote_control/controller.rs`` - Main changes to ``update_channel()`` method
* ``src/remote_control/ui.rs`` - UI already had proper update intervals in place

See Also
========

* User request: "Do not switch channel on the PSU unless you set voltage or current"
* Previous optimization: Commit c103238 "Optimize remote control channel switching"
