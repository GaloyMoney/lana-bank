describe("Terms Template", () => {
  beforeEach(() => {
    cy.on("uncaught:exception", (err) => {
      if (err.message.includes("ResizeObserver loop")) {
        return false
      }
    })
  })

  it("should successfully create a new terms template", () => {
    cy.visit("/terms-templates")

    cy.takeScreenshot("1_visit_terms_templates_page")

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("2_click_create_button")

    const templateName = `Test Template ${Date.now()}`

    cy.get('[data-testid="terms-template-name-input"]')
      .type(templateName)
      .should("have.value", templateName)
    cy.takeScreenshot("3_enter_template_name")

    cy.get('[data-testid="terms-template-annual-rate-input"]')
      .type("5.5")
      .should("have.value", "5.5")
    cy.takeScreenshot("4_enter_annual_rate")

    cy.get('[data-testid="terms-template-duration-units-input"]')
      .type("12")
      .should("have.value", "12")
    cy.takeScreenshot("5_enter_duration_units")

    cy.get('[data-testid="terms-template-duration-period-select"]').click()
    cy.get('[role="option"]').contains("Months").click()
    cy.takeScreenshot("6_select_duration_period")

    cy.get('[data-testid="terms-template-accrual-interval-select"]').click()
    cy.get('[role="option"]').contains("End Of Month").click()
    cy.takeScreenshot("7_select_accrual_interval")

    cy.get('[data-testid="terms-template-incurrence-interval-select"]').click()
    cy.get('[role="option"]').contains("End Of Month").click()
    cy.takeScreenshot("8_select_incurrence_interval")

    cy.get('[data-testid="terms-template-initial-cvl-input"]')
      .type("140")
      .should("have.value", "140")
    cy.takeScreenshot("9_enter_initial_cvl")

    cy.get('[data-testid="terms-template-margin-call-cvl-input"]')
      .type("120")
      .should("have.value", "120")
    cy.takeScreenshot("10_enter_margin_call_cvl")

    cy.get('[data-testid="terms-template-liquidation-cvl-input"]')
      .type("110")
      .should("have.value", "110")
    cy.takeScreenshot("11_enter_liquidation_cvl")

    cy.get('[data-testid="terms-template-submit-button"]').click()
    cy.takeScreenshot("12_submit_terms_template")

    cy.url().should(
      "match",
      /\/terms-templates\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
    )
    cy.contains(templateName).should("be.visible")
    cy.takeScreenshot("13_verify_terms_template_creation")
  })
})
