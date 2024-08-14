#!/bin/bash

set -e

# Function to prompt for confirmation
confirm() {
    read -r -p "${1:-Are you sure? [y/N]} " response
    case "$response" in
        [yY][eE][sS]|[yY]) 
            true
            ;;
        *)
            false
            ;;
    esac
}

# Check for root privileges
if [ "$EUID" -ne 0 ]; then
    echo "This script must be run with root privileges."
    exit 1
fi

# Remove the GitHub project
if [ -d "/log_manager" ]; then
    sudo rm -rf /log_manager
    echo "Removed /log_manager directory."
else
    echo "/log_manager directory does not exist."
fi

# Prompt for confirmation before removing dependencies
if confirm "Do you want to remove the installed dependencies (mariadb-server, mariadb-client, git, unzip, build-essential, pkg-config, libssl-dev)? [y/N]"; then
    sudo apt remove --purge -y mariadb-server mariadb-client git unzip build-essential pkg-config libssl-dev
    sudo apt autoremove -y
    echo "Removed dependencies."
else
    echo "Dependencies have not been removed."
fi

# Remove Rust and Bun if installed
if confirm "Do you want to remove Rust and Bun? [y/N]"; then
    if [ -d "$HOME/.cargo" ]; then
        source $HOME/.cargo/env
        rustup self uninstall
        echo "Removed Rust."
    else
        echo "Rust is not installed."
    fi

    if [ -d "$HOME/.bun" ]; then
        rm -rf "$HOME/.bun"
        echo "Removed Bun."
    else
        echo "Bun is not installed."
    fi
else
    echo "Rust and Bun not removed."
fi

echo "Uninstallation complete."