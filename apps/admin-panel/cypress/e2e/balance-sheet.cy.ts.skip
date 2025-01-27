import { print } from "@apollo/client/utilities"

import { BalanceSheetDocument, BalanceSheetQuery } from "../../lib/graphql/generated"

describe("Balance Sheet", () => {
  const currentDate = new Date()
  const lastMonthDate = new Date()
  lastMonthDate.setMonth(lastMonthDate.getMonth() - 1)

  beforeEach(() => {
    cy.visit("/balance-sheet")
  })

  it("should display page title", () => {
    cy.contains("Balance Sheet").should("exist")
  })

  it("should display balance sheet sections and categories", () => {
    cy.graphqlRequest<{ data: BalanceSheetQuery }>(print(BalanceSheetDocument), {
      from: lastMonthDate.toISOString(),
      until: currentDate.toISOString(),
    }).then((response) => {
      cy.contains("Total Assets").should("be.visible")
      cy.contains("Total Liabilities & Equity").should("be.visible")

      cy.get("[data-testid^='category-name-']").then(($cells) => {
        const categoryTexts = $cells.map((_, el) => Cypress.$(el).text().trim()).get()
        expect(categoryTexts).to.include("Assets")
        expect(categoryTexts).to.include("Liabilities")
        expect(categoryTexts).to.include("Equity")
      })

      if (response.data?.balanceSheet?.categories) {
        response.data.balanceSheet.categories.forEach((category) => {
          if (category?.accounts) {
            category.accounts.forEach((account) => {
              if (account?.name) {
                cy.contains(account.name).should("be.visible")
              }
            })
          }
        })
      }
    })
    cy.takeScreenshot("balance-sheet")
  })

  it("should allow currency switching", () => {
    cy.contains("USD").should("be.visible").click()
    cy.contains("BTC").should("be.visible").click()
    cy.takeScreenshot("balance-sheet-btc-currency")
  })

  it("should switch between balance layers", () => {
    cy.contains("All").should("exist")
    cy.contains("Settled").should("exist")
    cy.contains("Pending").should("exist")

    cy.contains("All").click()
    cy.contains("Settled").click()
    cy.contains("Pending").click()
    cy.takeScreenshot("balance-sheet-pending")
  })
})
