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


# Function to stop, disable, and remove a service if it exists
remove_service() {
    local service_name=$1
    local service_file="/etc/systemd/system/${service_name}.service"

    if [ -f "$service_file" ]; then
        echo "Stopping ${service_name} service..."
        sudo systemctl stop "${service_name}.service" || echo "Failed to stop ${service_name} service."

        echo "Disabling ${service_name} service..."
        sudo systemctl disable "${service_name}.service" || echo "Failed to disable ${service_name} service."

        echo "Removing ${service_name} service file..."
        sudo rm "$service_file" || echo "Failed to remove ${service_name} service file."
    else
        echo "${service_name} service is not installed."
    fi
}

# Uninstall the services
remove_service "log_processor"
remove_service "log_api"
remove_service "website"

# Reload systemd to apply the changes
sudo systemctl daemon-reload

echo "Services log_processor and log_api have been uninstalled."

# Prompt for confirmation before removing dependencies
if confirm "Do you want to remove the installed dependencies (mariadb-server, mariadb-client, git, unzip, build-essential, pkg-config, libssl-dev, nodejs, rustc, cargo)? [y/N]"; then
    sudo apt remove --purge -y mariadb-server mariadb-client git unzip build-essential pkg-config libssl-dev nodejs rustc cargo || { echo "Failed to remove apt dependencies."; exit 1; }
    sudo apt autoremove -y
    echo "Removed apt dependencies."
else
    echo "Apt dependencies have not been removed."
fi

# Remove NVM, Rust, and Bun if installed
if confirm "Do you want to remove Bun? [y/N]"; then
    cd $HOME

    if [ -d "/usr/local/bin/bun" ]; then
        rm -rf "/usr/local/bin/bun"
        echo "Removed Bun."
    else
        echo "Bun is not installed."
    fi

else
    echo "Bun not removed."
fi

# Remove log file
if [ -f /var/log/log_manager ]; then
    sudo rm -f /var/log/log_manager || echo "Failed to remove log file."
else
    echo "Log file does not exist."
fi

# Remove the log_manager user and group
if id "log_manager" &>/dev/null; then
    sudo gpasswd -d $USER log_manager || echo "Failed to remove $USER from log_manager group."
    sudo userdel log_manager || { echo "Failed to delete log_manager user."; exit 1; }
else
    echo "User log_manager does not exist."
fi

if getent group log_manager > /dev/null; then
    sudo groupdel log_manager || { echo "Failed to delete log_manager group."; exit 1; }
else
    echo "Group log_manager does not exist."
fi

# Remove the log_manager service user home directory
if [ -d "/home/log_manager" ]; then
    sudo rm -rf /home/log_manager
    echo "Removed /home/log_manager directory."
else
    echo "/home/log_manager directory does not exist."
fi

echo "Uninstallation complete."