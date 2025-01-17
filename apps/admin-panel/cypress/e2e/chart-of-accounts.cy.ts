import { print } from "@apollo/client/utilities"

import {
  ChartOfAccountsDocument,
  ChartOfAccountsQuery,
  ChartControlAccount,
  ChartControlSubAccount,
} from "../../lib/graphql/generated"

describe("Chart Of Accounts", () => {
  beforeEach(() => {
    cy.visit("/chart-of-accounts")
    cy.get('[data-testid="loading-skeleton"]').should("not.exist")
  })

  it("should render all categories and their accounts", () => {
    cy.graphqlRequest<{ data: ChartOfAccountsQuery }>(
      print(ChartOfAccountsDocument),
    ).then((response) => {
      const categories = response.data.chartOfAccounts?.categories
      if (!categories) return

      const categoryNames = [
        "assets",
        "liabilities",
        "equity",
        "revenues",
        "expenses",
      ] as const

      categoryNames.forEach((name) => {
        const category = categories[name]
        if (!category) return

        cy.get(`[data-testid="category-${name}"]`)
          .should("be.visible")
          .parent("tr")
          .within(() => {
            cy.contains(category.name).should("be.visible")
            cy.contains(category.accountCode).should("be.visible")
          })

        category.controlAccounts.forEach((control: ChartControlAccount) => {
          cy.contains("td", control.name)
            .should("be.visible")
            .parent("tr")
            .within(() => {
              cy.contains(control.accountCode).should("be.visible")
            })

          if (control.controlSubAccounts.length > 0) {
            cy.contains("td", control.name).click()
            control.controlSubAccounts.forEach((sub: ChartControlSubAccount) => {
              cy.contains("td", sub.name)
                .should("be.visible")
                .parent("tr")
                .within(() => {
                  cy.contains(sub.accountCode).should("be.visible")
                })
            })
          }
        })
      })
    })
    cy.takeScreenshot("chart-of-accounts")
  })

  it("should show Off Balance Sheet", () => {
    cy.contains("button", "Off Balance Sheet").click()
    cy.get('[data-testid="loading-skeleton"]').should("not.exist")
    cy.takeScreenshot("off-balance-sheet")
  })

  it("should toggle control accounts correctly", () => {
    cy.graphqlRequest<{ data: ChartOfAccountsQuery }>(
      print(ChartOfAccountsDocument),
    ).then((response) => {
      const assets = response.data.chartOfAccounts?.categories.assets
      if (!assets) return

      const controlAccount = assets.controlAccounts.find(
        (ca: ChartControlAccount) => ca.controlSubAccounts.length > 0,
      )
      if (!controlAccount) return

      const subAccountName = controlAccount.controlSubAccounts[0].name
      cy.contains("td", subAccountName).should("not.exist")
      cy.contains("td", controlAccount.name).click()
      cy.contains("td", subAccountName).should("be.visible")
      cy.contains("td", controlAccount.name).click()
      cy.contains("td", subAccountName).should("not.exist")
    })
  })
})
