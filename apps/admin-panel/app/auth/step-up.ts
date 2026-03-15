import { fetchConfig, getKeycloak } from "./keycloak"

export async function authenticateWithPassword(password: string): Promise<string> {
  const config = await fetchConfig()
  const keycloakInstance = await getKeycloak()
  const username = keycloakInstance?.tokenParsed?.preferred_username

  if (!config || !username) {
    throw new Error("Authentication context not available")
  }

  const response = await fetch(
    `${config.keycloakUrl}/realms/${config.keycloakRealm}/protocol/openid-connect/token`,
    {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        grant_type: "password",
        client_id: config.keycloakClientId,
        username,
        password,
      }),
    },
  )

  if (!response.ok) {
    throw new Error("Invalid password")
  }

  const data = await response.json()
  return data.access_token
}
