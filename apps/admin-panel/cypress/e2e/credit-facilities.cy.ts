import {
  InterestInterval,
  Period,
  CreateCommitteeMutationResult,
} from "../../lib/graphql/generated/index"

import { t } from "../support/translation"

const CF = "CreditFacilities"
const Committee = "Committees.CommitteeDetails"
const Policy = "Policies.PolicyDetails"

describe("credit facility", () => {
  let customerId: string
  const termsTemplateName: string = `Test Template ${Date.now()}`

  before(() => {
    Cypress.env("creditFacilityId", null)
    cy.createTermsTemplate({
      name: termsTemplateName,
      annualRate: "5.5",
      accrualInterval: InterestInterval.EndOfDay,
      incurrenceInterval: InterestInterval.EndOfMonth,
      oneTimeFeeRate: "5",
      liquidationCvl: "110",
      marginCallCvl: "120",
      initialCvl: "140",
      duration: {
        units: 12 * 5,
        period: Period.Months,
      },
      interestDueDuration: {
        units: 0,
        period: Period.Days,
      },
    }).then((id) => {
      cy.log(`Created terms template with ID: ${id}`)
    })

    const testEmail = `t${Date.now().toString().slice(-6)}@example.com`
    const testTelegramId = `t${Date.now()}`
    cy.createCustomer(testEmail, testTelegramId).then((customer) => {
      customerId = customer.customerId
      cy.log(`Created customer with ID: ${customerId}`)
    })
  })

  beforeEach(() => {
    cy.on("uncaught:exception", (err) => {
      if (err.message.includes("ResizeObserver loop")) {
        return false
      }
    })
  })

  it("should add admin to credit facility and disbursal approvers", () => {
    const committeeName = `${Date.now()}-CF-and-Disbursal-Approvers`
    const createCommitteeMutation = `mutation CreateCommittee($input: CommitteeCreateInput!) {
      committeeCreate(input: $input) {
        committee {
          committeeId
        }
      }
    }`
    cy.graphqlRequest<CreateCommitteeMutationResult>(createCommitteeMutation, {
      input: { name: committeeName },
    }).then((response) => {
      const committeeId = response.data?.committeeCreate.committee.committeeId
      cy.visit(`/committees/${committeeId}`)
      cy.get('[data-testid="committee-add-member-button"]').click()
      cy.get('[data-testid="committee-add-user-select"]').should("be.visible").click()
      cy.get('[role="option"]')
        .contains("admin")
        .then((option) => {
          cy.wrap(option).click()
          cy.get('[data-testid="committee-add-user-submit-button"]').click()
          cy.contains(t(Committee + ".AddUserCommitteeDialog.success")).should(
            "be.visible",
          )
          cy.contains(option.text().split(" ")[0]).should("be.visible")
        })

      cy.visit(`/policies`)
      cy.get('[data-testid="table-row-1"] > :nth-child(3) > a > .gap-2').should(
        "be.visible",
      )
      cy.get('[data-testid="table-row-1"] > :nth-child(3) > a > .gap-2').click()
      cy.get('[data-testid="policy-assign-committee"]').click()
      cy.get('[data-testid="policy-select-committee-selector"]').click()
      cy.get('[role="option"]').contains(committeeName).click()
      cy.get("[data-testid=policy-assign-committee-threshold-input]").type("1")
      cy.get("[data-testid=policy-assign-committee-submit-button]").click()
      cy.contains(t(Policy + ".CommitteeAssignmentDialog.success.assigned")).should(
        "be.visible",
      )
      cy.contains(committeeName).should("be.visible")

      cy.visit(`/policies`)
      cy.get('[data-testid="table-row-0"] > :nth-child(3) > a > .gap-2').should(
        "be.visible",
      )
      cy.get('[data-testid="table-row-0"] > :nth-child(3) > a > .gap-2').click()
      cy.get('[data-testid="policy-assign-committee"]').click()
      cy.get('[data-testid="policy-select-committee-selector"]').click()
      cy.get('[role="option"]').contains(committeeName).click()
      cy.get("[data-testid=policy-assign-committee-threshold-input]").type("1")
      cy.get("[data-testid=policy-assign-committee-submit-button]").click()
      cy.contains(t(Policy + ".CommitteeAssignmentDialog.success.assigned")).should(
        "be.visible",
      )
      cy.contains(committeeName).should("be.visible")
    })
  })

  it("should create a credit facility and verify initial state", () => {
    cy.visit(`/customers/${customerId}`)
    cy.get('[data-testid="loading-skeleton"]').should("not.exist")

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("1_click_create_credit_facility_button")

    cy.get('[data-testid="create-credit-facility-button"]').should("be.visible").click()
    cy.takeScreenshot("2_open_credit_facility_form")

    cy.get('[data-testid="facility-amount-input"]').type("5000", {
      delay: 0,
      waitForAnimations: false,
    })
    cy.get('[data-testid="credit-facility-terms-template-select"]').click()
    cy.get('[role="option"]').contains(termsTemplateName).click()

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

    cy.contains("No Collateral").should("be.visible")
    cy.takeScreenshot("5_credit_facility_created_success")
  })

  it("should show newly created credit facility in the list", () => {
    cy.visit(`/credit-facilities`)
    cy.get('[data-testid="table-row-0"] > :nth-child(7) > a > .gap-2').click()
    cy.contains("$5,000.00").should("be.visible")
    cy.takeScreenshot("credit_facility_in_list")
  })

  it("should update collateral, approve and activate the credit facility", () => {
    const creditFacilityId = Cypress.env("creditFacilityId")
    expect(creditFacilityId).to.exist

    cy.visit(`/credit-facilities/${creditFacilityId}`)
    cy.contains("$5,000").should("be.visible")
    cy.takeScreenshot("6_visit_credit_facility_page")

    cy.wait(2000)
    cy.get('[data-testid="collateral-to-reach-target"]')
      .should("be.visible")
      .invoke("text")
      .then((collateralValue) => {
        const numericValue = parseFloat(collateralValue.split(" ")[0])

        cy.log(`Collateral value: ${numericValue}`)
        cy.get('[data-testid="update-collateral-button"]').should("be.visible").click()
        cy.takeScreenshot("7_click_update_collateral_button")

        cy.get('[data-testid="new-collateral-input"]')
          .should("be.visible")
          .clear()
          .type(numericValue.toString(), { delay: 0, waitForAnimations: false })
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

        cy.get('[data-testid="credit-facility-approve-button"]')
          .should("be.visible")
          .click()
        cy.wait(5000).then(() => {
          cy.takeScreenshot("9_1_approve")
          cy.get('[data-testid="approval-process-dialog-approve-button"]')
            .should("be.visible")
            .click()

          cy.wait(5000).then(() => {
            cy.reload().then(() => {
              cy.get("[data-testid=credit-facility-status-badge]")
                .should("be.visible")
                .invoke("text")
                .should("eq", t(CF + ".CreditFacilityStatus.active").toUpperCase())
              cy.takeScreenshot("10_verify_active_status")
            })
          })
        })
      })
  })

  it("should successfully initiate and confirm a disbursal", () => {
    const creditFacilityId = Cypress.env("creditFacilityId")
    expect(creditFacilityId).to.exist

    cy.visit(`/credit-facilities/${creditFacilityId}`)
    cy.contains("$5,000").should("be.visible")
    cy.takeScreenshot("11_visit_credit_facility_page_for_disbursal")

    cy.get('[data-testid="global-create-button"]').click()
    cy.get('[data-testid="initiate-disbursal-button"]').should("be.visible").click()
    cy.takeScreenshot("12_click_initiate_disbursal_button")

    cy.get('[data-testid="disbursal-amount-input"]')
      .type("1000", { delay: 0, waitForAnimations: false })
      .should("have.value", "1,000")
    cy.takeScreenshot("13_enter_disbursal_amount")

    cy.get('[data-testid="disbursal-submit-button"]').click()
    cy.takeScreenshot("14_submit_disbursal_request")

    cy.url().should(
      "match",
      /\/disbursals\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
    )

    cy.takeScreenshot("15_disbursal_page")
    cy.takeScreenshot("16_disbursal_success_message")

    cy.reload()
    cy.get('[data-testid="disbursal-approve-button"]').should("be.visible").click()
    cy.wait(5000).then(() => {
      cy.takeScreenshot("16_1_approve")
      cy.get('[data-testid="approval-process-dialog-approve-button"]')
        .should("be.visible")
        .click()

      cy.wait(5000).then(() => {
        cy.get('[data-testid="disbursal-status-badge"]')
          .should("be.visible")
          .invoke("text")
          .should("eq", t("Disbursals.DisbursalStatus.confirmed").toUpperCase())
        cy.takeScreenshot("17_verify_disbursal_status_confirmed")
      })
    })
  })

  it("should show disbursal in the list page", () => {
    cy.visit(`/disbursals`)
    cy.contains("$1,000.00").should("be.visible")
    cy.takeScreenshot("18_disbursal_in_list")
  })
})
