#!/bin/sh
set -eu

if [ "${ZAGROS_AUTO_REFRESH:-true}" = "true" ]; then
    schedule="${ZAGROS_REFRESH_CRON:-0 3 * * *}"
    cron_file="/data/zagros-crontab"
    printf '%s /usr/local/bin/zagros-refresh\n' "$schedule" > "$cron_file"
    echo "[zagros] daily refresh enabled: $schedule UTC" >&2
    /usr/local/bin/supercronic "$cron_file" &
else
    echo "[zagros] daily refresh disabled" >&2
fi

exec /usr/local/bin/zagros-mcp-http "$@"
