// eslint-disable-next-line import/no-extraneous-dependencies, import/no-unassigned-import
import "cypress-file-upload"

import { t } from "../support/translation"
import { CustomerType, TermsTemplateCreateInput } from "../../lib/graphql/generated"

type Customer = {
  customerId: string
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
    }
  }
}

Cypress.Commands.add(
  "graphqlRequest",
  <T>(query: string, variables?: Record<string, unknown>): Cypress.Chainable<T> => {
    const cookies = JSON.parse(
      Buffer.from(Cypress.env("COOKIES"), "base64").toString("utf-8"),
    )
    const cookieHeader = `${cookies["cookie1_name"]}=${cookies["cookie1_value"]}; ${cookies["cookie2_name"]}=${cookies["cookie2_value"]}`

    return cy
      .request({
        method: "POST",
        url: "http://localhost:4455/admin/graphql",
        body: {
          query,
          variables,
        },
        headers: {
          "Content-Type": "application/json",
          "Cookie": cookieHeader,
        },
      })
      .then((response) => {
        if (response.body.errors) {
          throw new Error(`GraphQL Error: ${JSON.stringify(response.body.errors)}`)
        }
        return response.body
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
          .then((response) => response.data.customer)
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

export {}
