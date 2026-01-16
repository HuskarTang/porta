# Porta

Porta is a decentralized P2P network application built on [LibP2P](https://libp2p.io/) that enables secure tunneling for remote service access. It allows you to publish local services to a community network and subscribe to services shared by others, all through encrypted P2P connections.

[中文说明](README_CN.md)

## Features

- Service publishing for HTTP/HTTPS/TCP/WebSocket
- Service discovery and subscription
- Secure tunneling with Noise protocol
- Multi-hop relay routing
- Optional SOCKS5/HTTP proxy
- Community management
- Desktop GUI (Tauri + Vue 3) and headless server mode

## Packages

| Package | Description |
|--------|-------------|
| Desktop | Tauri GUI app with embedded backend |
| Server | Headless service mode for servers |

## Installation

### Desktop Application

Download the latest release for your platform:

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | [Porta.dmg](https://github.com/porta-app/porta/releases/latest) |
| macOS (Intel) | [Porta.dmg](https://github.com/porta-app/porta/releases/latest) |
| Windows | [Porta.msi](https://github.com/porta-app/porta/releases/latest) |
| Linux | [Porta.AppImage](https://github.com/porta-app/porta/releases/latest) |

### Server Binary

```bash
curl -LO https://github.com/porta-app/porta/releases/latest/download/porta-server-linux-x64.tar.gz
tar xzf porta-server-linux-x64.tar.gz
./porta-server --config porta.toml
```

### Systemd Service (Linux)

```bash
curl -LO https://github.com/porta-app/porta/releases/latest/download/porta-service-files.tar.gz
tar xzf porta-service-files.tar.gz
cd porta-service
sudo ./install.sh
```

## Quick Start

### Desktop App

1. Launch Porta
2. Configure node name in Settings
3. Join a community
4. Publish or discover services

### Server Mode

```toml
# porta.toml
[server]
listen_addr = "0.0.0.0"
port = 8090

[node]
name = "My Server"
role = "edge"

[database]
path = "/var/lib/porta/data.db"

[p2p]
tcp_port = 9000
quic_port = 9001
```

```bash
porta-server --config porta.toml
```

## Configuration

### Server Configuration (`porta.toml`)

| Section | Key | Default | Description |
|---------|-----|---------|-------------|
| `server` | `listen_addr` | `0.0.0.0` | HTTP API listen address |
| `server` | `port` | `8090` | HTTP API port |
| `node` | `name` | `Porta Server` | Node display name |
| `node` | `role` | `edge` | Node role: `edge` or `community` |
| `database` | `path` | `porta.db` | SQLite database path |
| `p2p` | `tcp_port` | `9000` | TCP transport port |
| `p2p` | `quic_port` | `9001` | QUIC transport port |
| `p2p` | `mdns_enable` | `true` | Enable mDNS discovery |
| `p2p` | `dht_enable` | `true` | Enable DHT discovery |
| `logging` | `level` | `info` | Log level |

### Command Line Options

```
porta-server [OPTIONS]

Options:
  -c, --config <FILE>    Path to configuration file [default: porta.toml]
      --listen <ADDR>    Override listen address
  -p, --port <PORT>      Override listen port
      --log-level <LVL>  Override log level
      --print-config     Print configuration and exit
      --validate         Validate configuration and exit
  -h, --help             Print help
  -V, --version          Print version
```

## Development

### Build from Source

```bash
npm install
npm run build:desktop
npm run build:server
```

### Testing

```bash
npm test
npm run test:system
```

## Project Structure

```
porta-app/
├── backend/       # Shared backend library
├── server/        # Headless server package
├── src-tauri/     # Desktop GUI package
└── frontend/      # Vue 3 frontend
```

## Contributing

We welcome contributions. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT. See [LICENSE](LICENSE).
