import { print } from "@apollo/client/utilities"

import { t } from "../support/translation"

import { BalanceSheetDocument, BalanceSheetQuery } from "../../lib/graphql/generated"

const BalanceSheet = "BalanceSheet"
const CLS = "CurrencyLayerSelection"

describe("Balance Sheet", () => {
  const currentDate = new Date()

  beforeEach(() => {
    cy.visit("/balance-sheet")
  })

  it("should display page title", () => {
    cy.contains(t(BalanceSheet + ".title")).should("exist")
  })

  it("should display balance sheet sections and categories", () => {
    cy.graphqlRequest<{ data: BalanceSheetQuery }>(print(BalanceSheetDocument), {
      asOf: currentDate.toISOString().split("T")[0],
    }).then((response) => {
      cy.contains(t(BalanceSheet + ".columns.assets")).should("be.visible")
      cy.contains(t(BalanceSheet + ".columns.liabilitiesAndEquity")).should("be.visible")

      cy.get("[data-testid^='category-name-']").then(($cells) => {
        const categoryTexts = $cells.map((_, el) => Cypress.$(el).text().trim()).get()
        expect(categoryTexts).to.include(t(BalanceSheet + ".categories.Assets"))
        expect(categoryTexts).to.include(t(BalanceSheet + ".categories.Liabilities"))
        expect(categoryTexts).to.include(t(BalanceSheet + ".categories.Equity"))
      })

      const rows = response.data?.balanceSheet?.rows ?? []
      const rootRows = rows.filter((row) => !row.parentBalanceSheetAccountId)
      rootRows.forEach((category) => {
        const children = rows.filter(
          (row) => row.parentBalanceSheetAccountId === category.balanceSheetAccountId,
        )
        children.forEach((child) => {
          if (child.name) {
            cy.contains(child.name).should("be.visible")
          }
        })
      })
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

  it("should default to collapsed rows and allow toggling", () => {
    cy.graphqlRequest<{ data: BalanceSheetQuery }>(print(BalanceSheetDocument), {
      asOf: currentDate.toISOString().split("T")[0],
    }).then((response) => {
      const rows = response.data?.balanceSheet?.rows ?? []
      const expandableRow = rows.find((row) =>
        rows.some(
          (candidate) =>
            candidate.parentBalanceSheetAccountId === row.balanceSheetAccountId,
        ),
      )

      expect(expandableRow, "expected an expandable balance sheet row").to.exist
      if (!expandableRow) return

      const childRow = rows.find(
        (row) =>
          row.parentBalanceSheetAccountId === expandableRow.balanceSheetAccountId,
      )
      expect(childRow, "expected a child row for expandable row").to.exist
      if (!childRow) return

      cy.get(`[data-testid="account-${expandableRow.balanceSheetAccountId}"]`).within(() => {
        cy.get(`[data-testid="toggle-${expandableRow.balanceSheetAccountId}"]`)
          .should("have.attr", "aria-label", "Expand account")
          .click()
      })

      cy.get(`[data-testid="account-${childRow.balanceSheetAccountId}"]`).should("exist")

      cy.get(`[data-testid="account-${expandableRow.balanceSheetAccountId}"]`).within(() => {
        cy.get(`[data-testid="toggle-${expandableRow.balanceSheetAccountId}"]`)
          .should("have.attr", "aria-label", "Collapse account")
          .click()
      })

      cy.get(`[data-testid="account-${childRow.balanceSheetAccountId}"]`).should("not.exist")
    })
  })
})
