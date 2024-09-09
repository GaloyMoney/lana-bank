import { authenticator } from "otplib"

import { addVirtualAuthenticator, generateRandomEmail } from "../../support/utils"

describe("Login with two factor", () => {
  it("successful login for TOTP", () => {
    const email = generateRandomEmail()
    const telegramId = generateRandomEmail()
    cy.registerUser(email, telegramId).then((sessionToken) => {
      cy.setupTotp(sessionToken).then((totpSecretKey) => {
        cy.visit("/auth")
        cy.get('[data-test-id="auth-email-input"]').type(email)
        cy.get('[data-test-id="auth-email-submit-btn"]').click()

        cy.getOTP(email).then((otp) => {
          const totpCode = authenticator.generate(totpSecretKey)
          cy.get('[data-test-id="auth-otp-input"]').type(otp)
          cy.get('[data-test-id="auth-totp-input"]').type(totpCode)
          cy.get('[data-test-id="auth-totp-submit-btn"]').click()
          cy.url().should("eq", Cypress.config().baseUrl + "/")
        })
      })
    })
  })

  it("fail login for incorrect TOTP", () => {
    const email = generateRandomEmail()
    const telegramId = generateRandomEmail()
    cy.registerUser(email, telegramId).then((sessionToken) => {
      cy.setupTotp(sessionToken).then(() => {
        cy.visit("/auth")
        cy.get('[data-test-id="auth-email-input"]').type(email)
        cy.get('[data-test-id="auth-email-submit-btn"]').click()

        cy.getOTP(email).then((otp) => {
          cy.get('[data-test-id="auth-otp-input"]').type(otp)
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
    const telegramId = generateRandomEmail()
    const passkeyName = "test-passkey"

    cy.registerUser(email, telegramId).then(() => {
      addVirtualAuthenticator()

      cy.visit("/auth")
      cy.get('[data-test-id="auth-email-input"]').type(email)
      cy.get('[data-test-id="auth-email-submit-btn"]').click()

      cy.getOTP(email).then((otp) => {
        cy.get('[data-test-id="auth-otp-input"]').type(otp)
        cy.get('[data-test-id="enable-2fa-button"]').click()
        cy.get('[data-test-id="setup-passkey-button"]').click()

        // Complete passkey setup
        cy.get('[data-test-id="passkey-name-input"]').type(passkeyName)
        cy.get('[data-test-id="submit-passkey-name"]').click()

        cy.get("table").contains("td", passkeyName).should("exist")

        cy.clearAllCookies()
        cy.visit("/auth")
        cy.get('[data-test-id="auth-email-input"]').type(email)
        cy.get('[data-test-id="auth-email-submit-btn"]').click()

        cy.getOTP(email).then((otp) => {
          cy.get('[data-test-id="auth-otp-input"]').type(otp)
        })
      })
    })
  })
})
