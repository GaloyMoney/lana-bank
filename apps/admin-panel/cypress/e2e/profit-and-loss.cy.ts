import { print } from "@apollo/client/utilities"

import {
  ProfitAndLossStatementDocument,
  ProfitAndLossStatementQuery,
} from "../../lib/graphql/generated"

import { t } from "../support/translation"

const PL = "ProfitAndLoss"
const CLS = "CurrencyLayerSelection"

describe("Profit and Loss Statement", () => {
  const currentDate = new Date()
  const lastMonthDate = new Date()
  lastMonthDate.setMonth(lastMonthDate.getMonth() - 1)

  beforeEach(() => {
    cy.visit("/profit-and-loss")
  })

  it("should render all categories and their children", () => {
    cy.graphqlRequest<{ data: ProfitAndLossStatementQuery }>(
      print(ProfitAndLossStatementDocument),
      {
        from: lastMonthDate.toISOString().split("T")[0],
        until: currentDate.toISOString().split("T")[0],
      },
    ).then((response) => {
      const rows = response.data.profitAndLossStatement?.rows ?? []
      const categories = rows.filter((row) => !row.parentProfitAndLossAccountId)
      categories.forEach((category) => {
        cy.get(`[data-testid="category-${category.name.toLowerCase()}"]`).should("exist")
        const children = rows.filter(
          (row) => row.parentProfitAndLossAccountId === category.profitAndLossAccountId,
        )
        children.forEach((child) => {
          cy.get(`[data-testid="account-${child.profitAndLossAccountId}"]`).should("exist")
        })
      })
    })
    cy.takeScreenshot("profit-and-loss")
  })

  it("should display basic page elements", () => {
    cy.contains(t(PL + ".title")).should("exist")
    cy.contains(t("DateRangePicker.dateRange")).should("exist")
    cy.contains(t(PL + ".net")).should("exist")
  })

  it("should allow currency switching", () => {
    cy.contains(t(CLS + ".currency.options.usd"))
      .should("be.visible")
      .click()
    cy.contains(t(CLS + ".currency.options.btc"))
      .should("be.visible")
      .click()
    cy.takeScreenshot("profit-and-loss-btc-currency")
  })

  it("should switch between balance layers", () => {
    cy.contains(t(CLS + ".layer.options.settled")).should("exist")
    cy.contains(t(CLS + ".layer.options.pending")).should("exist")

    cy.contains(t(CLS + ".layer.options.settled")).should("exist")
    cy.get('[role="tablist"]')
      .contains(t(CLS + ".layer.options.pending"))
      .click()
    cy.takeScreenshot("profit-and-loss-pending")
    cy.get('[role="tablist"]')
      .contains(t(CLS + ".layer.options.settled"))
      .click()
  })

  it("should default to collapsed rows and allow toggling", () => {
    cy.graphqlRequest<{ data: ProfitAndLossStatementQuery }>(
      print(ProfitAndLossStatementDocument),
      {
        from: lastMonthDate.toISOString().split("T")[0],
        until: currentDate.toISOString().split("T")[0],
      },
    ).then((response) => {
      const rows = response.data.profitAndLossStatement?.rows ?? []
      const expandableRow = rows.find((row) =>
        rows.some(
          (candidate) =>
            candidate.parentProfitAndLossAccountId === row.profitAndLossAccountId,
        ),
      )

      expect(expandableRow, "expected an expandable profit and loss row").to.exist
      if (!expandableRow) return

      const childRow = rows.find(
        (row) =>
          row.parentProfitAndLossAccountId === expandableRow.profitAndLossAccountId,
      )
      expect(childRow, "expected a child row for expandable row").to.exist
      if (!childRow) return

      cy.get(`[data-testid="account-${expandableRow.profitAndLossAccountId}"]`).within(() => {
        cy.get(`[data-testid="toggle-${expandableRow.profitAndLossAccountId}"]`)
          .should("have.attr", "aria-label", "Expand account")
          .click()
      })

      cy.get(`[data-testid="account-${childRow.profitAndLossAccountId}"]`).should("exist")

      cy.get(`[data-testid="account-${expandableRow.profitAndLossAccountId}"]`).within(() => {
        cy.get(`[data-testid="toggle-${expandableRow.profitAndLossAccountId}"]`)
          .should("have.attr", "aria-label", "Collapse account")
          .click()
      })

      cy.get(`[data-testid="account-${childRow.profitAndLossAccountId}"]`).should("not.exist")
    })
  })
})
