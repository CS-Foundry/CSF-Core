#!/bin/sh
set -eu

GARAGE="garage -c /etc/garage.toml"

until $GARAGE status >/dev/null 2>&1; do
  echo "waiting for garage rpc..."
  sleep 1
done

NODE_ID=$($GARAGE status | awk '/NODE ID/{found=1; next} found && NF {print $1; exit}')
if [ -z "$NODE_ID" ]; then
  NODE_ID=$($GARAGE node id -q | cut -d'@' -f1)
fi

if $GARAGE layout show | grep -q "No nodes"; then
  echo "assigning layout to node ${NODE_ID}"
  $GARAGE layout assign -z dev -c 1G "$NODE_ID"
  $GARAGE layout apply --version 1
else
  echo "layout already assigned"
fi

echo "garage bootstrap complete"
