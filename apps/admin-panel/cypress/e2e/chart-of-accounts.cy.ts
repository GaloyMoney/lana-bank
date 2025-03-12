describe("Chart Of Accounts", () => {
  it("should upload CSV and display chart data", () => {
    cy.visit("/chart-of-accounts")
    cy.get('[data-testid="loading-skeleton"]').should("not.exist")

    cy.get("body").then(($body) => {
      const hasUploadButton = $body.find('button:contains("Upload")').length > 0
      const hasDropzoneText = $body.find(':contains("Drag and drop")').length > 0

      cy.takeScreenshot("1_chart_of_account_upload")
      if (hasUploadButton || hasDropzoneText) {
        cy.get('input[type="file"]').attachFile("coa.csv", { force: true })
        cy.contains("button", /upload/i, { timeout: 5000 }).click()
      }
    })

    cy.get("body")
      .contains(/Assets/i)
      .should("be.visible")
    cy.get("body")
      .contains(/Liabilities/i)
      .should("be.visible")
    cy.get("body")
      .contains(/Equity/i)
      .should("be.visible")
    cy.get("body")
      .contains(/Revenue/i)
      .should("be.visible")
    cy.get("body")
      .contains(/Expenses/i)
      .should("be.visible")

    cy.get("body")
      .contains(/Current Assets/i)
      .should("be.visible")
    cy.get("body")
      .contains(/Non-Current Assets/i)
      .should("be.visible")

    cy.takeScreenshot("2_chart_of_account_view")
  })
})
