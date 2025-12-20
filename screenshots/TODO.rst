TODO: Screenshots
=================

The following screenshot files are referenced in the documentation but need to be captured:

Required Files
--------------

Battery Simulator
~~~~~~~~~~~~~~~~~

1. ``battery-sim-single-channel.png``
   - Single channel simulation
   - Run: ``battery-sim --ip <IP> -p profiles/lifepo4.json``

2. ``battery-sim-three-channels.png``
   - Three channel split view
   - Run: ``battery-sim --ip <IP> -p profiles/lifepo4.json -p profiles/liion_18650.json -p profiles/lipo_1s.json``

3. ``battery-sim-log-windows.png`` (optional)
   - Close-up of log windows

Remote Control
~~~~~~~~~~~~~~~

4. ``remote-control-main.png``
   - Main interface view
   - Run: ``remote-control --ip <IP>``

5. ``remote-control-editing.png`` (optional)
   - Shows editing dialog

6. ``remote-control-all-channels-on.png`` (optional)
   - All channels enabled

Quick Capture Instructions
---------------------------

1. Build the project:
   
   .. code-block:: bash
   
      cargo build --release

2. Connect to your DP832 (replace IP address):

   For battery simulator:
   
   .. code-block:: bash
   
      # Three channels
      ./target/release/battery-sim --ip 192.168.1.100 \
        -p profiles/lifepo4.json \
        -p profiles/liion_18650.json \
        -p profiles/lipo_1s.json

   For remote control:
   
   .. code-block:: bash
   
      ./target/release/remote-control --ip 192.168.1.100

3. Wait for the UI to show meaningful data (graphs filled, measurements active)

4. Take screenshot using your preferred tool:

   .. code-block:: bash
   
      # Using gnome-screenshot (delay 5 seconds)
      gnome-screenshot -w -d 5 -f screenshots/battery-sim-three-channels.png
      
      # Or using scrot
      scrot -u -d 5 screenshots/battery-sim-three-channels.png

5. Verify image looks good:

   .. code-block:: bash
   
      # View the image
      xdg-open screenshots/battery-sim-three-channels.png

Notes
-----

- Use PNG format for best quality
- Keep width around 1200-1600px
- Dark terminal background preferred
- Show real data, not initialization screens
- Make sure text is clearly readable
- Crop to show just the terminal window

Once screenshots are captured, the documentation will display them automatically when built with Sphinx.
