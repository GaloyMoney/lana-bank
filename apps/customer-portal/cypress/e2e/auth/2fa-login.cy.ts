import { authenticator } from "otplib"

import { addVirtualAuthenticator, generateRandomEmail } from "../../support/utils"

describe("Login with two factor", () => {
  it("successful login for TOTP", () => {
    const email = generateRandomEmail()
    cy.registerUser(email).then((sessionToken) => {
      cy.setupTotp(sessionToken).then((totpSecretKey) => {
        cy.startAuth(email)
        cy.submitOtp(email).then(() => {
          const totpCode = authenticator.generate(totpSecretKey)
          cy.get('[data-test-id="auth-totp-input"]').type(totpCode)
          cy.get('[data-test-id="auth-totp-submit-btn"]').click()
          cy.url().should("eq", Cypress.config().baseUrl + "/")
        })
      })
    })
  })

  it("fail login for incorrect TOTP", () => {
    const email = generateRandomEmail()
    cy.registerUser(email).then((sessionToken) => {
      cy.setupTotp(sessionToken).then(() => {
        cy.startAuth(email)
        cy.submitOtp(email).then(() => {
          cy.get('[data-test-id="auth-totp-input"]').type("000000")
          cy.get('[data-test-id="auth-totp-submit-btn"]').click()
          cy.get('[data-test-id="auth-totp-error"]').should(
            "have.text",
            "The provided authentication code is invalid, please try again.",
          )
        })
      })
    })
  })

  it("Setup and Login webauthn", () => {
    //TODO this test is flaky need improvements.
    const email = generateRandomEmail()
    const passkeyName = "test-passkey"

    cy.registerUser(email).then(() => {
      addVirtualAuthenticator()

      cy.startAuth(email)
      cy.submitOtp(email).then(() => {
        cy.get('[data-test-id="enable-2fa-button"]').click()
        cy.get('[data-test-id="setup-passkey-button"]').click()

        // Complete passkey setup
        cy.get('[data-test-id="passkey-name-input"]').should("be.visible").type(passkeyName)
        cy.get('[data-test-id="submit-passkey-name"]').click()

        cy.get("table").contains("td", passkeyName).should("be.visible")

        cy.clearAllCookies()
        cy.getCookies().should("be.empty")

        cy.startAuth(email)
        cy.submitOtp(email).then(() => {
          cy.url().should("eq", Cypress.config().baseUrl + "/")
        })
      })
    })
  })
})
