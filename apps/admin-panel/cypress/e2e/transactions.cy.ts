import { faker } from "@faker-js/faker"

describe("Transactions Deposit and Withdraw", () => {
  let customerId: string
  let depositAccountId: string
  const depositAmount = faker.number.int({ min: 1000, max: 5000 })
  const withdrawAmount = faker.number.int({ min: 1000, max: depositAmount })

  before(() => {
    const testEmail = `t${Date.now().toString().slice(-6)}@example.com`
    const testTelegramId = `t${Date.now()}`
    cy.createCustomer(testEmail, testTelegramId).then((customer) => {
      customerId = customer.customerId
      depositAccountId = customer.depositAccount.depositAccountId
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
    cy.visit(`/customers/${customerId}`)
    cy.wait(1000)

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("1_deposit_create_button")

    cy.get('[data-testid="create-deposit-button"]').should("be.visible").click()
    cy.takeScreenshot("2_deposit_select")

    // Create dialog
    cy.get('[data-testid="deposit-amount-input"]').type(depositAmount.toString(), {
      delay: 0,
      waitForAnimations: false,
    })
    cy.takeScreenshot("3_deposit_enter_amount")

    cy.get('[data-testid="deposit-submit-button"]').click()
    cy.takeScreenshot("4_deposit_submit")

    cy.contains("Deposit created successfully").should("be.visible")
    cy.takeScreenshot("5_deposit_success")
  })

  it("should show newly created Deposit in list page", () => {
    cy.visit(`/deposits`)
    cy.contains(`$${depositAmount.toLocaleString()}.00`).should("be.visible")
    cy.takeScreenshot("6_deposit_in_list")
  })

  it("should show newly created Deposit in customer details page", () => {
    cy.visit(`/customers/${customerId}`)
    cy.contains(`$${depositAmount.toLocaleString()}.00`).should("be.visible")
    cy.takeScreenshot("7_deposit_in_transactions")
  })

  it("should create Withdraw", () => {
    cy.visit(`/customers/${customerId}`)
    cy.wait(1000)

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("8_withdrawal_create_button")

    cy.get('[data-testid="create-withdrawal-button"]').should("be.visible").click()
    cy.takeScreenshot("9_withdrawal_select")

    cy.get('[data-testid="withdraw-amount-input"]').type(withdrawAmount.toString(), {
      delay: 0,
      waitForAnimations: false,
    })
    cy.takeScreenshot("10_withdrawal_enter_amount")

    cy.get('[data-testid="withdraw-submit-button"]').click()

    cy.url()
      .should("include", "/withdrawals/")
      .then(() => {
        cy.contains(`$${withdrawAmount.toLocaleString()}.00`).should("be.visible")
        cy.takeScreenshot("11_withdrawal_submit")
      })
  })

  it("should show newly created Withdraw in list page", () => {
    cy.visit(`/withdrawals`)
    cy.contains(`$${withdrawAmount.toLocaleString()}.00`).should("be.visible")
    cy.takeScreenshot("12_withdrawal_in_list")
  })

  it("should show newly created Withdraw in customer details page", () => {
    cy.visit(`/customers/${customerId}`)
    cy.contains(`$${withdrawAmount.toLocaleString()}.00`).should("be.visible")
    cy.takeScreenshot("13_withdrawal_in_transactions")
  })

  it("should show newly created Withdraw in list page", () => {
    console.log("should show newly created Withdraw in list page")

    cy.createDeposit(depositAmount, depositAccountId).then(() => {
      cy.initiateWithdrawal(withdrawAmount, depositAccountId).then((withdrawalId) => {
        cy.visit(`/withdrawals/${withdrawalId}`)
        cy.wait(1000)
        cy.get("[data-testid=withdrawal-status-badge]").then((badge) => {
          if (badge.text() === "PENDING APPROVAL") {
            // case when we have policy attached for withdrawal no ss needed here
            cy.get('[data-testid="approval-process-deny-button"]').click()
            cy.get('[data-testid="approval-process-dialog-deny-reason"]').type(
              "testing",
              { delay: 0, waitForAnimations: false },
            )
            cy.get('[data-testid="approval-process-dialog-deny-button"]').click()
          } else {
            // expected flow
            cy.get('[data-testid="withdraw-cancel-button"]').should("be.visible").click()
            cy.takeScreenshot("14_withdrawal_cancel_button")

            cy.get('[data-testid="withdrawal-confirm-dialog-button"]')
              .should("be.visible")
              .click()
            cy.takeScreenshot("15_withdrawal_cancel_confirm")

            cy.get("[data-testid=withdrawal-status-badge]")
              .should("be.visible")
              .invoke("text")
              .should("eq", "CANCELLED")
            cy.takeScreenshot("16_withdrawal_cancelled_status")
          }
        })
      })
    })
  })

  it("should approve Withdraw", () => {
    cy.createDeposit(depositAmount, depositAccountId).then(() => {
      cy.initiateWithdrawal(withdrawAmount, depositAccountId).then((withdrawalId) => {
        cy.visit(`/withdrawals/${withdrawalId}`)
        cy.wait(1000)
        cy.get("[data-testid=withdrawal-status-badge]")
          .then((badge) => {
            // case when we have policy attached for withdrawal no ss needed here
            if (badge.text() === "PENDING APPROVAL") {
              cy.get('[data-testid="approval-process-approve-button"]').click()
              cy.get('[data-testid="approval-process-dialog-approve-button"]').click()
            }
          })
          .then(() => {
            cy.get('[data-testid="withdraw-confirm-button"]').should("be.visible").click()
            cy.takeScreenshot("17_withdrawal_approve_button")

            cy.get('[data-testid="withdrawal-confirm-dialog-button"]')
              .should("be.visible")
              .click()
            cy.takeScreenshot("18_withdrawal_approve_confirm")

            cy.get("[data-testid=withdrawal-status-badge]")
              .should("be.visible")
              .invoke("text")
              .should("eq", "CONFIRMED")
            cy.takeScreenshot("19_withdrawal_approved_status")
          })
      })
    })
  })
})
