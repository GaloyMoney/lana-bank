#!/usr/bin/env bash
set -euo pipefail

##############################################################################
# Settings you might tweak
##############################################################################
REALM="internal"                # change to 'internal' or 'customer' if you prefer
CLIENT_ID="backend-service"   # the service client we’re creating/updating
##############################################################################

# Helper: pick the right admin-API client inside this realm
if [[ "$REALM" == "master" ]]; then
  MGMT_CLIENT_ID="${REALM}-realm"          # → master-realm
else
  MGMT_CLIENT_ID="realm-management"
fi

##############################################################################
# 1) Create the service client if it does not yet exist
##############################################################################
if ! kcadm.sh get clients -r "$REALM" -q clientId="$CLIENT_ID" | grep -q '"id"'; then
  kcadm.sh create clients -r "$REALM" \
      -s clientId="$CLIENT_ID" \
      -s protocol=openid-connect \
      -s publicClient=false \
      -s serviceAccountsEnabled=true \
      -s standardFlowEnabled=false \
      -s directAccessGrantsEnabled=false \
      -s bearerOnly=false
fi

##############################################################################
# 2) Fetch the new (or existing) client’s UUID and secret
##############################################################################
CID=$(kcadm.sh get clients -r "$REALM" -q clientId="$CLIENT_ID" \
          --fields id --format csv | tr -d '"\r')

##############################################################################
# 3) Grant the least-privilege roles to the **service-account user**
##############################################################################
SA_USER=$(kcadm.sh get clients/$CID/service-account-user -r "$REALM" \
              --fields username --format csv | tr -d '"\r')

kcadm.sh add-roles -r "$REALM" \
      --uusername "$SA_USER" \
      --cclientid "$MGMT_CLIENT_ID" \
      --rolename manage-users \
      --rolename view-users

SECRET=$(kcadm.sh get clients/$CID/client-secret -r "$REALM" | jq -r '.value')

# If asked, emit environment exports for the caller to source
if [[ "${1:-}" == "--emit-env" ]]; then
  printf 'export KC_REALM=%q\n' "$REALM"
  printf 'export KC_CLIENT_ID=%q\n' "$CLIENT_ID"
  printf 'export KC_CLIENT_SECRET=%q\n' "$SECRET"
else
  echo "Client '$CLIENT_ID' ready in realm '$REALM'."
  echo "Use: export KC_CLIENT_SECRET=$SECRET"
fi

# echo "✅ Service client '$CLIENT_ID' ready in realm '$REALM'; roles assigned from '$MGMT_CLIENT_ID' to $SA_USER"
# export CLIENT_SECRET=$(kcadm.sh get clients/$CID/client-secret -r "$REALM" | jq -r '.value')
