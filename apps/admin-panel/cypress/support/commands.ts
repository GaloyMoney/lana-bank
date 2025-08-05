// eslint-disable-next-line import/no-extraneous-dependencies, import/no-unassigned-import
import "cypress-file-upload"

import { t } from "../support/translation"
import { CustomerType, TermsTemplateCreateInput } from "../../lib/graphql/generated"

type Customer = {
  customerId: string
  publicId: string
  depositAccount: {
    id: string
    depositAccountId: string
  }
}

declare global {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace Cypress {
    interface Chainable {
      takeScreenshot(filename: string): Chainable<null>
      createCustomer(email: string, telegramId: string): Chainable<Customer>
      createTermsTemplate(input: TermsTemplateCreateInput): Chainable<string>
      graphqlRequest<T>(query: string, variables?: Record<string, unknown>): Chainable<T>
      getIdFromUrl(pathSegment: string): Chainable<string>
      createDeposit(amount: number, depositAccountId: string): Chainable<string>
      initiateWithdrawal(amount: number, depositAccountId: string): Chainable<string>
      uploadChartOfAccounts(): Chainable<void>
      waitForKeycloak(): Chainable<void>
      KcLogin(email: string): Chainable<void>
    }
  }
}

Cypress.Commands.add(
  "graphqlRequest",
  <T>(query: string, variables?: Record<string, unknown>): Cypress.Chainable<T> => {
    const root = "http://localhost:8081"
    const realm = "lana-admin"
    const userEmail = "admin@galoy.io"
    const userPassword = "admin"

    return cy
      .request({
        method: "POST",
        url: `${root}/realms/${realm}/protocol/openid-connect/token`,
        form: true,
        body: {
          client_id: "lana-admin-panel",
          grant_type: "password",
          username: userEmail,
          password: userPassword,
        },
      })
      .then(({ body: tokenBody }) => {
        return cy
          .request({
            method: "POST",
            url: "http://admin.localhost:4455/graphql",
            body: {
              query,
              variables,
            },
            headers: {
              "Content-Type": "application/json",
              "Authorization": `Bearer ${tokenBody.access_token}`,
            },
          })
          .then((response) => {
            if (response.body.errors) {
              throw new Error(
                `GraphQL Error: ${JSON.stringify(response.body.errors)} variables: ${JSON.stringify(variables)}`,
              )
            }
            return response.body
          })
      })
  },
)

Cypress.Commands.add("takeScreenshot", (filename): Cypress.Chainable<null> => {
  cy.get('[data-testid="loading-skeleton"]', { timeout: 30000 }).should("not.exist")
  cy.get('[data-testid="global-loader"]', { timeout: 30000 }).should("not.exist")
  cy.screenshot(filename, { capture: "viewport", overwrite: true })
  return cy.wrap(null)
})

interface CustomerCreateResponse {
  data: {
    customerCreate: {
      customer: Customer
    }
  }
}
interface CustomerQueryResponse {
  data: {
    customer: Customer
  }
}

Cypress.Commands.add(
  "createCustomer",
  (email: string, telegramId: string): Cypress.Chainable<Customer> => {
    const mutation = `
      mutation CustomerCreate($input: CustomerCreateInput!) {
        customerCreate(input: $input) {
          customer {
            customerId
            publicId
            depositAccount {
              id
              depositAccountId
            }
          }
        }
      }
    `
    const query = `
      query Customer($id: UUID!) {
        customer(id: $id) {
          customerId
          publicId
          applicantId
          level
          status
          email
          depositAccount {
            depositAccountId
          }
        }
      }
    `
    return cy
      .graphqlRequest<CustomerCreateResponse>(mutation, {
        input: { email, telegramId, customerType: CustomerType.Individual },
      })
      .then((response) => {
        const customerId = response.data.customerCreate.customer.customerId
        return cy
          .wait(1000) // to make sure deposit account is created
          .graphqlRequest<CustomerQueryResponse>(query, {
            id: customerId,
          })
          .then((resp) => resp.data.customer)
      })
  },
)

interface TermsTemplateResponse {
  data: {
    termsTemplateCreate: {
      termsTemplate: {
        termsId: string
      }
    }
  }
}
Cypress.Commands.add(
  "createTermsTemplate",
  (input: TermsTemplateCreateInput): Cypress.Chainable<string> => {
    const mutation = `
      mutation CreateTermsTemplate($input: TermsTemplateCreateInput!) {
        termsTemplateCreate(input: $input) {
          termsTemplate {
            termsId
          }
        }
      }
    `
    return cy
      .graphqlRequest<TermsTemplateResponse>(mutation, {
        input: {
          name: input.name,
          annualRate: input.annualRate,
          accrualCycleInterval: input.accrualCycleInterval,
          accrualInterval: input.accrualInterval,
          duration: {
            period: input.duration.period,
            units: input.duration.units,
          },
          interestDueDurationFromAccrual: {
            period: input.interestDueDurationFromAccrual.period,
            units: input.interestDueDurationFromAccrual.units,
          },
          obligationOverdueDurationFromDue: {
            period: input.obligationOverdueDurationFromDue.period,
            units: input.obligationOverdueDurationFromDue.units,
          },
          obligationLiquidationDurationFromDue: {
            period: input.obligationLiquidationDurationFromDue.period,
            units: input.obligationLiquidationDurationFromDue.units,
          },
          liquidationCvl: input.liquidationCvl,
          marginCallCvl: input.marginCallCvl,
          initialCvl: input.initialCvl,
          oneTimeFeeRate: input.oneTimeFeeRate,
        },
      })
      .then((response) => response.data.termsTemplateCreate.termsTemplate.termsId)
  },
)

Cypress.Commands.add("getIdFromUrl", (pathSegment: string) => {
  return cy.url().then((url) => {
    const id = url.split(pathSegment)[1]
    return id
  })
})

interface DepositResponse {
  data: {
    depositRecord: {
      deposit: {
        depositId: string
      }
    }
  }
}

interface WithdrawalInitiateResponse {
  data: {
    withdrawalInitiate: {
      withdrawal: {
        withdrawalId: string
      }
    }
  }
}

Cypress.Commands.add(
  "createDeposit",
  (amount: number, depositAccountId: string): Cypress.Chainable<string> => {
    const mutation = `
      mutation CreateDeposit($input: DepositRecordInput!) {
        depositRecord(input: $input) {
          deposit {
            depositId
          }
        }
      }
    `
    return cy
      .graphqlRequest<DepositResponse>(mutation, {
        input: { amount, depositAccountId },
      })
      .then((response) => response.data.depositRecord.deposit.depositId)
  },
)

Cypress.Commands.add(
  "initiateWithdrawal",
  (amount: number, depositAccountId: string): Cypress.Chainable<string> => {
    const mutation = `
      mutation WithdrawalInitiate($input: WithdrawalInitiateInput!) {
        withdrawalInitiate(input: $input) {
          withdrawal {
            withdrawalId
          }
        }
      }
    `
    return cy
      .graphqlRequest<WithdrawalInitiateResponse>(mutation, {
        input: { amount, depositAccountId },
      })
      .then((response) => response.data.withdrawalInitiate.withdrawal.withdrawalId)
  },
)

Cypress.Commands.add("uploadChartOfAccounts", () => {
  const COA = "ChartOfAccounts"

  cy.visit("/chart-of-accounts")
  cy.get('[data-testid="loading-skeleton"]').should("not.exist")

  cy.wait(5000)
    .window()
    .then((win) => {
      const table = win.document.querySelector("table")

      if (table) {
        cy.log("Chart of accounts already uploaded, skipping upload.")
        return
      }

      cy.get("body").then(async ($body) => {
        const hasUploadButton =
          $body.find(`button:contains("${t(COA + ".upload.upload")}")`).length > 0
        const hasDropzoneText =
          $body.find(`:contains("${t(COA + ".upload.dragAndDrop")}")`).length > 0

        cy.takeScreenshot("1_chart_of_account_upload")
        if (hasUploadButton || hasDropzoneText) {
          cy.get('input[type="file"]').attachFile("coa.csv", { force: true })
          cy.contains("button", new RegExp(t(COA + ".upload.upload"), "i"), {
            timeout: 5000,
          }).click()
        }
      })
    })

  cy.get("body")
    .contains(/Assets/i)
    .should("be.visible")
})

Cypress.Commands.add("waitForKeycloak", () => {
  const root = "http://localhost:8081"
  const maxAttempts = 30

  const checkKeycloak = (attempt: number) => {
    if (attempt > maxAttempts) {
      throw new Error(`Keycloak not ready after ${maxAttempts} attempts`)
    }
    cy.log(`Checking Keycloak readiness (attempt ${attempt}/${maxAttempts})`)
    cy.task("checkUrl", `${root}/realms/master`).then((masterReady: any) => {
      if (masterReady) {
        cy.task("checkUrl", `${root}/realms/lana-admin`).then((adminReady: any) => {
          if (adminReady) {
            cy.request({
              method: "POST",
              url: `${root}/realms/master/protocol/openid-connect/token`,
              form: true,
              body: {
                client_id: "admin-cli",
                username: "admin",
                password: "admin",
                grant_type: "password",
              },
              failOnStatusCode: false,
            }).then((tokenResponse) => {
              if (tokenResponse.status === 200) {
                const adminToken = tokenResponse.body.access_token
                cy.request({
                  method: "GET",
                  url: `${root}/admin/realms/lana-admin/users`,
                  qs: { email: "admin@galoy.io", exact: true },
                  headers: { Authorization: `Bearer ${adminToken}` },
                  failOnStatusCode: false,
                }).then((userResponse) => {
                  if (userResponse.status === 200 && userResponse.body.length > 0) {
                    cy.log("Keycloak and admin user are ready")
                  } else {
                    cy.log(`Admin user not found, retrying...`)
                    cy.wait(2000).then(() => checkKeycloak(attempt + 1))
                  }
                })
              } else {
                cy.log(`Cannot get admin token, retrying...`)
                cy.wait(2000).then(() => checkKeycloak(attempt + 1))
              }
            })
          } else {
            cy.log(`Lana-admin realm not ready, retrying...`)
            cy.wait(2000).then(() => checkKeycloak(attempt + 1))
          }
        })
      } else {
        cy.log(`Master realm not ready, retrying...`)
        cy.wait(2000).then(() => checkKeycloak(attempt + 1))
      }
    })
  }
  checkKeycloak(1)
})

Cypress.Commands.add("KcLogin", (email: string) => {
  const root = "http://localhost:8081"
  const realm = "lana-admin"
  const adminU = "admin"
  const adminP = "admin"

  cy.request({
    method: "POST",
    url: `${root}/realms/master/protocol/openid-connect/token`,
    form: true,
    body: {
      client_id: "admin-cli",
      username: adminU,
      password: adminP,
      grant_type: "password",
    },
  })
    .then(({ body }) => {
      const adminToken = body.access_token
      return cy
        .request({
          method: "GET",
          url: `${root}/admin/realms/${realm}/users`,
          qs: { search: email },
          headers: { Authorization: `Bearer ${adminToken}` },
        })
        .then(({ body: users }) => {
          if (!users.length) {
            throw new Error(`No Keycloak user found for ${email}`)
          }
          const user = users[0]
          const userId = user.id
          return cy
            .request({
              method: "PUT",
              url: `${root}/admin/realms/${realm}/users/${userId}/reset-password`,
              headers: { Authorization: `Bearer ${adminToken}` },
              body: {
                type: "password",
                value: "admin",
                temporary: false,
              },
            })
            .then(() => {
              return cy.request({
                method: "POST",
                url: `${root}/admin/realms/${realm}/users/${userId}/impersonation`,
                headers: { Authorization: `Bearer ${adminToken}` },
                followRedirect: false,
              })
            })
        })
    })
    .then(({ headers }) => {
      if (headers["set-cookie"]) {
        ;(headers["set-cookie"] as string[]).forEach((cookieString) => {
          const [nameValue, ...attributes] = cookieString.split(";")
          const [name, value] = nameValue.split("=")
          if (
            name &&
            value &&
            (name === "KEYCLOAK_SESSION" || name === "KEYCLOAK_IDENTITY")
          ) {
            const cookieOptions: Record<string, unknown> = {
              domain: "localhost",
              path: "/realms/lana-admin/",
            }
            attributes.forEach((attr) => {
              const [key, val] = attr.trim().split("=")
              if (key.toLowerCase() === "httponly") cookieOptions.httpOnly = true
              if (key.toLowerCase() === "secure") cookieOptions.secure = true
              if (key.toLowerCase() === "samesite") cookieOptions.sameSite = val
            })
            cy.setCookie(name, value, cookieOptions)
          }
        })
      }
    })
    .then(() => {
      cy.visit("/")
    })
})

export {}
