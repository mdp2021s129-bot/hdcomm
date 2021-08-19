# `hdcomm`

Host to device (I/O expander) communication for MDP.

Provides:
    - A RPC channel with the device acting as an RPC server and the host
      as an RPC client.
    - A stream channel allowing the device to transmit messages to the
      host without a prior request.

# Components

- `hdcomm-core`: shared functionality beteween host & device.
- `hdcomm-device`: device-specific functionality.
- `hdcomm-host`: host-specific functionality.

# Assumptions

Transport takes place over a serial port.
