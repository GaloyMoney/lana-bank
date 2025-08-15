import { env } from "@/env"

export async function GET() {
  return Response.json({
    keycloakUrl: env.NEXT_PUBLIC_KEYCLOAK_URL,
    keycloakRealm: env.NEXT_PUBLIC_KEYCLOAK_REALM,
    keycloakClientId: env.NEXT_PUBLIC_KEYCLOAK_CLIENT_ID,
  })
}
