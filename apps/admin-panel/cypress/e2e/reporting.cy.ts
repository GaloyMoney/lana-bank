import { t } from "../support/translation"

const R = "Reports"

describe("Regulatory Report Management", () => {
  beforeEach(() => {
    cy.on("uncaught:exception", (err) => {
      if (err.message.includes("ResizeObserver loop")) {
        return false
      }
    })
    cy.visit("/regulatory-reporting")
  })

  it("should show available reports and recent runs", () => {
    cy.contains(t(R + ".title"))
    cy.contains(t(R + ".description"))
    cy.contains(t(R + ".availableReports"))
    cy.contains(t(R + ".recentRuns"))

    cy.takeScreenshot("1_report_management")
  })
})
