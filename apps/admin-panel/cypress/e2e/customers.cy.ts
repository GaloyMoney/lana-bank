describe("Customers", () => {
  it("should successfully create a new customer", () => {
    cy.visit("/customers")
    cy.screenshot("1_visit_customers_page")

    cy.contains(
      "Individuals or entities who hold accounts, loans, or credit facilities with the bank",
      { timeout: 10000 },
    )
    cy.screenshot("2_list_all_customers")

    cy.get('[data-testid="global-create-button"]').click()
    cy.screenshot("3_click_create_button")

    cy.get('[data-testid="customer-create-email"]').should("be.visible")
    cy.screenshot("4_verify_email_input_visible")

    const testEmail = `test-${Date.now()}@example.com`
    cy.get('[data-testid="customer-create-email"]')
      .type(testEmail)
      .should("have.value", testEmail)
    cy.screenshot("5_enter_email")

    const testTelegramId = `user${Date.now()}`
    cy.get('[data-testid="customer-create-telegram-id"]')
      .type(testTelegramId)
      .should("have.value", testTelegramId)
    cy.screenshot("6_enter_telegram_id")

    cy.get('[data-testid="customer-create-submit-button"]')
      .contains("Review Details")
      .click()
    cy.screenshot("7_click_review_details")

    cy.contains(testEmail).should("be.visible")
    cy.contains(testTelegramId).should("be.visible")
    cy.screenshot("8_verify_details")

    cy.get('[data-testid="customer-create-submit-button"]')
      .contains("Confirm and Submit")
      .click()
    cy.screenshot("9_click_confirm_submit")

    cy.url().should(
      "match",
      /\/customers\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
    )
    cy.contains(testEmail).should("be.visible")
    cy.screenshot("10_verify_email")
  })
})
