#!/bin/sh
set -eu

LOG_FILE="${ZAGROS_REFRESH_LOG:-/data/refresh-history.jsonl}"
STATUS_FILE="${ZAGROS_REFRESH_STATUS:-/data/refresh-status.json}"
CVE_LIMIT="${ZAGROS_DAILY_CVE_LIMIT:-1000}"

started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
printf '{"event":"refresh_started","at":"%s","cve_limit":%s}\n' "$started_at" "$CVE_LIMIT" >> "$LOG_FILE"

if /usr/local/bin/zagros ingest --limit "$CVE_LIMIT" && /usr/local/bin/zagros source all; then
    finished_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    printf '{"event":"refresh_succeeded","at":"%s"}\n' "$finished_at" >> "$LOG_FILE"
    printf '{"status":"ok","last_attempt":"%s","last_success":"%s"}\n' "$finished_at" "$finished_at" > "$STATUS_FILE"
    exit 0
fi

finished_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
printf '{"event":"refresh_failed","at":"%s"}\n' "$finished_at" >> "$LOG_FILE"
printf '{"status":"failed","last_attempt":"%s"}\n' "$finished_at" > "$STATUS_FILE"
exit 1
