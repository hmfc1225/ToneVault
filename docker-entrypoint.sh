#!/bin/sh
set -e

# Generate config from environment variables if config file doesn't exist
CONFIG_FILE="${TONEVAULT_CONFIG:-/app/config/tonevault.toml}"

if [ ! -f "$CONFIG_FILE" ]; then
    mkdir -p "$(dirname "$CONFIG_FILE")"
    cat > "$CONFIG_FILE" <<EOF
[server]
host = "${TONEVAULT_HOST:-0.0.0.0}"
port = ${TONEVAULT_PORT:-3000}

[database]
engine = "${TONEVAULT_DB_ENGINE:-sqlite}"
sqlite_path = "${TONEVAULT_DB_PATH:-/app/data/tonevault.db}"

[auth]
jwt_secret = "${TONEVAULT_AUTH_SECRET:-change-me-in-production}"
jwt_expiry_hours = ${TONEVAULT_AUTH_EXPIRY_HOURS:-1}
refresh_expiry_days = ${TONEVAULT_AUTH_REFRESH_DAYS:-7}

[scanner]
scan_on_startup = ${TONEVAULT_SCAN_ON_STARTUP:-true}
watch_for_changes = ${TONEVAULT_WATCH:-true}
scan_interval_minutes = ${TONEVAULT_SCAN_INTERVAL:-60}

[webdav]
enabled = ${TONEVAULT_WEBDAV_ENABLED:-true}
mount_path = "/dav"

[logging]
level = "${TONEVAULT_LOG_LEVEL:-info}"
EOF
    echo "Generated config at $CONFIG_FILE from environment variables"
fi

exec /app/tonevault-server
