Screenshots
===========

This directory contains screenshots of the DP832 Multitool applications.

Required Screenshots
--------------------

To complete the documentation, please add the following screenshots:

Battery Simulator
~~~~~~~~~~~~~~~~~

1. **battery-sim-single-channel.png**
   
   - Single channel battery simulation
   - Shows voltage/current/power graphs
   - SoC gauge visible
   - Event and SCPI log windows
   - Command: ``battery-sim --ip <IP> -p profiles/lifepo4.json``

2. **battery-sim-three-channels.png**
   
   - Three channel split view
   - Different battery profiles running
   - All graphs and gauges visible
   - Command: ``battery-sim --ip <IP> -p profiles/lifepo4.json -p profiles/liion_18650.json -p profiles/lipo_1s.json``

3. **battery-sim-log-windows.png**
   
   - Close-up of event and SCPI log windows
   - Shows typical log entries
   - Demonstrates auto-scrolling

Remote Control
~~~~~~~~~~~~~~

4. **remote-control-main.png**
   
   - Main remote control interface
   - Shows all three channels
   - Voltage and current setpoints visible
   - Channel status (ON/OFF) indicators
   - Command: ``remote-control --ip <IP>``

5. **remote-control-editing.png**
   
   - Shows voltage or current editing mode
   - Demonstrates input dialog
   - Selected channel highlighted

6. **remote-control-all-channels-on.png**
   
   - All three channels enabled
   - Real-time measurements visible
   - Power calculations shown

How to Capture Screenshots
---------------------------

Using the Capture Script (Recommended)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

A helper script is provided to simplify screenshot capture:

.. code-block:: bash

   # Interactive mode - will prompt for screenshot name
   ./screenshots/capture.sh

   # Direct mode - specify screenshot name
   ./screenshots/capture.sh battery-sim-three-channels

The script will:

- Auto-detect available screenshot tools (gnome-screenshot, scrot, or import)
- Provide a 5-second countdown
- Save the screenshot with correct naming
- Display file information
- Show commands to view and commit the screenshot

Using Terminal Screenshot Tools
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

For Linux systems, you can use various tools:

**Using gnome-screenshot:**

.. code-block:: bash

   # Capture entire window after 5 second delay
   gnome-screenshot -w -d 5 -f battery-sim-single-channel.png

**Using scrot:**

.. code-block:: bash

   # Capture selected window
   scrot -u -d 5 battery-sim-single-channel.png

**Using ImageMagick:**

.. code-block:: bash

   # Capture window by clicking
   import battery-sim-single-channel.png

Manual Process
~~~~~~~~~~~~~~

1. Start the application
2. Wait for it to stabilize and show meaningful data
3. Take a screenshot using your preferred tool
4. Crop to show just the terminal window
5. Save with the appropriate filename in this directory

Screenshot Naming Convention
-----------------------------

- Use lowercase with hyphens
- Include the tool name as prefix
- Be descriptive
- Use .png format for best quality

Examples:

- ``battery-sim-*.png`` - Battery simulator screenshots
- ``remote-control-*.png`` - Remote control screenshots
- ``ui-*.png`` - General UI elements

Image Format Guidelines
-----------------------

- **Format**: PNG (preferred) or JPEG
- **Size**: Keep width around 1200-1600px for readability
- **Quality**: High quality, clearly readable text
- **Content**: Show real data, not empty/initialization screens
- **Background**: Dark terminal background preferred for consistency

Notes
-----

Screenshots should demonstrate:

- Realistic usage scenarios
- All major UI components
- Both success and normal operation states
- Log windows with actual content
- Multiple channels when applicable
- Visual appeal of the terminal UI
