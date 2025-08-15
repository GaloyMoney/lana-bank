"use client"

import Keycloak from "keycloak-js"

const PKCE_METHOD = "S256"

let keycloak: null | Keycloak = null
let configPromise: Promise<{
  keycloakUrl: string
  keycloakRealm: string
  keycloakClientId: string
}> | null = null

const fetchConfig = async () => {
  if (configPromise) return configPromise
  configPromise = fetch("/api/config")
    .then((res) => res.json())
    .catch((err) => {
      console.error("Failed to fetch Keycloak config", err)
    })
  return configPromise
}

const getKeycloak = async () => {
  if (!keycloak && typeof window !== "undefined") {
    const config = await fetchConfig()
    keycloak = new Keycloak({
      url: config.keycloakUrl,
      realm: config.keycloakRealm,
      clientId: config.keycloakClientId,
    })
  }
  return keycloak
}

let isInitialized = false
let initializationPromise: Promise<boolean> | null = null

export const initKeycloak = async () => {
  if (isInitialized) {
    const keycloakInstance = await getKeycloak()
    return keycloakInstance?.authenticated ?? false
  }

  if (initializationPromise) {
    return initializationPromise
  }

  initializationPromise = (async () => {
    try {
      const keycloakInstance = await getKeycloak()

      if (!keycloakInstance) {
        return false
      }

      const authenticated = await keycloakInstance.init({
        onLoad: "login-required",
        checkLoginIframe: false,
        pkceMethod: PKCE_METHOD,
      })

      isInitialized = true
      return authenticated
    } catch (err) {
      initializationPromise = null
      console.error("Failed to initialize Keycloak", err)
      throw err
    }
  })()

  return initializationPromise
}

export const logout = async () => {
  const keycloakInstance = await getKeycloak()
  if (keycloakInstance) {
    keycloakInstance.logout({
      redirectUri: `${window.location.origin}/`,
    })
  }
}

export const getToken = () => {
  if (keycloak && isInitialized) {
    return keycloak.token
  }
  return null
}

export { getKeycloak as keycloak }
