# Haybox Debugger

A desktop application built with Tauri and Vue.js for managing and debugging Haybox controller connections and drivers.

## Features

- Real-time device connection monitoring for:
  - Default Mode (VID: 0x2E8A, PID: 0x0004)
  - Config Mode (VID: 0x2E8A, PID: 0x000A)
  - BOOTSEL Mode (VID: 0x2E8A, PID: 0x0003)
  - Switch Mode (VID: 0x2E8A, PID: 0x0005)
- GameCube adapter support:
  - Connection status monitoring
  - WinUSB driver installation
- XInput driver management:
  - Installation status monitoring
  - Driver installation/uninstallation
  - Driver reinstallation for troubleshooting

