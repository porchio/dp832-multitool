#!/bin/bash
# SPDX-License-Identifier: GPL-2.0-or-later
# Copyright (C) 2025 Marcus Folkesson
#
# Helper script to capture screenshots for documentation
# Usage: ./screenshots/capture.sh [screenshot-name]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}DP832 Multitool Screenshot Capture Helper${NC}"
echo "=========================================="
echo ""

# Check for screenshot tools
if ! command -v gnome-screenshot &> /dev/null && ! command -v scrot &> /dev/null && ! command -v import &> /dev/null; then
    echo -e "${RED}Error: No screenshot tool found!${NC}"
    echo "Please install one of the following:"
    echo "  - gnome-screenshot (GNOME)"
    echo "  - scrot (lightweight)"
    echo "  - imagemagick (import command)"
    exit 1
fi

# Determine which tool to use
if command -v gnome-screenshot &> /dev/null; then
    TOOL="gnome-screenshot"
elif command -v scrot &> /dev/null; then
    TOOL="scrot"
elif command -v import &> /dev/null; then
    TOOL="import"
fi

echo -e "Using screenshot tool: ${GREEN}$TOOL${NC}"
echo ""

# Show available screenshots to capture
echo "Required screenshots:"
echo "  1. battery-sim-single-channel.png"
echo "  2. battery-sim-three-channels.png"
echo "  3. remote-control-main.png"
echo ""
echo "Optional screenshots:"
echo "  4. battery-sim-log-windows.png"
echo "  5. remote-control-editing.png"
echo "  6. remote-control-all-channels-on.png"
echo ""

# Get filename
if [ $# -eq 0 ]; then
    echo -e "${YELLOW}Enter screenshot name (without .png):${NC}"
    read -p "> " NAME
else
    NAME="$1"
fi

# Add .png extension if not present
if [[ ! "$NAME" =~ \.png$ ]]; then
    NAME="${NAME}.png"
fi

FILEPATH="$SCRIPT_DIR/$NAME"

echo ""
echo -e "${YELLOW}Instructions:${NC}"
echo "1. Make sure your application window is visible and showing meaningful data"
echo "2. The screenshot will be taken in 5 seconds"
echo "3. Click on the application window when prompted (if required)"
echo ""
echo -e "Screenshot will be saved to: ${GREEN}$FILEPATH${NC}"
echo ""
read -p "Press ENTER to start the 5-second countdown..."

# Capture screenshot based on available tool
case $TOOL in
    gnome-screenshot)
        echo "Get ready... (select window if prompted)"
        sleep 1
        gnome-screenshot -w -d 5 -f "$FILEPATH"
        ;;
    scrot)
        echo "Get ready... Click on the window you want to capture"
        sleep 1
        scrot -u -d 5 "$FILEPATH"
        ;;
    import)
        echo "Get ready... Click on the window you want to capture"
        sleep 5
        import "$FILEPATH"
        ;;
esac

if [ -f "$FILEPATH" ]; then
    echo ""
    echo -e "${GREEN}✓ Screenshot captured successfully!${NC}"
    echo -e "  File: $FILEPATH"
    
    # Show file size
    SIZE=$(du -h "$FILEPATH" | cut -f1)
    echo -e "  Size: $SIZE"
    
    # Show image dimensions if identify is available
    if command -v identify &> /dev/null; then
        DIMS=$(identify -format "%wx%h" "$FILEPATH" 2>/dev/null || echo "unknown")
        echo -e "  Dimensions: $DIMS"
    fi
    
    echo ""
    echo "To view the screenshot:"
    echo "  xdg-open $FILEPATH"
    echo ""
    echo "To add to git:"
    echo "  git add $FILEPATH"
    echo "  git commit -m 'docs: Add $NAME screenshot'"
    
else
    echo -e "${RED}✗ Screenshot capture failed!${NC}"
    exit 1
fi
