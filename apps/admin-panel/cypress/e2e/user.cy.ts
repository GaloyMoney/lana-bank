describe("Users", () => {
  let userEmail: string
  let userId: string

  beforeEach(() => {
    cy.on("uncaught:exception", (err) => {
      if (err.message.includes("ResizeObserver loop")) {
        return false
      }
    })
  })

  it("should create a user successfully", () => {
    userEmail = `t${Date.now().toString().slice(-6)}@example.com`

    cy.visit(`/users`)
    cy.wait(1000)
    cy.takeScreenshot("1_users_list")

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("2_click_create_button")

    cy.get('[data-testid="create-user-email-input"]')
      .type(userEmail, { delay: 0, waitForAnimations: false })
      .should("have.value", userEmail)
    cy.takeScreenshot("3_enter_email")

    cy.get('[data-testid="create-user-role-admin-checkbox"]').click()
    cy.takeScreenshot("4_assign_admin_role")

    cy.get('[data-testid="create-user-submit-button"]').click()
    cy.takeScreenshot("5_submit_creation")

    cy.url()
      .should("include", "/users/")
      .then((url) => {
        userId = url.split("/users/")[1]
        cy.takeScreenshot("6_verify_creation")
      })

    cy.get("[data-testid=user-details-email]")
      .should("be.visible")
      .invoke("text")
      .should("eq", userEmail)
  })

  it("should show newly created user in the list", () => {
    cy.visit("/users")
    cy.wait(1000)
    cy.contains(userEmail).should("be.visible")
    cy.takeScreenshot("7_view_in_list")
  })

  it("Can update user roles", () => {
    cy.visit(`/users/${userId}`)
    cy.wait(1000)
    cy.takeScreenshot("8_manage_roles")

    cy.get('[data-testid="user-details-manage-role"]').click()
    cy.takeScreenshot("9_update_roles")
    cy.get('[data-testid="user-details-manage-role-accountant-checkbox"]').click()

    cy.contains("Role assigned").should("be.visible")
    cy.takeScreenshot("10_verify_update")
  })
})
