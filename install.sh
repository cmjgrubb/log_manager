#!/bin/bash

set -e

# Check for root privileges
if [ "$EUID" -ne 0 ]; then
   echo "This script must be run with root privileges."
   exit 1
fi

# Install dependencies
sudo apt update && sudo apt install -y mariadb-server mariadb-client git unzip build-essential pkg-config libssl-dev || { echo "Failed to install dependencies."; exit 1; }
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y || { echo "Failed to install Rust."; exit 1; }
source $HOME/.cargo/env
curl -fsSL https://bun.sh/install | bash || { echo "Failed to install Bun."; exit 1; }
export PATH="$HOME/.bun/bin:$PATH"

# Download the project from GitHub
sudo mkdir -p /log_manager
sudo chown $USER:$USER /log_manager
cd /log_manager
git clone https://github.com/cmjgrubb/log_manager.git . || { echo "Failed to clone GitHub repository"; exit 1; }

# Build the project
## Database
sudo systemctl start mariadb || { echo "Failed to start MariaDB."; exit 1; }
sudo systemctl enable mariadb || { echo "Failed to enable MariaDB."; exit 1; }

read -sp "Enter MariaDB root password: " ROOT_PASS
echo
read -p "Enter new MariaDB service account: " DB_USER
read -sp "Enter new MariaDB user password: " DB_PASS
echo

mysql -u root -p"$ROOT_PASS" -e "
CREATE DATABASE IF NOT EXISTS log_database;
CREATE USER IF NOT EXISTS '$DB_USER'@'localhost' IDENTIFIED BY '$DB_PASS';
GRANT ALL PRIVILEGES ON log_database.* TO '$DB_USER'@'localhost';
FLUSH PRIVILEGES;
" || { echo "Failed to create database, service account, or grant privileges."; exit 1; }

if [ ! -f database/schema.sql ]; then
    echo "database/schema.sql file not found!"
    exit 1
fi

mysql -u "$DB_USER" -p"$DB_PASS" log_database < database/schema.sql || { echo "Failed to import database schema."; exit 1; }
echo "Users, database, and table created successfully."

cat <<EOL > .env
DATABASE_URL="mysql://$DB_USER:$DB_PASS@localhost:3306/log_database"
EOL

echo ".env file created successfully."
source .env

## Log Processor
cd log_processor
cargo build --release || { echo "Failed to build log_processor"; exit 1; }
cd ..

## Log API
cd log_api
cargo build --release || { echo "Failed to build log_api"; exit 1; }
cd ..

## Website
cd website
bun install || { echo "Failed to install website dependencies"; exit 1; }

# Create systemd service files
## Log Processor service
sudo bash -c 'cat <<EOL > /etc/systemd/system/log_processor.service
[Unit]
Description=Log Processor Service
After=network.target

[Service]
ExecStart=/log_manager/log_processor/target/release/log_processor
WorkingDirectory=/log_manager/log_processor
Restart=always
User=$USER
EnvironmentFile=/log_manager/.env

[Install]
WantedBy=multi-user.target
EOL'

## Log API service
sudo bash -c 'cat <<EOL > /etc/systemd/system/log_api.service
[Unit]
Description=Log API Service
After=network.target

[Service]
ExecStart=/log_manager/log_api/target/release/log_api
WorkingDirectory=/log_manager/log_api
Restart=always
User=$USER
EnvironmentFile=/log_manager/.env

[Install]
WantedBy=multi-user.target
EOL'

## Website service
sudo bash -c 'cat <<EOL > /etc/systemd/system/website.service
[Unit]
Description=Website Service
After=network.target

[Service]
ExecStart=/usr/local/bin/bun run
WorkingDirectory=/log_manager/website
Restart=always
User=$USER
EnvironmentFile=/log_manager/.env

[Install]
WantedBy=multi-user.target
EOL'

## Reload systemd to apply the new service files
sudo systemctl daemon-reload

## Enable and start the services
sudo systemctl enable log_processor.service
sudo systemctl start log_processor.service

sudo systemctl enable log_api.service
sudo systemctl start log_api.service

sudo systemctl enable website.service
sudo systemctl start website.service