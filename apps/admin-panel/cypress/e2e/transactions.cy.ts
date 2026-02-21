import { faker } from "@faker-js/faker"

import { t } from "../support/translation"

const D = "Deposits"
const W = "Withdrawals"
const usdFormatter = new Intl.NumberFormat("en-US")
const formatUsd = (amount: number) =>
  `$${usdFormatter.format(amount)}.00`

describe("Transactions Deposit and Withdraw", () => {
  let customerId: string
  let customerPublicId: string
  let depositAccountId: string
  let depositAccountPublicId: string
  const depositAmount = faker.number.int({ min: 1000, max: 5000 })
  const withdrawAmount = faker.number.int({ min: 1000, max: depositAmount })

  before(() => {
    const testEmail = `t${Date.now().toString().slice(-6)}@example.com`
    const testTelegramId = `t${Date.now()}`
    cy.createCustomer(testEmail, testTelegramId).then((customer) => {
      customerId = customer.customerId
      customerPublicId = customer.publicId
      depositAccountId = customer.depositAccount.depositAccountId
      depositAccountPublicId = customer.depositAccount.publicId
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

  it("should create a Deposit", () => {
    cy.visit(`/deposit-accounts/${depositAccountPublicId}`)
    cy.wait(1000)

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("1_deposit_create_button")

    cy.get('[data-testid="create-deposit-button"]').should("be.visible").click()
    cy.takeScreenshot("2_deposit_select")

    // Create dialog
    cy.get('[data-testid="deposit-amount-input"]').type(depositAmount.toString())
    cy.takeScreenshot("3_deposit_enter_amount")

    cy.get('[data-testid="deposit-submit-button"]').click()
    cy.takeScreenshot("4_deposit_submit")

    cy.contains(t(D + ".CreateDepositDialog.success")).should("be.visible")
    cy.takeScreenshot("5_deposit_success")
  })

  it("should show newly created Deposit in list page", () => {
    cy.visit(`/deposits`)
    cy.get("table", { timeout: 10000 }).should("exist")
    cy.contains(formatUsd(depositAmount)).should("be.visible")
    cy.takeScreenshot("6_deposit_in_list")
  })

  it("should show newly created Deposit in customer details page", () => {
    cy.visit(`/customers/${customerPublicId}`)
    cy.contains(formatUsd(depositAmount)).should("be.visible")
    cy.takeScreenshot("7_deposit_in_transactions")
  })

  it("should create Withdraw", () => {
    cy.visit(`/deposit-accounts/${depositAccountPublicId}`)
    cy.wait(1000)

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("8_withdrawal_create_button")

    cy.get('[data-testid="create-withdrawal-button"]').should("be.visible").click()
    cy.takeScreenshot("9_withdrawal_select")

    cy.get('[data-testid="withdraw-amount-input"]').type(withdrawAmount.toString())
    cy.takeScreenshot("10_withdrawal_enter_amount")

    cy.get('[data-testid="withdraw-submit-button"]').click()

    cy.url({ timeout: 40000 })
      .should("include", "/withdrawals/")
      .then(() => {
        cy.contains(formatUsd(withdrawAmount)).should("be.visible")
        cy.takeScreenshot("11_withdrawal_submit")
      })
  })

  it("should show newly created Withdraw in list page", () => {
    cy.visit(`/withdrawals`)
    cy.get("table", { timeout: 10000 }).should("exist")
    cy.contains(formatUsd(withdrawAmount)).should("be.visible")
    cy.takeScreenshot("12_withdrawal_in_list")
  })

  it("should show newly created Withdraw in customer details page", () => {
    cy.visit(`/customers/${customerPublicId}`)
    cy.contains(formatUsd(withdrawAmount)).should("be.visible")
    cy.takeScreenshot("13_withdrawal_in_transactions")
  })

  it("should show newly created Withdraw in list page", () => {
    console.log("should show newly created Withdraw in list page")

    cy.createDeposit(depositAmount, depositAccountId).then(() => {
      cy.initiateWithdrawal(withdrawAmount, depositAccountId).then(
        (withdrawalPublicId) => {
          cy.visit(`/withdrawals/${withdrawalPublicId}`)
          cy.wait(1000)
          cy.get("[data-testid=withdrawal-status-badge]").then((badge) => {
            if (badge.text() === t(W + ".WithdrawalStatus.pending_approval")) {
              // case when we have policy attached for withdrawal no ss needed here
              cy.get('[data-testid="approval-process-deny-button"]').click()
              cy.get('[data-testid="approval-process-dialog-deny-reason"]').type(
                "testing",
              )
              cy.get('[data-testid="approval-process-dialog-deny-button"]').click()
            } else {
              // expected flow
              cy.get('[data-testid="withdraw-cancel-button"]')
                .should("be.visible")
                .click()
              cy.takeScreenshot("14_withdrawal_cancel_button")

              cy.get('[data-testid="withdrawal-cancel-dialog-button"]')
                .should("be.visible")
                .click()
              cy.takeScreenshot("15_withdrawal_cancel_confirm")

              cy.get("[data-testid=withdrawal-status-badge]")
                .should("be.visible")
                .invoke("text")
                .should("eq", t(W + ".WithdrawalStatus.cancelled"))
              cy.takeScreenshot("16_withdrawal_cancelled_status")
            }
          })
        },
      )
    })
  })

  it("should approve Withdraw", () => {
    cy.createDeposit(depositAmount, depositAccountId).then(() => {
      cy.initiateWithdrawal(withdrawAmount, depositAccountId).then(
        (withdrawalPublicId) => {
          cy.visit(`/withdrawals/${withdrawalPublicId}`)
          cy.wait(1000)
          cy.get("[data-testid=withdrawal-status-badge]")
            .then((badge) => {
              // case when we have policy attached for withdrawal no ss needed here
              if (badge.text() === t(W + ".WithdrawalStatus.pending_approval")) {
                cy.get('[data-testid="approval-process-approve-button"]').click()
                cy.get('[data-testid="approval-process-dialog-approve-button"]').click()
              }
            })
            .then(() => {
              cy.get('[data-testid="withdraw-confirm-button"]')
                .should("be.visible")
                .click()
              cy.takeScreenshot("17_withdrawal_approve_button")

              cy.get('[data-testid="withdrawal-confirm-dialog-button"]')
                .should("be.visible")
                .click()
              cy.takeScreenshot("18_withdrawal_approve_confirm")

              cy.get("[data-testid=withdrawal-status-badge]")
                .should("be.visible")
                .invoke("text")
                .should("eq", t(W + ".WithdrawalStatus.confirmed"))
              cy.takeScreenshot("19_withdrawal_approved_status")
            })
        },
      )
    })
  })
})
