"use client"

import Keycloak from "keycloak-js"

import { env } from "@/env"

const PKCE_METHOD = "S256"

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
let initializationPromise: Promise<boolean> | null = null

export const initKeycloak = () => {
  if (!keycloak) {
    return Promise.resolve(false)
  }

  if (isInitialized) {
    return Promise.resolve(keycloak.authenticated ?? false)
  }

  if (initializationPromise) {
    return initializationPromise
  }

  initializationPromise = keycloak
    .init({ onLoad: "login-required", checkLoginIframe: false, pkceMethod: PKCE_METHOD })
    .then((authenticated) => {
      isInitialized = true
      return authenticated
    })
    .catch((err) => {
      initializationPromise = null
      console.error("Failed to initialize Keycloak", err)
      throw err
    })

  return initializationPromise
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
