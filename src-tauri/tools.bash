#!/bin/bash

# Function to display the menu
show_menu() {
    echo "================================================"
    echo "            SQLx Management Script"
    echo "================================================"
    echo "1. Update sqlx cache"
    echo "2. Create .env file"
    echo "3. Exit"
    echo "================================================"
}

# Function to update sqlx cache
update_sqlx_cache() {
    echo "Updating sqlx cache..."
    echo "Running: cargo sqlx prepare --workspace -- --all-targets --all-features"
    cargo sqlx prepare --workspace -- --all-targets --all-features
    
    if [ $? -eq 0 ]; then
        echo "✅ SQLx cache updated successfully!"
    else
        echo "❌ Failed to update SQLx cache. Please check your cargo and sqlx installation."
    fi
    echo
}

# Function to create .env file
create_env_file() {
    echo "Creating .env file..."
    echo -n "Please enter the database file location (e.g., ./database.db or /path/to/database.db): "
    read -r db_location
    
    # Validate input
    if [ -z "$db_location" ]; then
        echo "❌ Database location cannot be empty!"
        return 1
    fi
    
    # Create .env file content
    env_content="DATABASE_URL=\"sqlite:///$db_location?mode=rwc\""
    
    # Check if .env file already exists
    if [ -f ".env" ]; then
        echo "⚠️  .env file already exists. Do you want to overwrite it? (y/N):"
        read -r confirm
        if [[ ! $confirm =~ ^[Yy]$ ]]; then
            echo "Operation cancelled."
            return 1
        fi
    fi
    
    # Write to .env file
    echo "$env_content" > .env
    
    if [ $? -eq 0 ]; then
        echo "✅ .env file created successfully!"
        echo "Content: $env_content"
    else
        echo "❌ Failed to create .env file."
    fi
    echo
}

show_menu
echo -n "Please select an option (1-3): "
read -r choice

case $choice in
    1)
        update_sqlx_cache
        ;;
    2)
        create_env_file
        ;;
    3)
        exit 0
        ;;
    *)
        echo "❌ Invalid option. Please select 1, 2, or 3."
        echo
        ;;
esac
