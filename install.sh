#!/bin/bash

set -e

# Check for root privileges
if [ "$EUID" -ne 0 ]; then
   echo "This script must be run with root privileges."
   exit 1
fi

# Install dependencies
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo bash - || { echo "Failed to install add Nodesource repository."; exit 1; }
sudo apt update && sudo apt install -y mariadb-server mariadb-client git unzip build-essential pkg-config libssl-dev nodejs rustc cargo || { echo "Failed to install dependencies."; exit 1; }
curl -fsSL https://bun.sh/install | bash || { echo "Failed to install Bun."; exit 1; }
sudo mv /root/.bun/bin/bun /usr/local/bin/ || { echo "Failed to move Bun to /usr/local/bin."; exit 1; }
sudo chmod a+x /usr/local/bin/bun || { echo "Failed to update Bun permissions."; exit 1; }
sudo rm -rf /root/.bun || { echo "Failed to remove /root/.bun directory."; exit 1; }

# Create a dedicated service account and group
if ! getent group log_manager > /dev/null; then
    sudo groupadd -r log_manager || { echo "Failed to create service group."; exit 1; }
else
    echo "Group log_manager already exists."
fi

if ! id -u log_manager > /dev/null 2>&1; then
    sudo useradd -r -g log_manager -s /bin/false log_manager || { echo "Failed to create service account."; exit 1; }
else
    echo "User log_manager already exists."
fi

# Add the current user to the log_manager group
sudo usermod -aG log_manager $USER || { echo "Failed to add user to log_manager group."; exit 1; }

# Download the project from GitHub
sudo mkdir -p /log_manager
sudo chown log_manager:log_manager /log_manager
sudo chmod 770 /log_manager
cd /log_manager
sudo -u log_manager git clone https://github.com/cmjgrubb/log_manager.git . || { echo "Failed to clone GitHub repository"; exit 1; }

# Build the project
## Database
sudo systemctl start mariadb || { echo "Failed to start MariaDB."; exit 1; }
sudo systemctl enable mariadb || { echo "Failed to enable MariaDB."; exit 1; }

read -sp "Enter MariaDB root password: " ROOT_PASS
echo
read -p "Enter new MariaDB service account username: " DB_USER
read -sp "Enter new MariaDB service account password: " DB_PASS
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

## Set up .cargo directory in home of log_manager service account
sudo mkdir -p /home/log_manager/.cargo
sudo chown -R log_manager:log_manager /home/log_manager/.cargo

## Log Processor
cd /log_manager/log_processor
sudo -u log_manager cargo build --release || { echo "Failed to build log_processor."; exit 1; }

## Log API
cd /log_manager/log_api
sudo -u log_manager cargo build --release || { echo "Failed to build log_api."; exit 1; }

## Website
cd /log_manager/website
sudo -u log_manager bun install || { echo "Failed to install website dependencies."; exit 1; }
sudo -u log_manager bun pm trust --all || { echo "Failed to run bun pm trust."; exit 1; }
sudo -u log_manager bun run build || { echo "Failed to build website."; exit 1; }


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
User=log_manager
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
User=log_manager
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
ExecStart=/usr/local/bin/bun ./build/index.js
WorkingDirectory=/log_manager/website
Restart=always
User=log_manager
EnvironmentFile=/log_manager/.env

[Install]
WantedBy=multi-user.target
EOL'

# Set permissions for the service account
sudo chown -R log_manager:log_manager /log_manager
sudo chmod -R 770 /log_manager

# Create log file for the log_manager application
sudo touch /var/log/log_manager
sudo chown log_manager:log_manager /var/log/log_manager
sudo chmod 644 /var/log/log_manager

# Enable log_processor to communicate on port 514
sudo setcap 'cap_net_bind_service=+ep' /log_manager/log_processor/target/release/log_processor || { echo "Failed to enable log_processor to communicate on port 514."; exit 1; }

## Reload systemd to apply the new service files
sudo systemctl daemon-reload

## Enable and start the services
sudo systemctl enable log_processor.service
sudo systemctl start log_processor.service

sudo systemctl enable log_api.service
sudo systemctl start log_api.service

sudo systemctl enable website.service
sudo systemctl start website.service

echo "Services log_processor, log_api, and website have been installed and started."
echo "Installation and setup completed successfully."