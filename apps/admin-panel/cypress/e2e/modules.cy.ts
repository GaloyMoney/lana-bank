import { t } from "../support/translation"

const Modules = "Modules"

describe("Modules Configuration", () => {
  it("should configure deposit module", () => {
    cy.uploadChartOfAccounts()

    cy.visit("/modules")
    cy.takeScreenshot("1_modules_configuration")

    cy.contains(t(Modules + ".deposit.title")).should("be.visible")

    cy.takeScreenshot("2_deposit_configuration")
  })

  it("should configure deposit module", () => {
    cy.visit("/modules")
    cy.contains(t(Modules + ".credit.title")).should("be.visible")

    cy.takeScreenshot("3_credit_configuration")
  })
})
