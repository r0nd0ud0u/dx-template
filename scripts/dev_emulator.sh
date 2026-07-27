#!/bin/bash
# Boots an Android emulator for local `dx serve --platform android` testing.
# Usage: ./scripts/dev_emulator.sh <avd-name> [--wipe] [--fresh]
# List available AVDs with: emulator -list-avds
#
# --wipe factory-resets the AVD's data partition before boot — use this when `adb install`
# fails with INSTALL_FAILED_INSUFFICIENT_STORAGE (a dev AVD's small default storage fills up
# after enough install/uninstall cycles) or when you just want a known-clean device state.
#
# --fresh skips resuming the saved quick-boot snapshot (cold boot instead) — use this when the
# emulator window comes up black/unresponsive or adb reports the device stuck 'offline'. Unlike
# --wipe, this keeps installed apps and app data; it only discards the suspended session state.
AVD="${1:?Usage: $0 <avd-name> [--wipe] [--fresh]  (list available AVDs with: emulator -list-avds)}"
shift
EXTRA_ARGS=()
for arg in "$@"; do
    case "$arg" in
        --wipe) EXTRA_ARGS+=(-wipe-data) ;;
        --fresh) EXTRA_ARGS+=(-no-snapshot-load) ;;
        *) echo "unknown option: $arg (expected --wipe and/or --fresh)" >&2; exit 1 ;;
    esac
done
emulator -avd "$AVD" "${EXTRA_ARGS[@]}"
