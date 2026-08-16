#!/bin/sh
set -eu

ADMIN_URL="http://garage:3903"
TOKEN="dev-garage-admin-token"

until curl -sf -H "Authorization: Bearer ${TOKEN}" "${ADMIN_URL}/v2/GetClusterStatus" >/tmp/status.json; do
  echo "waiting for garage admin api..."
  sleep 1
done

NODE_ID=$(jq -r '.nodes[0].id' /tmp/status.json)
LAYOUT_VERSION=$(jq -r '.layoutVersion' /tmp/status.json)

EXISTING_ROLES=$(curl -sf -H "Authorization: Bearer ${TOKEN}" "${ADMIN_URL}/v2/GetClusterLayout" | jq -r '.roles | length')

if [ "$EXISTING_ROLES" -gt 0 ]; then
  echo "layout already assigned"
  exit 0
fi

echo "assigning layout to node ${NODE_ID}"

curl -sf -X POST \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{\"roles\":[{\"id\":\"${NODE_ID}\",\"zone\":\"dev\",\"capacity\":1000000000,\"tags\":[]}]}" \
  "${ADMIN_URL}/v2/UpdateClusterLayout"

APPLY_VERSION=$((LAYOUT_VERSION + 1))

curl -sf -X POST \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{\"version\":${APPLY_VERSION}}" \
  "${ADMIN_URL}/v2/ApplyClusterLayout"

echo "garage bootstrap complete"
