"use client"

import Keycloak from "keycloak-js"

import { env } from "@/env"

const keycloakConfig = {
  url: env.NEXT_PUBLIC_KEYCLOAK_URL,
  realm: env.NEXT_PUBLIC_KEYCLOAK_REALM,
  clientId: env.NEXT_PUBLIC_KEYCLOAK_CLIENT_ID,
}

let keycloak: null | Keycloak = null

if (typeof window !== "undefined") {
  keycloak = new Keycloak(keycloakConfig)
}

let isInitialized = false
export const initKeycloak = () => {
  if (!isInitialized && keycloak) {
    isInitialized = true
    return keycloak
      .init({ onLoad: "login-required", checkLoginIframe: false, pkceMethod: "S256" })
      .then((authenticated) => authenticated)
      .catch((err) => {
        isInitialized = false
        console.error("Failed to initialize Keycloak", err)
        throw err
      })
  }
  return Promise.resolve(keycloak?.authenticated ?? false)
}

export const logout = () => {
  if (keycloak) {
    keycloak.logout({
      redirectUri: `${window.location.origin}/`,
    })
  }
}

export const getToken = () => {
  if (keycloak) {
    return keycloak.token
  }
  return null
}

export { keycloak }
