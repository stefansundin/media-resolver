#!/bin/bash -e

if [[ "$LOG_EXPORT_ENABLED" == "true" ]]; then
  if [[ -z "$LOG_EXPORT_SUBJECT" ]]; then
    # By default only export the container's own logs, important if you're running multiple containers!
    export LOG_EXPORT_SUBJECT="logs.${FLY_APP_NAME}.${FLY_REGION}.${FLY_ALLOC_ID:0:8}"
  fi
  /vector.sh &
fi

exec /media-resolver
