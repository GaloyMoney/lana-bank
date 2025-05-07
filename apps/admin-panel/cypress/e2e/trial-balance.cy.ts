import { print } from "@apollo/client/utilities"

import {
  GetTrialBalanceDocument,
  GetTrialBalanceQuery,
} from "../../lib/graphql/generated"

import { t } from "../support/translation"

const TB = "TrialBalance"
const CLS = "CurrencyLayerSelection"

describe(t(TB + ".title"), () => {
  const currentDate = new Date()
  const lastMonthDate = new Date()
  lastMonthDate.setMonth(lastMonthDate.getMonth() - 1)

  beforeEach(() => {
    cy.visit("/trial-balance")
  })

  it("should render trial balance with accounts and their balances", () => {
    cy.graphqlRequest<{ data: GetTrialBalanceQuery }>(print(GetTrialBalanceDocument), {
      from: lastMonthDate.toISOString().split('T')[0],
      until: currentDate.toISOString().split('T')[0],
      first: 10,
    }).then((response) => {
      response.data.trialBalance?.accounts.edges.forEach(({ node: account }) => {
        cy.get("main")
          .contains(new RegExp(`^${account.name}$`))
          .should("exist")
        cy.get("main")
          .contains(new RegExp(`^${account.name}$`))
          .parent("tr")
          .within(() => {
            cy.get("td").should("have.length", 5)
          })
      })
    })
    cy.takeScreenshot("trial-balance")
  })

  it("should switch between currency types", () => {
    cy.contains(t(CLS + ".currency.options.usd")).should("exist")
    cy.contains(t(CLS + ".currency.options.btc")).should("exist")

    cy.contains(t(CLS + ".currency.options.usd")).click()
    cy.contains(t(CLS + ".currency.options.btc")).click()
    cy.takeScreenshot("trial-balance-btc-currency")
  })

  it("should switch between balance layers", () => {
    cy.contains(t(CLS + ".layer.options.settled")).should("exist")
    cy.contains(t(CLS + ".layer.options.pending")).should("exist")

    cy.contains(t(CLS + ".layer.options.settled")).click()
    cy.contains(t(CLS + ".layer.options.pending")).click()
  })

  it("should show date range selector", () => {
    cy.contains(t(TB + ".dateRange") + ":").should("exist")
  })
})
