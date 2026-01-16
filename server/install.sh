#!/bin/bash
# =============================================================================
# Porta Server Installation Script
# =============================================================================
# This script installs the Porta server as a systemd service on Linux.
# Run with sudo: sudo ./install.sh
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/porta"
DATA_DIR="/var/lib/porta"
SERVICE_FILE="/etc/systemd/system/porta.service"
USER="porta"
GROUP="porta"

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
check_root() {
    if [ "$EUID" -ne 0 ]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Check if binary exists
check_binary() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    BINARY_PATH="$SCRIPT_DIR/target/release/porta-server"
    
    if [ ! -f "$BINARY_PATH" ]; then
        print_error "Binary not found: $BINARY_PATH"
        print_info "Please build the server first: cargo build --release"
        exit 1
    fi
}

# Create user and group
create_user() {
    if id "$USER" &>/dev/null; then
        print_info "User '$USER' already exists"
    else
        print_info "Creating user '$USER'..."
        useradd --system --no-create-home --shell /usr/sbin/nologin "$USER"
        print_success "User '$USER' created"
    fi
}

# Create directories
create_directories() {
    print_info "Creating directories..."
    
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$DATA_DIR"
    
    chown "$USER:$GROUP" "$DATA_DIR"
    chmod 750 "$DATA_DIR"
    
    print_success "Directories created"
}

# Install binary
install_binary() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    BINARY_PATH="$SCRIPT_DIR/target/release/porta-server"
    
    print_info "Installing binary..."
    
    cp "$BINARY_PATH" "$INSTALL_DIR/porta-server"
    chmod 755 "$INSTALL_DIR/porta-server"
    
    print_success "Binary installed to $INSTALL_DIR/porta-server"
}

# Install configuration
install_config() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    CONFIG_EXAMPLE="$SCRIPT_DIR/porta.toml.example"
    CONFIG_FILE="$CONFIG_DIR/porta.toml"
    
    if [ -f "$CONFIG_FILE" ]; then
        print_warning "Configuration file already exists: $CONFIG_FILE"
        print_info "Skipping configuration installation"
    else
        print_info "Installing configuration..."
        cp "$CONFIG_EXAMPLE" "$CONFIG_FILE"
        
        # Update default paths for production
        sed -i 's|path = "porta.db"|path = "/var/lib/porta/data.db"|' "$CONFIG_FILE"
        
        chmod 640 "$CONFIG_FILE"
        chown root:$GROUP "$CONFIG_FILE"
        
        print_success "Configuration installed to $CONFIG_FILE"
    fi
}

# Install systemd service
install_service() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    SERVICE_SRC="$SCRIPT_DIR/porta.service"
    
    print_info "Installing systemd service..."
    
    cp "$SERVICE_SRC" "$SERVICE_FILE"
    chmod 644 "$SERVICE_FILE"
    
    systemctl daemon-reload
    
    print_success "Systemd service installed"
}

# Enable and start service
enable_service() {
    print_info "Enabling service..."
    systemctl enable porta.service
    
    print_info "Starting service..."
    systemctl start porta.service
    
    sleep 2
    
    if systemctl is-active --quiet porta.service; then
        print_success "Porta service is running"
    else
        print_error "Service failed to start. Check logs: journalctl -u porta.service"
        exit 1
    fi
}

# Print status
print_status() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              Porta Server Installation Complete                   ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Binary:        $INSTALL_DIR/porta-server"
    echo "Configuration: $CONFIG_DIR/porta.toml"
    echo "Data:          $DATA_DIR/"
    echo "Service:       porta.service"
    echo ""
    echo "Commands:"
    echo "  Start:   sudo systemctl start porta"
    echo "  Stop:    sudo systemctl stop porta"
    echo "  Restart: sudo systemctl restart porta"
    echo "  Status:  sudo systemctl status porta"
    echo "  Logs:    sudo journalctl -u porta -f"
    echo ""
}

# Uninstall function
uninstall() {
    print_info "Uninstalling Porta server..."
    
    # Stop and disable service
    systemctl stop porta.service 2>/dev/null || true
    systemctl disable porta.service 2>/dev/null || true
    
    # Remove files
    rm -f "$SERVICE_FILE"
    rm -f "$INSTALL_DIR/porta-server"
    
    systemctl daemon-reload
    
    print_success "Porta server uninstalled"
    print_info "Configuration and data preserved in $CONFIG_DIR and $DATA_DIR"
    print_info "To remove user: userdel $USER"
    print_info "To remove data: rm -rf $CONFIG_DIR $DATA_DIR"
}

# Main
main() {
    case "${1:-install}" in
        install)
            echo ""
            echo -e "${BLUE}Porta Server Installation${NC}"
            echo ""
            
            check_root
            check_binary
            create_user
            create_directories
            install_binary
            install_config
            install_service
            enable_service
            print_status
            ;;
        uninstall)
            check_root
            uninstall
            ;;
        *)
            echo "Usage: $0 {install|uninstall}"
            exit 1
            ;;
    esac
}

main "$@"
