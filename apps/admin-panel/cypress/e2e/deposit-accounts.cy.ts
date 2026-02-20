import { t } from "../support/translation"

const DA_CARD = "DepositAccounts.DepositAccountDetails.DepositAccountDetailsCard"
const DA_TRANS = "DepositAccounts.DepositAccountDetails.transactions"
const DA_FREEZE = "DepositAccounts.DepositAccountDetails.freezeDepositAccount"
const DA_UNFREEZE = "DepositAccounts.DepositAccountDetails.unfreezeDepositAccount"
const DA_CLOSE = "DepositAccounts.DepositAccountDetails.closeDepositAccount"
const DA_STATUS = "DepositAccounts.status"

describe("Deposit Accounts", () => {
  let testDepositAccountId: string
  let testDepositAccountPublicId: string

  before(() => {
    const testEmail = `deposit${Date.now().toString().slice(-6)}@example.com`
    const testTelegramId = `deposit${Date.now().toString().slice(-6)}`

    cy.createCustomer(testEmail, testTelegramId).then((customer) => {
      testDepositAccountId = customer.depositAccount.depositAccountId
      testDepositAccountPublicId = customer.depositAccount.publicId
      cy.log(`Created deposit account with public ID: ${testDepositAccountPublicId}`)
    })
  })

  it("should display deposit account details correctly", () => {
    cy.visit(`/deposit-accounts/${testDepositAccountPublicId}`)
    cy.contains(t(DA_CARD + ".fields.customerId")).should("be.visible")
    cy.contains(t(DA_CARD + ".fields.settledBalance")).should("be.visible")
    cy.contains(t(DA_CARD + ".fields.pendingBalance")).should("be.visible")
    cy.get('[data-testid="deposit-account-status-badge"]').should("be.visible")
    cy.contains(t(DA_CARD + ".buttons.viewLedgerAccount")).should("be.visible")
    cy.contains(t(DA_CARD + ".buttons.freezeDepositAccount")).should("be.visible")
  })

  it("should show deposit account in list page", () => {
    cy.visit("/deposit-accounts")
    cy.contains(testDepositAccountPublicId).should("be.visible")
    cy.contains(testDepositAccountPublicId)
      .parents("tr")
      .within(() => {
        cy.contains(t("PaginatedTable.view")).click()
      })
    cy.url().should("include", `/deposit-accounts/${testDepositAccountPublicId}`)
  })

  it("should display transactions table with deposit", () => {
    cy.createDeposit(10000, testDepositAccountId)
    cy.visit(`/deposit-accounts/${testDepositAccountPublicId}`)
    cy.contains(t(DA_TRANS + ".title")).should("be.visible")

    cy.contains(t(DA_TRANS + ".table.headers.date"))
    cy.contains(t(DA_TRANS + ".table.headers.type"))
    cy.contains(t(DA_TRANS + ".table.headers.amount"))
    cy.contains(t(DA_TRANS + ".table.headers.status"))
    cy.contains(t(DA_TRANS + ".table.types.deposit"))
  })

  it("should freeze a deposit account", () => {
    cy.visit(`/deposit-accounts/${testDepositAccountPublicId}`)
    cy.contains(t(DA_CARD + ".buttons.freezeDepositAccount")).click()
    cy.contains(t(DA_FREEZE + ".title")).should("be.visible")
    cy.contains(t(DA_FREEZE + ".fields.settledBalance")).should("be.visible")
    cy.contains(t(DA_FREEZE + ".fields.pendingBalance")).should("be.visible")
    cy.get('[data-testid="freeze-deposit-account-dialog-button"]').click()
    cy.wait(1000)
    cy.get('[data-testid="deposit-account-status-badge"]', { timeout: 10000 })
      .invoke("text")
      .should("eq", t(DA_STATUS + ".frozen"))
    cy.contains(t(DA_CARD + ".buttons.unfreezeDepositAccount")).should("be.visible")
  })

  it("should unfreeze a deposit account", () => {
    cy.visit(`/deposit-accounts/${testDepositAccountPublicId}`)
    cy.get('[data-testid="deposit-account-status-badge"]')
      .invoke("text")
      .should("eq", t(DA_STATUS + ".frozen"))
    cy.contains(t(DA_CARD + ".buttons.unfreezeDepositAccount")).click()
    cy.contains(t(DA_UNFREEZE + ".title")).should("be.visible")
    cy.get('[data-testid="unfreeze-deposit-account-dialog-button"]').click()
    cy.wait(1000)
    cy.get('[data-testid="deposit-account-status-badge"]', { timeout: 10000 })
      .invoke("text")
      .should("eq", t(DA_STATUS + ".active"))
    cy.contains(t(DA_CARD + ".buttons.freezeDepositAccount")).should("be.visible")
  })

  it("should allow closing account with zero balance", () => {
    const closeTestEmail = `close${Date.now().toString().slice(-6)}@example.com`
    const closeTestTelegramId = `close${Date.now().toString().slice(-6)}`
    cy.createCustomer(closeTestEmail, closeTestTelegramId).then((customer) => {
      const closeTestDepositAccountPublicId = customer.depositAccount.publicId
      cy.log(`Created deposit account for close test: ${closeTestDepositAccountPublicId}`)
      cy.visit(`/deposit-accounts/${closeTestDepositAccountPublicId}`)
      cy.contains(t(DA_CARD + ".buttons.closeDepositAccount")).should("not.be.disabled")
      cy.contains(t(DA_CARD + ".buttons.closeDepositAccount")).click()
      cy.contains(t(DA_CLOSE + ".title")).should("be.visible")
      cy.get('[data-testid="close-deposit-account-dialog-button"]').click()
      cy.wait(1000)
      cy.get('[data-testid="deposit-account-status-badge"]', { timeout: 10000 })
        .invoke("text")
        .should("eq", t(DA_STATUS + ".closed"))
    })
  })
})
