import { t } from "../support/translation"

describe("Customers", () => {
  let testEmail: string
  let testTelegramId: string
  let testCustomerId: string
  let testCustomerPublicId: string

  it("should successfully create a new customer", () => {
    testEmail = `t${Date.now().toString().slice(-6)}@example.com`
    testTelegramId = `t${Date.now().toString().slice(-6)}`

    cy.visit("/customers")
    cy.takeScreenshot("2_list_all_customers")

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("3_click_create_button")

    cy.get('[data-testid="customer-create-email"]').should("be.visible")
    cy.takeScreenshot("4_verify_email_input_visible")

    cy.get('[data-testid="customer-create-email"]')
      .should("be.visible")
      .should("be.enabled")
      .clear()
      .type(testEmail)
      .should("have.value", testEmail)
    cy.takeScreenshot("5_enter_email")

    cy.get('[data-testid="customer-create-telegram-id"]')
      .should("be.visible")
      .should("be.enabled")
      .clear()
      .type(testTelegramId)
      .should("have.value", testTelegramId)
    cy.takeScreenshot("6_enter_telegram_id")

    cy.get('[data-testid="customer-create-submit-button"]')
      .contains(t("Customers.create.reviewButton"))
      .click()
    cy.takeScreenshot("7_click_review_details")

    cy.contains(testEmail).should("be.visible")
    cy.contains(testTelegramId).should("be.visible")
    cy.takeScreenshot("8_verify_details")

    cy.get('[data-testid="customer-create-submit-button"]')
      .contains(t("Customers.create.confirmButton"))
      .click()
    cy.takeScreenshot("9_click_confirm_submit")

    cy.url().should("match", /\/customers\/[0-9]+$/)
    cy.contains(testEmail).should("be.visible")
    cy.contains(t("Customers.create.title")).should("not.exist")
    cy.takeScreenshot("10_verify_email")
    cy.getIdFromUrl("/customers/").then((id) => {
      testCustomerPublicId = id
    })
    cy.graphqlRequest<{ data: { customerByPublicId: { customerId: string } } }>(
      `query CustomerByPublicId($id: PublicId!) { customerByPublicId(id: $id) { customerId } }`,
      { id: testCustomerPublicId },
    ).then((res) => {
      testCustomerId = res.data.customerByPublicId.customerId
    })
  })

  it("should show newly created customer in the list", () => {
    cy.visit("/customers")
    cy.contains(testEmail).should("be.visible")
    cy.takeScreenshot("11_verify_customer_in_list")
  })

  it("should upload a document", function () {
    if (!Cypress.env("GOOGLE_CLOUD_AVAILABLE")) {
      this.skip()
    }
    cy.visit(`/customers/${testCustomerPublicId}/documents`)
    cy.contains(t("Customers.CustomerDetails.Documents.description")).should("exist")
    cy.takeScreenshot("12_customer_documents")
    cy.fixture("test.pdf", "binary").then((content) => {
      cy.get('input[type="file"]').attachFile({
        fileContent: content,
        fileName: "test.pdf",
        mimeType: "application/pdf",
      })
    })
    cy.contains(t("Customers.CustomerDetails.Documents.messages.uploadSuccess")).should(
      "exist",
    )
    cy.takeScreenshot("13_upload_document")
  })

  it("KYC verification", () => {
    cy.intercept("POST", "/graphql", (req) => {
      if (req.body.operationName === "sumsubPermalinkCreate") {
        req.reply({
          statusCode: 200,
          headers: {
            "content-type": "application/json",
          },
          body: {
            data: {
              sumsubPermalinkCreate: {
                url: "https://in.sumsub.com/test/link",
                __typename: "SumsubPermalinkCreatePayload",
              },
            },
          },
        })
      }
    }).as("sumsubPermalink")

    cy.visit(`/customers/${testCustomerPublicId}`)
    cy.takeScreenshot("14_customer_kyc_details_page")

    cy.get('[data-testid="customer-create-kyc-link"]').click()
    cy.contains("https://in.sumsub.com/test/link")
      .should("be.visible")
      .and("have.attr", "href", "https://in.sumsub.com/test/link")
    cy.takeScreenshot("15_kyc_link_created")

    cy.request({
      method: "POST",
      url: "http://localhost:5253/sumsub/callback",
      headers: {
        "Content-Type": "application/json",
      },
      body: {
        applicantId: "5cb56e8e0a975a35f333cb83",
        inspectionId: "5cb56e8e0a975a35f333cb84",
        correlationId: "req-a260b669-4f14-4bb5-a4c5-ac0218acb9a4",
        externalUserId: testCustomerId,
        levelName: "basic-kyc-level",
        type: "applicantReviewed",
        reviewResult: {
          reviewAnswer: "GREEN",
        },
        reviewStatus: "completed",
        createdAtMs: "2020-02-21 13:23:19.321",
      },
    }).then((response) => {
      expect(response.status).to.eq(200)
    })

    cy.reload()
    cy.contains("Basic").should("be.visible")
    cy.takeScreenshot("16_kyc_status_updated")
  })
})
