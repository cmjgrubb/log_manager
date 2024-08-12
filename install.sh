#!/bin/bash

set -e

# Install dependencies
sudo apt update
sudo apt install -y mariadb-server git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
curl -fsSL https://bun.sh/install | bash

# Download the project from GitHub
sudo mkdir /log_manager
sudo chown $USER:$USER /log_manager
cd /log_manager
git clone https://github.com/cmjgrubb/log_manager.git .

# Build the project
## Database
sudo systemctl start mariadb
sudo systemctl enable mariadb

read -sp "Enter MariaDB root password: " ROOT_PASS
echo
read -p "Enter new MariaDB username: " DB_USER
read -sp "Enter new MariaDB user password: " DB_PASS
echo

mysql -u root -p$ROOT_PASS -e "
CREATE USER IF NOT EXISTS '$DB_USER'@'localhost' IDENTIFIED BY '$DB_PASS';
GRANT ALL PRIVILEGES ON log_database.* TO '$DB_USER'@'localhost';
FLUSH PRIVILEGES;
"

if [ ! -f database/schema.sql ]; then
    echo "database/schema.sql file not found!"
    exit 1
fi

mysql -u $DB_USER -p$DB_PASS < database/schema.sql
echo "Users, database, and table created successfully."

cat <<EOL > .env
DATABASE_URL=mysql://$DB_USER:$DB_PASS@localhost:3306/log_database
EOL

echo ".env file created successfully."

## Log Processor
cd log_processor
cargo build --release
cargo run --release
cd ..

## Log API
cd log_api
cargo build --release
cargo run --release
cd ..

## Website
cd website
bun install
bun run
