#!/bin/sh
# Railway mounts persistent /data as root; initialize its ownership with
# narrowly scoped root privileges, then run the workload as its service user.
set -eu
if [ "$#" -lt 2 ]; then
  echo "usage: railway-volume-entrypoint USER COMMAND [ARGS...]" >&2
  exit 64
fi
service_user="$1"
shift
case "$service_user" in
  zagros|helix) ;;
  *) echo "unsupported Railway runtime user" >&2; exit 64 ;;
esac
if [ "$(id -u)" -ne 0 ]; then
  echo "Railway volume initialization requires root only at startup" >&2
  exit 77
fi
if [ ! -d /data ] || [ -L /data ]; then
  echo "Missing or unsafe /data mount" >&2
  exit 78
fi
chown -R "$service_user:$service_user" /data
exec runuser -u "$service_user" -- "$@"
