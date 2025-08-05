// ***********************************************************
// This example support/e2e.ts is processed and
// loaded automatically before your test files.
//
// This is a great place to put global configuration and
// behavior that modifies Cypress.
//
// You can change the location of this file or turn off
// automatically serving support files with the
// 'supportFile' configuration option.
//
// You can read more here:
// https://on.cypress.io/configuration
// ***********************************************************

// Import commands.js using ES2015 syntax:
// eslint-disable-next-line import/no-unassigned-import
import "./commands"
import { t } from "./translation"

Cypress.on("window:before:load", (win) => {
  const style = win.document.createElement("style")
  style.innerHTML = `
    nextjs-portal,
    [data-nextjs-toast-wrapper] {
      display: none !important;
    }
  `
  win.document.head.appendChild(style)
})

const testLanguage = Cypress.env("TEST_LANGUAGE")
beforeEach(() => {
  cy.session(
    "loginSession",
    () => {
      cy.KcLogin("admin@galoy.io")
      cy.setCookie("NEXT_LOCALE", testLanguage)
      cy.visit("/dashboard")
      cy.contains(t("Sidebar.navItems.dashboard"), {
        timeout: 60000,
      })
    },
    {
      cacheAcrossSpecs: true,
      validate: () => {
        cy.getCookie("KEYCLOAK_SESSION").should("exist")
        cy.getCookie("KEYCLOAK_IDENTITY").should("exist")
      },
    },
  )
})
