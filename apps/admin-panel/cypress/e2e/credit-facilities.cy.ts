import { InterestInterval, Period } from "../../lib/graphql/generated/index"

describe("credit facility", () => {
  let customerId: string

  before(() => {
    Cypress.env("creditFacilityId", null)
    cy.createTermsTemplate({
      name: `Test Template ${Date.now()}`,
      annualRate: "5.5",
      accrualInterval: InterestInterval.EndOfMonth,
      incurrenceInterval: InterestInterval.EndOfMonth,
      oneTimeFeeRate: "5",
      liquidationCvl: "110",
      marginCallCvl: "120",
      initialCvl: "140",
      duration: {
        units: 12,
        period: Period.Months,
      },
    }).then((id) => {
      cy.log(`Created terms template with ID: ${id}`)
    })

    const testEmail = `test-${Date.now()}@example.com`
    const testTelegramId = `user${Date.now()}`
    cy.createCustomer(testEmail, testTelegramId).then((id) => {
      customerId = id
      cy.log(`Created customer with ID: ${id}`)
    })
  })

  beforeEach(() => {
    cy.on("uncaught:exception", (err) => {
      if (err.message.includes("ResizeObserver loop")) {
        return false
      }
    })
  })

  it("should create a credit facility and verify initial state", () => {
    cy.visit(`/customers/${customerId}`)

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("1_click_create_credit_facility_button")

    cy.get('[data-testid="create-credit-facility-button"]').should("be.visible").click()
    cy.takeScreenshot("2_open_credit_facility_form")

    cy.get('[data-testid="facility-amount-input"]')
      .type("5000")
      .should("have.value", "5000")
    cy.takeScreenshot("3_enter_facility_amount")

    cy.get('[data-testid="create-credit-facility-submit"]').click()
    cy.takeScreenshot("4_submit_credit_facility_form")

    cy.url()
      .should(
        "match",
        /\/credit-facilities\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
      )
      .then((url) => {
        const facilityId = url.split("/").pop()
        Cypress.env("creditFacilityId", facilityId)
      })

    cy.contains("Credit Facility created successfully").should("be.visible")
    cy.takeScreenshot("5_credit_facility_created_success")
  })

  it("should update collateral and activate the credit facility", () => {
    const creditFacilityId = Cypress.env("creditFacilityId")
    expect(creditFacilityId).to.exist

    cy.visit(`/credit-facilities/${creditFacilityId}`)
    cy.takeScreenshot("6_visit_credit_facility_page")

    cy.get('[data-testid="collateral-to-reach-target"]')
      .should("be.visible")
      .invoke("text")
      .then((collateralValue) => {
        const numericValue = parseFloat(collateralValue.split(" ")[0])

        cy.get('[data-testid="update-collateral-button"]').should("be.visible").click()
        cy.takeScreenshot("7_click_update_collateral_button")

        cy.get('[data-testid="new-collateral-input"]')
          .should("be.visible")
          .clear()
          .type(numericValue.toString())
        cy.takeScreenshot("8_enter_new_collateral_value")

        cy.get('[data-testid="proceed-to-confirm-button"]').should("be.visible")
        cy.takeScreenshot("9_confirm_collateral_update")

        cy.get('[data-testid="proceed-to-confirm-button"]')
          .should("be.visible")
          .then(($el) => {
            $el.on("click", (e) => e.preventDefault())
          })
          .click()

        cy.get('[data-testid="confirm-update-button"]').should("be.visible").click()

        cy.get("[data-testid=credit-facility-status-badge]")
          .should("be.visible")
          .invoke("text")
          .should("eq", "ACTIVE")
        cy.takeScreenshot("10_verify_active_status")
      })
  })

  it("should successfully initiate and confirm a disbursal", () => {
    const creditFacilityId = Cypress.env("creditFacilityId")
    expect(creditFacilityId).to.exist

    cy.visit(`/credit-facilities/${creditFacilityId}`)
    cy.takeScreenshot("11_visit_credit_facility_page_for_disbursal")

    cy.get('[data-testid="initiate-disbursal-button"]').should("be.visible").click()
    cy.takeScreenshot("12_click_initiate_disbursal_button")

    cy.get('[data-testid="disbursal-amount-input"]')
      .type("1000")
      .should("have.value", "1000")
    cy.takeScreenshot("13_enter_disbursal_amount")

    cy.get('[data-testid="disbursal-submit-button"]').click()
    cy.takeScreenshot("14_submit_disbursal_request")

    cy.url().should(
      "match",
      /\/disbursals\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
    )

    cy.contains("Disbursal initiated successfully").should("be.visible")
    cy.takeScreenshot("15_disbursal_page")
    cy.takeScreenshot("16_disbursal_success_message")

    cy.get('[data-testid="disbursal-status-badge"]')
      .should("be.visible")
      .invoke("text")
      .should("eq", "CONFIRMED")
    cy.takeScreenshot("17_verify_disbursal_status_confirmed")
  })
})
