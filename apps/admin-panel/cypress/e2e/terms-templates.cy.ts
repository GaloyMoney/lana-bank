describe("Terms Template", () => {
  it("should successfully create a new terms template", () => {
    cy.visit("/terms-templates")
    cy.screenshot("1_visit_terms_templates_page")

    cy.get('[data-testid="global-create-button"]').click()
    cy.screenshot("2_click_create_button")

    const templateName = `Test Template ${Date.now()}`
    cy.get('[data-testid="terms-template-name-input"]')
      .type(templateName)
      .should("have.value", templateName)
    cy.screenshot("3_enter_template_name")

    cy.get('[data-testid="terms-template-annual-rate-input"]')
      .type("5.5")
      .should("have.value", "5.5")
    cy.screenshot("4_enter_annual_rate")

    cy.get('[data-testid="terms-template-duration-units-input"]')
      .type("12")
      .should("have.value", "12")
    cy.screenshot("5_enter_duration_units")

    cy.get('[data-testid="terms-template-duration-period-select"]').select("MONTHS")
    cy.screenshot("6_select_duration_period")

    cy.get('[data-testid="terms-template-accrual-interval-select"]').select(
      "END_OF_MONTH",
    )
    cy.screenshot("7_select_accrual_interval")

    cy.get('[data-testid="terms-template-incurrence-interval-select"]').select(
      "END_OF_MONTH",
    )
    cy.screenshot("8_select_incurrence_interval")

    cy.get('[data-testid="terms-template-initial-cvl-input"]')
      .type("140")
      .should("have.value", "140")
    cy.screenshot("9_enter_initial_cvl")

    cy.get('[data-testid="terms-template-margin-call-cvl-input"]')
      .type("120")
      .should("have.value", "120")
    cy.screenshot("10_enter_margin_call_cvl")

    cy.get('[data-testid="terms-template-liquidation-cvl-input"]')
      .type("110")
      .should("have.value", "110")
    cy.screenshot("11_enter_liquidation_cvl")

    cy.get('[data-testid="terms-template-submit-button"]').click()
    cy.screenshot("12_submit_terms_template")

    cy.url().should(
      "match",
      /\/terms-templates\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
    )
    cy.contains(templateName).should("be.visible")
    cy.screenshot("13_verify_terms_template_creation")
  })
})
