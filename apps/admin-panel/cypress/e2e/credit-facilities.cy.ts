import {
  InterestInterval,
  Period,
  CreateCommitteeMutationResult,
  DisbursalPolicy,
} from "../../lib/graphql/generated"
import { DEFAULT_TERMS } from "../../lib/constants/terms"

import { t } from "../support/translation"

const CF = "CreditFacilities"
const CFP = "CreditFacilityProposals"
const PCF = "PendingCreditFacilities"
const Committee = "Committees.CommitteeDetails"
const Policy = "Policies.PolicyDetails"
const Disbursals = "Disbursals"

describe("credit facility", () => {
  let customerId: string
  let customerPublicId: string
  let proposalId: string
  let pendingFacilityId: string
  const termsTemplateName: string = `Test Template ${Date.now()}`

  before(() => {
    Cypress.env("creditFacilityPublicId", null)
    Cypress.env("creditFacilityProposalId", null)
    cy.createTermsTemplate({
      name: termsTemplateName,
      annualRate: "5.5",
      accrualCycleInterval: InterestInterval.EndOfMonth,
      accrualInterval: InterestInterval.EndOfDay,
      oneTimeFeeRate: "5",
      liquidationCvl: "110",
      marginCallCvl: "120",
      initialCvl: "140",
      disbursalPolicy: DisbursalPolicy.MultipleDisbursal,
      duration: {
        units: 12 * 100,
        period: Period.Months,
      },
      interestDueDurationFromAccrual: {
        units: DEFAULT_TERMS.INTEREST_DUE_DURATION_FROM_ACCRUAL.UNITS,
        period: DEFAULT_TERMS.INTEREST_DUE_DURATION_FROM_ACCRUAL.PERIOD,
      },
      obligationOverdueDurationFromDue: {
        units: DEFAULT_TERMS.OBLIGATION_OVERDUE_DURATION_FROM_DUE.UNITS,
        period: DEFAULT_TERMS.OBLIGATION_OVERDUE_DURATION_FROM_DUE.PERIOD,
      },
      obligationLiquidationDurationFromDue: {
        period: DEFAULT_TERMS.OBLIGATION_LIQUIDATION_DURATION_FROM_DUE.PERIOD,
        units: DEFAULT_TERMS.OBLIGATION_LIQUIDATION_DURATION_FROM_DUE.UNITS,
      },
    }).then((id) => {
      cy.log(`Created terms template with ID: ${id}`)
    })

    const testEmail = `t${Date.now().toString().slice(-6)}@example.com`
    const testTelegramId = `t${Date.now()}`
    cy.createCustomer(testEmail, testTelegramId).then((customer) => {
      customerId = customer.customerId
      customerPublicId = customer.publicId
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
      cy.get('[data-testid="table-row-1"] > :nth-child(3) > a').should(
        "be.visible",
      )
      cy.get('[data-testid="table-row-1"] > :nth-child(3) > a').click()
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
      cy.get('[data-testid="table-row-0"] > :nth-child(3) > a').should(
        "be.visible",
      )
      cy.get('[data-testid="table-row-0"] > :nth-child(3) > a').click()
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

  it("should create a credit facility proposal and verify initial state", () => {
    cy.visit(`/customers/${customerPublicId}`)
    cy.get('[data-testid="loading-skeleton"]').should("not.exist")

    cy.get('[data-testid="global-create-button"]').click()
    cy.get('[data-testid="create-credit-facility-button"]').should("be.visible").click()
    cy.takeScreenshot("01_click_create_proposal_button")
    cy.takeScreenshot("02_open_proposal_form")

    cy.get('[data-testid="facility-amount-input"]').type("5000")
    cy.get('[data-testid="credit-facility-terms-template-select"]').click()
    cy.get('[role="option"]').contains(termsTemplateName).click()

    cy.takeScreenshot("03_enter_facility_amount")

    cy.get('[data-testid="create-credit-facility-submit"]').click()
    cy.takeScreenshot("04_submit_proposal_form")

    cy.url()
      .should("match", /\/credit-facility-proposals\/[a-f0-9-]+$/)
      .then((url) => {
        proposalId = url.split("/").pop() as string
        Cypress.env("creditFacilityProposalId", proposalId)
      })

    cy.get("[data-testid=proposal-status-badge]")
      .should("be.visible")
      .invoke("text")
      .should("eq", t(CFP + ".status.pending_customer_approval"))
    cy.takeScreenshot("05_proposal_created_success")
  })

  it("should show newly created proposal in the list", () => {
    cy.visit(`/credit-facility-proposals`)
    cy.get('[data-testid="table-row-0"] > :nth-child(5) > a').click()
    cy.contains("$5,000.00").should("be.visible")
    cy.takeScreenshot("06_proposal_in_list")
  })

  it("should navigate to proposal and verify initial state", () => {
    const proposalUuid = Cypress.env("creditFacilityProposalId")
    expect(proposalUuid).to.exist

    cy.visit(`/credit-facility-proposals/${proposalUuid}`)
    cy.contains("$5,000").should("be.visible")
    cy.takeScreenshot("07_visit_proposal_page")

    cy.get('[data-testid="customer-approval-approve-button"]').should("be.visible")
  })

  it("should mark customer as accepting the proposal", () => {
    const proposalUuid = Cypress.env("creditFacilityProposalId")
    expect(proposalUuid).to.exist

    cy.visit(`/credit-facility-proposals/${proposalUuid}`)

    cy.get('[data-testid="customer-approval-approve-button"]').should("be.visible")
    cy.takeScreenshot("08_customer_approval_button")
    cy.get('[data-testid="customer-approval-approve-button"]').click()

    cy.wait(2000).then(() => {
      cy.takeScreenshot("09_customer_approval_dialog")
      cy.get('[data-testid="customer-approval-dialog-approve-button"]')
        .should("exist")
        .should("be.visible")
        .click()
      cy.wait(2000)
      cy.reload()
    })
  })

  it("should verify proposal is pending internal approval after customer acceptance", () => {
    const proposalUuid = Cypress.env("creditFacilityProposalId")
    expect(proposalUuid).to.exist
    cy.visit(`/credit-facility-proposals/${proposalUuid}`)
    cy.wait(2000)
    cy.reload()
    cy.get("[data-testid=proposal-status-badge]")
      .should("be.visible")
      .invoke("text")
      .should("eq", t(CFP + ".status.pending_approval"))
    cy.takeScreenshot("10_proposal_pending_approval_status")
  })

  it("should approve the proposal", () => {
    const proposalUuid = Cypress.env("creditFacilityProposalId")
    expect(proposalUuid).to.exist

    cy.visit(`/credit-facility-proposals/${proposalUuid}`)
    cy.reload()

    cy.get('[data-testid="approval-process-approve-button"]').should("be.visible")
    cy.takeScreenshot("11_approve_proposal_button")
    cy.get('[data-testid="approval-process-approve-button"]').click()

    cy.wait(2000).then(() => {
      cy.takeScreenshot("12_approve_proposal_dialog")
      cy.get('[data-testid="approval-process-dialog-approve-button"]')
        .should("exist")
        .should("be.visible")
        .click()
      cy.wait(2000)
    })
  })

  it("should verify proposal approved status", () => {
    const proposalUuid = Cypress.env("creditFacilityProposalId")
    expect(proposalUuid).to.exist
    cy.visit(`/credit-facility-proposals/${proposalUuid}`)
    cy.wait(2000)
    cy.reload()
    cy.get("[data-testid=proposal-status-badge]")
      .should("be.visible")
      .invoke("text")
      .should("eq", t(CFP + ".status.approved"))
    cy.takeScreenshot("13_proposal_approved_status")
  })

  it("should navigate to pending credit facility from proposal", () => {
    const proposalUuid = Cypress.env("creditFacilityProposalId")
    expect(proposalUuid).to.exist
    cy.visit(`/credit-facility-proposals/${proposalUuid}`)
    cy.get('[data-testid="view-pending-facility-button"]').should("be.visible")
    cy.takeScreenshot("14_view_pending_facility_button")
    cy.get('[data-testid="view-pending-facility-button"]').click()
    cy.url()
      .should("match", /\/pending-credit-facilities\/[a-f0-9-]+$/)
      .then((url) => {
        pendingFacilityId = url.split("/").pop() as string
        Cypress.env("pendingCreditFacilityId", pendingFacilityId)
        cy.log(`Pending credit facility ID: ${pendingFacilityId}`)
      })
  })

  it("should verify pending facility initial state", () => {
    const pendingUuid = Cypress.env("pendingCreditFacilityId")
    expect(pendingUuid).to.exist
    cy.visit(`/pending-credit-facilities/${pendingUuid}`)
    cy.contains("$5,000").should("be.visible")
    cy.get("[data-testid=pending-status-badge]")
      .should("be.visible")
      .invoke("text")
      .should("eq", t(PCF + ".status.pending_collateralization"))
    cy.takeScreenshot("15_pending_facility_initial_state")
  })

  it("should calculate and store required collateral amount", () => {
    const pendingUuid = Cypress.env("pendingCreditFacilityId")
    expect(pendingUuid).to.exist

    cy.visit(`/pending-credit-facilities/${pendingUuid}`)
    cy.wait(2000)

    cy.get('[data-testid="collateral-to-reach-target"]')
      .scrollIntoView()
      .should("be.visible")
      .invoke("text")
      .then((collateralValue) => {
        const numericValue = parseFloat(collateralValue.split(" ")[0])
        Cypress.env("requiredCollateralAmount", numericValue)
        cy.log(`Required collateral amount: ${numericValue}`)
      })
  })

  it("should update collateral in pending facility", () => {
    const pendingUuid = Cypress.env("pendingCreditFacilityId")
    const requiredAmount = Cypress.env("requiredCollateralAmount")
    expect(pendingUuid).to.exist
    expect(requiredAmount).to.exist

    cy.visit(`/pending-credit-facilities/${pendingUuid}`)
    cy.get('[data-testid="update-collateral-button"]').should("be.visible").click()
    cy.takeScreenshot("16_click_update_collateral_button")

    cy.get('[data-testid="new-collateral-input"]')
      .should("be.visible")
      .clear()
      .type(requiredAmount.toString())
    cy.takeScreenshot("17_enter_new_collateral_value")

    cy.get('[data-testid="proceed-to-confirm-button"]')
      .should("be.visible")
      .then(($el) => {
        $el.on("click", (e) => e.preventDefault())
      })
      .click()

    cy.get('[data-testid="confirm-update-button"]')
      .should("be.visible")
      .click()
      .wait(2000)
    cy.takeScreenshot("18_collateral_updated")
  })

  it("should verify pending facility completed status", () => {
    const pendingUuid = Cypress.env("pendingCreditFacilityId")
    expect(pendingUuid).to.exist
    cy.visit(`/pending-credit-facilities/${pendingUuid}`)
    cy.wait(2000)
    cy.reload()
    cy.get("[data-testid=pending-status-badge]")
      .should("be.visible")
      .invoke("text")
      .should("eq", t(PCF + ".status.completed"))
    cy.takeScreenshot("19_pending_facility_completed")
  })

  it("should navigate to created credit facility from pending facility", () => {
    const pendingUuid = Cypress.env("pendingCreditFacilityId")
    expect(pendingUuid).to.exist
    cy.visit(`/pending-credit-facilities/${pendingUuid}`)
    cy.get('[data-testid="view-facility-button"]').should("be.visible")
    cy.takeScreenshot("20_view_facility_button")
    cy.get('[data-testid="view-facility-button"]').click()
    cy.url()
      .should("match", /\/credit-facilities\/\d+$/)
      .then((url) => {
        const publicId = url.split("/").pop() as string
        Cypress.env("creditFacilityPublicId", publicId)
        cy.log(`Credit facility public ID: ${publicId}`)
      })
  })

  it("should verify credit facility is active", () => {
    const publicId = Cypress.env("creditFacilityPublicId")
    expect(publicId).to.exist

    cy.visit(`/credit-facilities/${publicId}`)

    cy.get("[data-testid=credit-facility-status-badge]")
      .should("be.visible")
      .invoke("text")
      .should("eq", t(CF + ".CreditFacilityStatus.active"))
    cy.takeScreenshot("21_verify_active_status")
  })

  it("should show newly created credit facility in the list", () => {
    cy.visit(`/credit-facilities`)
    cy.get('[data-testid="table-row-0"] > :nth-child(7) > a').click()
    cy.contains("$5,000.00").should("be.visible")
    cy.takeScreenshot("22_credit_facility_in_list")
  })

  it("should successfully initiate and confirm a disbursal", () => {
    const publicId = Cypress.env("creditFacilityPublicId")
    expect(publicId).to.exist

    cy.visit(`/credit-facilities/${publicId}`)
    cy.contains("$5,000").should("be.visible")

    cy.get('[data-testid="global-create-button"]').click()
    cy.get('[data-testid="initiate-disbursal-button"]').should("be.visible").click()
    cy.takeScreenshot("23_click_initiate_disbursal_button")

    cy.get('[data-testid="disbursal-amount-input"]')
      .type("1000")
      .should("have.value", "1,000")
    cy.takeScreenshot("24_enter_disbursal_amount")

    cy.get('[data-testid="disbursal-submit-button"]').click()
    cy.takeScreenshot("25_submit_disbursal_request")

    cy.url().should("match", /\/disbursals\/\w+$/)

    cy.takeScreenshot("26_disbursal_page")

    cy.reload()
    cy.get('[data-testid="disbursal-approve-button"]').should("be.visible").click()
    cy.wait(2000).then(() => {
      cy.takeScreenshot("27_approve")
      cy.get('[data-testid="approval-process-dialog-approve-button"]')
        .should("be.visible")
        .click()

      cy.wait(2000).then(() => {
        cy.reload()
        cy.get('[data-testid="disbursal-status-badge"]')
          .should("be.visible")
          .invoke("text")
          .should("eq", t(Disbursals + ".DisbursalStatus.confirmed"))
        cy.takeScreenshot("28_verify_disbursal_status_confirmed")
      })
    })
  })

  it("should show disbursal in the list page", () => {
    cy.visit(`/disbursals`)
    cy.contains("$1,000.00").should("be.visible")
    cy.takeScreenshot("29_disbursal_in_list")
  })
})
