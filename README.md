# CWMP Listener

[![Release](https://github.com/zeroows/cwmp_listener/actions/workflows/release.yaml/badge.svg?event=release)](https://github.com/zeroows/cwmp_listener/actions/workflows/release.yaml)

A simple, secure CWMP (TR-069) listener that monitors and logs incoming ACS (Auto Configuration Server) communications. This tool is useful for debugging and monitoring TR-069 device interactions.

## Features

- Listen for CWMP/TR-069 connections (default port: 7547)
- Basic authentication support
- Configurable timeout settings
- Detailed logging of incoming connections and messages
- Customizable host and port settings
- Environment variable support through `.env` files

## Installation

1. Ensure you have Rust installed on your system. If not, install it from [rustup.rs](https://rustup.rs/)

2. Clone this repository:
```bash
git clone <repository-url>
cd cwmp-listener
```

3. Build the application:
```bash
cargo build -r
```

4. Run the application:
```bash
cargo run
```

## Configuration

The application can be configured through a YAML configuration file or environment variables.

### Configuration File (config/default.yml)

```yaml
application:
  port: 7547
  host: 0.0.0.0
  log_lvl: info
  timeout: 5
auth:
  username: admin
  password: password
```

### Environment Variables

You can also use environment variables by creating a `.env` file:

```env
APP_HOST=0.0.0.0
APP_PORT=7547
APP_TIMEOUT=30
APP_LOG_LVL=debug
AUTH_USERNAME=admin
AUTH_PASSWORD=password
```

## Usage

1. Start the listener:

```bash
./target/release/cwmp-listener
```

With custom config folder:
```bash
./target/release/cwmp-listener --config-folder /path/to/config
```

2. The listener will start monitoring the specified port (default: 7547) for incoming CWMP connections.

3. All incoming connections and messages will be logged according to the configured log level.

## Authentication

The listener supports Basic Authentication. Clients must provide valid credentials in the Authorization header to connect. If no authentication is configured, all connections are allowed.

## Logging

Logs include:
- Connection attempts
- Authentication attempts
- Received messages
- Connection timeouts
- Errors and warnings

Log levels can be configured through the configuration file or environment variables.

## Development

Requirements:
- Rust 1.70 or higher
- Cargo package manager

Build the project:
```bash
cargo build
```

Run tests:
```bash
cargo test
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
