import { authenticator } from "otplib"

declare global {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace Cypress {
    interface Chainable {
      getOTP(email: string): Chainable<string>
      registerUser(email: string): Chainable<string>
      loginUser(email: string): Chainable<string>
      setupTotp(sessionToken: string): Chainable<string>
      startAuth(email: string): Chainable<void>
      submitOtp(email: string): Chainable<void>
    }
  }
}

const KRATOS_PG_URL = "postgres://dbuser:secret@localhost:5434/default?sslmode=disable"

Cypress.Commands.add("getOTP", (email) => {
  const query = `psql "${KRATOS_PG_URL}" -t -c "SELECT body FROM courier_messages WHERE recipient='${email}' ORDER BY created_at DESC LIMIT 1;"`
  cy.log(`Executing query: ${query}`)
  return cy.exec(query).then((result) => {
    const rawMessage = result.stdout
    const otpMatch = rawMessage.match(/(\d{6})/)
    if (otpMatch && otpMatch[1]) {
      return cy.wrap(otpMatch[1])
    } else {
      throw new Error("OTP not found in the message")
    }
  })
})

Cypress.Commands.add("registerUser", (email) => {
  cy.request("GET", "/self-service/registration/api").then((response) => {
    const flowId = response.body.id
    const registrationUrl = `http://localhost:4433/self-service/registration?flow=${flowId}`
    cy.request({
      method: "POST",
      url: registrationUrl,
      body: {
        method: "code",
        traits: {
          email: email,
        },
      },
      failOnStatusCode: false,
    }).then(() => {
      const query = `psql "${KRATOS_PG_URL}" -t -c "SELECT body FROM courier_messages WHERE recipient='${email}' ORDER BY created_at DESC LIMIT 1;"`
      cy.exec(query).then((result) => {
        const rawMessage = result.stdout
        const otpMatch = rawMessage.match(/(\d{6})/)
        if (otpMatch && otpMatch[1]) {
          const otpCode = otpMatch[1]
          cy.request({
            method: "POST",
            url: registrationUrl,
            body: {
              method: "code",
              code: otpCode,
              traits: {
                email: email,
              },
            },
          }).then((finalResponse) => {
            const sessionToken: string = finalResponse.body.session_token
            return cy.wrap(sessionToken)
          })
        } else {
          throw new Error("OTP not found in the message")
        }
      })
    })
  })
})

Cypress.Commands.add("loginUser", (email) => {
  cy.request("GET", "/self-service/login/api").then((response) => {
    const flowId = response.body.id
    const loginUrl = `http://localhost:4433/self-service/login?flow=${flowId}`
    cy.request({
      method: "POST",
      url: loginUrl,
      body: {
        method: "code",
        identifier: email,
      },
      failOnStatusCode: false,
    }).then(() => {
      const query = `psql "${KRATOS_PG_URL}" -t -c "SELECT body FROM courier_messages WHERE recipient='${email}' ORDER BY created_at DESC LIMIT 1;"`
      cy.exec(query).then((result) => {
        const rawMessage = result.stdout
        const otpMatch = rawMessage.match(/(\d{6})/)

        if (otpMatch && otpMatch[1]) {
          const otpCode = otpMatch[1]

          cy.request({
            method: "POST",
            url: loginUrl,
            body: {
              method: "code",
              code: otpCode,
              identifier: email,
            },
          }).then((finalResponse) => {
            const sessionToken: string = finalResponse.body.session_token
            return cy.wrap(sessionToken)
          })
        } else {
          throw new Error("OTP not found in the message")
        }
      })
    })
  })
})

Cypress.Commands.add("setupTotp", (sessionToken) => {
  cy.request({
    method: "GET",
    url: "/self-service/settings/api",
    headers: {
      "X-Session-Token": sessionToken,
    },
  }).then((response) => {
    const flowId = response.body.id
    const totpSecretNode = response.body.ui.nodes.find(
      (node: {
        group: string
        attributes: {
          id: string
          text: {
            context: {
              secret: string
            }
          }
        }
      }) => node.group === "totp" && node.attributes.id === "totp_secret_key",
    )
    const totpSecretKey = totpSecretNode
      ? totpSecretNode.attributes.text.context.secret
      : null
    if (!totpSecretKey) {
      throw new Error("TOTP secret key not found")
    }
    const totpCode = authenticator.generate(totpSecretKey)
    const settingsUrl = `http://localhost:4433/self-service/settings?flow=${flowId}`
    cy.request({
      method: "POST",
      url: settingsUrl,
      body: {
        method: "totp",
        totp_code: totpCode,
      },
      headers: {
        "X-Session-Token": sessionToken,
      },
    }).then((finalResponse) => {
      if (finalResponse.status !== 200) {
        throw new Error("Failed to complete TOTP setup")
      }
      cy.log("TOTP setup complete")
      return cy.wrap(totpSecretKey)
    })
  })
})

Cypress.Commands.add("startAuth", (email) => {
  cy.visit("/auth")
  cy.get('[data-test-id="auth-email-input"]').type(email)
  return cy.get('[data-test-id="auth-email-submit-btn"]').click()
})

Cypress.Commands.add("submitOtp", (email) => {
  return cy.getOTP(email).then((otp) => {
    cy.get('[data-test-id="auth-otp-input"]').type(otp)
  })
})
export {}
