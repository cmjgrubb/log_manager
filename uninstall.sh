#!/bin/bash

set -e

cd $HOME

# Uninstall Rust
if [ -d "$HOME/.cargo" ]; then
    source $HOME/.cargo/env
    rustup self uninstall
    echo "Removed Rust."
else
    echo "Rust is not installed."
fi

# Uninstall Bun
if [ -d "$HOME/.bun" ]; then
    rm -rf "$HOME/.bun"
    echo "Removed Bun."
else
    echo "Bun is not installed."
fi

# Uninstall NVM (if applicable)
if [ -d "$HOME/.nvm" ]; then
    rm -rf "$HOME/.nvm"
    echo "Removed NVM."
else
    echo "NVM is not installed."
fi

# Remove log_manager user and group
if id "log_manager" &>/dev/null; then
    sudo userdel -r log_manager
    echo "Removed log_manager user."
else
    echo "log_manager user does not exist."
fi

if getent group log_manager > /dev/null; then
    sudo groupdel log_manager
    echo "Removed log_manager group."
else
    echo "log_manager group does not exist."
fi

# Remove /log_manager directory
if [ -d "/log_manager" ]; then
    sudo rm -rf /log_manager
    echo "Removed /log_manager directory."
else
    echo "/log_manager directory does not exist."
fi

echo "Uninstallation completed."