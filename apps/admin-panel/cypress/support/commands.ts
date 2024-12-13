import { TermsTemplateCreateInput } from "@/lib/graphql/generated"

declare global {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace Cypress {
    interface Chainable {
      takeScreenshot(filename: string): Chainable<null>
      createCustomer(email: string, telegramId: string): Chainable<string>
      createTermsTemplate(input: TermsTemplateCreateInput): Chainable<string>
      graphqlRequest<T>(query: string, variables?: Record<string, unknown>): Chainable<T>
      getIdFromUrl(pathSegment: string): Chainable<string>
      createDeposit(amount: number, customerId: string): Chainable<string>
      initiateWithdrawal(amount: number, customerId: string): Chainable<string>
    }
  }
}

Cypress.Commands.add(
  "graphqlRequest",
  <T>(query: string, variables?: Record<string, unknown>): Cypress.Chainable<T> => {
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
  cy.viewport(1263, 573)
  cy.screenshot(filename, { capture: "viewport" })
  return cy.wrap(null)
})

interface CustomerResponse {
  data: {
    customerCreate: {
      customer: {
        customerId: string
      }
    }
  }
}
Cypress.Commands.add(
  "createCustomer",
  (email: string, telegramId: string): Cypress.Chainable<string> => {
    const mutation = `
      mutation CustomerCreate($input: CustomerCreateInput!) {
        customerCreate(input: $input) {
          customer {
            customerId
          }
        }
      }
    `
    return cy
      .graphqlRequest<CustomerResponse>(mutation, {
        input: { email, telegramId },
      })
      .then((response) => response.data.customerCreate.customer.customerId)
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
  (amount: number, customerId: string): Cypress.Chainable<string> => {
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
        input: { amount, customerId },
      })
      .then((response) => response.data.depositRecord.deposit.depositId)
  },
)

Cypress.Commands.add(
  "initiateWithdrawal",
  (amount: number, customerId: string): Cypress.Chainable<string> => {
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
        input: { amount, customerId },
      })
      .then((response) => response.data.withdrawalInitiate.withdrawal.withdrawalId)
  },
)

export {}
