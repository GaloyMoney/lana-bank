import { t } from "../support/translation"

const COALA = "ChartOfAccountsLedgerAccount"

describe("Chart Of Accounts", () => {
  it("should upload CSV and display chart data", () => {
    cy.uploadChartOfAccounts()

    cy.get("body")
      .contains(/Assets/i)
      .should("be.visible")
    cy.get("body")
      .contains(/Liabilities/i)
      .should("exist")
    cy.get("body")
      .contains(/Equity/i)
      .should("exist")
    cy.get("body")
      .contains(/Revenue/i)
      .should("exist")
    cy.get("body")
      .contains(/Expenses/i)
      .should("exist")

    cy.get("body")
      .contains(/Current Assets/i)
      .should("be.visible")
    cy.get("body")
      .contains(/Non-Current Assets/i)
      .should("be.visible")

    cy.takeScreenshot("2_chart_of_account_view")
  })

  it("should open ledger account details", () => {
    cy.visit("/chart-of-accounts")
    cy.get("body")
      .contains(/Assets/i)
      .should("be.visible")
      .click()

    cy.contains(t(COALA + ".title")).should("be.visible")

    cy.takeScreenshot("3_ledger_account_details")
  })
})
