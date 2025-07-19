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

  it("should have generate button", () => {
    cy.contains(t(R + ".title"))
    cy.contains(t(R + ".description"))

    cy.takeScreenshot("1_generate_report_button")
  })
})
