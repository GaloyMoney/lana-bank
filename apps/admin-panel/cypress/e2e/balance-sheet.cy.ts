import { print } from "@apollo/client/utilities"

import { t } from "../support/translation"

import { BalanceSheetDocument, BalanceSheetQuery } from "../../lib/graphql/generated"

const BalanceSheet = "BalanceSheet"
const CLS = "CurrencyLayerSelection"

describe("Balance Sheet", () => {
  const currentDate = new Date()
  const lastMonthDate = new Date()
  lastMonthDate.setMonth(lastMonthDate.getMonth() - 1)

  beforeEach(() => {
    cy.visit("/balance-sheet")
  })

  it("should display page title", () => {
    cy.contains(t(BalanceSheet + ".title")).should("exist")
  })

  it("should display balance sheet sections and categories", () => {
    cy.graphqlRequest<{ data: BalanceSheetQuery }>(print(BalanceSheetDocument), {
      from: lastMonthDate.toISOString().split('T')[0],
      until: currentDate.toISOString().split('T')[0],
    }).then((response) => {
      cy.contains(t(BalanceSheet + ".columns.assets")).should("be.visible")
      cy.contains(t(BalanceSheet + ".columns.liabilitiesAndEquity")).should("be.visible")

      cy.get("[data-testid^='category-name-']").then(($cells) => {
        const categoryTexts = $cells.map((_, el) => Cypress.$(el).text().trim()).get()
        expect(categoryTexts).to.include(t(BalanceSheet + ".categories.Assets"))
        expect(categoryTexts).to.include(t(BalanceSheet + ".categories.Liabilities"))
        expect(categoryTexts).to.include(t(BalanceSheet + ".categories.Equity"))
      })

      if (response.data?.balanceSheet?.categories) {
        response.data.balanceSheet.categories.forEach((category) => {
          if (category?.children) {
            category.children.forEach((child) => {
              if (child?.name) {
                cy.contains(child.name).should("be.visible")
              }
            })
          }
        })
      }
    })
    cy.takeScreenshot("balance-sheet")
  })

  it("should allow currency switching", () => {
    cy.contains(t(CLS + ".currency.options.usd"))
      .should("be.visible")
      .click()
    cy.contains(t(CLS + ".currency.options.btc"))
      .should("be.visible")
      .click()
    cy.takeScreenshot("balance-sheet-btc-currency")
  })

  it("should switch between balance layers", () => {
    cy.contains(t(CLS + ".layer.options.settled")).should("exist")
    cy.contains(t(CLS + ".layer.options.pending")).should("exist")

    cy.contains(t(CLS + ".layer.options.settled")).click()
    cy.contains(t(CLS + ".layer.options.pending")).click()
    cy.takeScreenshot("balance-sheet-pending")
  })
})
