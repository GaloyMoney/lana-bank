import { t } from "../support/translation"

const TTDetails = "TermsTemplates.TermsTemplateDetails"

describe("Terms Template", () => {
  let templateName: string
  let templateId: string

  beforeEach(() => {
    cy.on("uncaught:exception", (err) => {
      if (err.message.includes("ResizeObserver loop")) {
        return false
      }
    })
  })

  it("should successfully create a new terms template", () => {
    templateName = `Test Template ${Date.now()}`
    cy.visit("/terms-templates")

    cy.takeScreenshot("1_visit_terms_templates_page")

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("2_click_create_button")

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

    cy.get('[data-testid="terms-template-initial-cvl-input"]')
      .type("140")
      .should("have.value", "140")
    cy.takeScreenshot("6_enter_initial_cvl")

    cy.get('[data-testid="terms-template-margin-call-cvl-input"]')
      .type("120")
      .should("have.value", "120")
    cy.takeScreenshot("7_enter_margin_call_cvl")

    cy.get('[data-testid="terms-template-liquidation-cvl-input"]')
      .type("110")
      .should("have.value", "110")
    cy.takeScreenshot("8_enter_liquidation_cvl")

    cy.get('[data-testid="terms-template-one-time-fee-rate-input"]')
      .type("5")
      .should("have.value", "5")
    cy.takeScreenshot("9_enter_fee_rate")

    cy.get('[data-testid="terms-template-submit-button"]').click()
    cy.takeScreenshot("10_submit_terms_template")

    cy.url().should(
      "match",
      /\/terms-templates\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
    )
    cy.contains(templateName).should("be.visible")
    cy.contains(t(TTDetails + ".CreateTermsTemplate.title")).should("not.exist")
    cy.takeScreenshot("11_verify_terms_template_creation")

    cy.getIdFromUrl("/terms-templates/").then((id) => {
      templateId = id
      cy.log(`Template ID: ${templateId}`)
    })
  })

  it("should show newly created terms template in the list", () => {
    cy.visit("/terms-templates")
    cy.wait(1000)
    cy.contains(templateName).should("be.visible")
    cy.takeScreenshot("12_terms_template_in_list")
  })

  it("should update the terms template", () => {
    cy.visit(`/terms-templates/${templateId}`)
    cy.wait(1000)
    cy.takeScreenshot("13_terms_template_details")

    cy.get('[data-testid="terms-template-update-button"]').click()
    cy.takeScreenshot("14_click_update_button")

    cy.get('[data-testid="terms-template-annual-rate-input"]')
      .type("6")
      .should("have.value", "6")
    cy.takeScreenshot("15_update_annual_rate")

    cy.get('[data-testid="terms-template-update-submit-button"]').click()
    cy.takeScreenshot("16_submit_update")

    cy.contains(t(TTDetails + ".UpdateTermsTemplate.success.updated")).should(
      "be.visible",
    )
    cy.takeScreenshot("17_update_success")
  })
})
