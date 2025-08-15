import { env } from "@/env"

export async function GET() {
  return Response.json({
    keycloakUrl: env.KEYCLOAK_URL,
    keycloakRealm: env.KEYCLOAK_REALM,
    keycloakClientId: env.KEYCLOAK_CLIENT_ID,
  })
}
