// eslint-disable-next-line import/no-extraneous-dependencies, import/no-unassigned-import
import "cypress-file-upload"

import { TermsTemplateCreateInput } from "@/lib/graphql/generated"

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
        input: { email, telegramId },
      })
      .then((response) => {
        const customerId = response.data.customerCreate.customer.customerId
        return cy
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
          accrualInterval: input.accrualInterval,
          incurrenceInterval: input.incurrenceInterval,
          duration: {
            period: input.duration.period,
            units: input.duration.units,
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

export {}
