"use client"

import Keycloak from "keycloak-js"

let keycloakInstance: Keycloak | null = null

export interface KeycloakConfig {
  url: string
  realm: string
  clientId: string
}

const getKeycloakConfig = (): KeycloakConfig => ({
  url: "http://localhost:8081",
  realm: "lana-admin",
  clientId: "lana-admin-panel",
})

const getKeycloakInstance = (): Keycloak => {
  if (!keycloakInstance) {
    keycloakInstance = new Keycloak(getKeycloakConfig())
  }
  return keycloakInstance
}

export const initKeycloak = async (): Promise<boolean> => {
  if (typeof window === "undefined") return false
  const keycloak = getKeycloakInstance()
  if (keycloak.didInitialize) {
    return !!keycloak.authenticated
  }

  try {
    const authenticated = await keycloak.init({
      onLoad: "check-sso",
      silentCheckSsoRedirectUri: window.location.origin + "/silent-check-sso.html",
      checkLoginIframe: false,
      pkceMethod: "S256",
    })

    return authenticated
  } catch (error) {
    console.error("Failed to initialize Keycloak", error)
    return false
  }
}

export const login = async (): Promise<void> => {
  const keycloak = getKeycloakInstance()
  await keycloak.login({
    redirectUri: "http://admin.localhost:4455/dashboard",
  })
}

export const logout = async (): Promise<void> => {
  const keycloak = getKeycloakInstance()
  await keycloak.logout({
    redirectUri: "http://admin.localhost:4455/",
  })
}

export const getToken = (): string | undefined => {
  const keycloak = getKeycloakInstance()
  return keycloak.authenticated ? keycloak.token : undefined
}

export const isAuthenticated = (): boolean => {
  const keycloak = getKeycloakInstance()
  return !!keycloak.authenticated && !!keycloak.token
}
