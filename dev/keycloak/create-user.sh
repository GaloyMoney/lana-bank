USER_ID=$(
  curl -is -X POST "$KC_BASE/admin/realms/$KC_REALM/users" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"username":"alice","email":"alice+'$(uuidgen | tr '[:upper:]' '[:lower:]')'@example.com","emailVerified":true,"enabled":true}' \
  | awk -F/ '/^Location: /{print $NF}' | tr -d "\r"
)
echo "New user id: $USER_ID"

curl -s -X PUT "$KC_BASE/admin/realms/$KC_REALM/users/$USER_ID/reset-password" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"type":"password","value":"password","temporary":false}'
echo "Password set for user $USER_ID"