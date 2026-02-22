import { t } from "../support/translation"

describe("Customers", () => {
  let testEmail: string
  let testTelegramHandle: string
  let testCustomerId: string
  let testCustomerPublicId: string
  let testProspectPublicId: string

  it("should successfully create a new prospect", () => {
    testEmail = `t${Date.now().toString().slice(-6)}@example.com`
    testTelegramHandle = `t${Date.now().toString().slice(-6)}`

    cy.visit("/prospects")
    cy.takeScreenshot("2_list_all_prospects")

    cy.get('[data-testid="global-create-button"]').click()
    cy.takeScreenshot("3_click_create_button")

    cy.get('[data-testid="prospect-create-email"]').should("be.visible")
    cy.takeScreenshot("4_verify_email_input_visible")

    cy.get('[data-testid="prospect-create-email"]')
      .should("be.visible")
      .should("be.enabled")
      .clear()
      .type(testEmail)
      .should("have.value", testEmail)
    cy.takeScreenshot("5_enter_email")

    cy.get('[data-testid="prospect-create-telegram-handle"]')
      .should("be.visible")
      .should("be.enabled")
      .clear()
      .type(testTelegramHandle)
      .should("have.value", testTelegramHandle)
    cy.takeScreenshot("6_enter_telegram_handle")

    cy.get('[data-testid="prospect-create-submit-button"]')
      .contains(t("Prospects.create.reviewButton"))
      .click()
    cy.takeScreenshot("7_click_review_details")

    cy.contains(testEmail).should("be.visible")
    cy.contains(testTelegramHandle).should("be.visible")
    cy.takeScreenshot("8_verify_details")

    cy.get('[data-testid="prospect-create-submit-button"]')
      .contains(t("Prospects.create.confirmButton"))
      .click()
    cy.takeScreenshot("9_click_confirm_submit")

    cy.url().should("match", /\/prospects\/[0-9]+$/)
    cy.contains(testEmail).should("be.visible")
    cy.takeScreenshot("10_verify_email")
    cy.getIdFromUrl("/prospects/")
      .then((id) => {
        testProspectPublicId = id
      })
      .then(() => {
        cy.graphqlRequest<{ data: { prospectByPublicId: { prospectId: string } } }>(
          `query ProspectByPublicId($id: PublicId!) { prospectByPublicId(id: $id) { prospectId } }`,
          { id: testProspectPublicId },
        ).then((res) => {
          testCustomerId = res.data.prospectByPublicId.prospectId
        })
      })
  })

  it("KYC verification and customer creation", () => {
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

    cy.visit(`/prospects/${testProspectPublicId}`)
    cy.takeScreenshot("14_prospect_kyc_details_page")

    cy.get('[data-testid="prospect-create-kyc-link"]').click()
    cy.contains("https://in.sumsub.com/test/link")
      .should("be.visible")
      .and("have.attr", "href", "https://in.sumsub.com/test/link")
    cy.takeScreenshot("15_kyc_link_created")

    const webhookId = `req-${Date.now()}`
    const applicantId = `test-applicant-${webhookId}`

    // Simulate KYC start via SumSub applicantCreated webhook
    cy.request({
      method: "POST",
      url: "http://localhost:5253/webhook/sumsub",
      headers: {
        "Content-Type": "application/json",
      },
      body: {
        applicantId,
        inspectionId: `test-inspection-${webhookId}`,
        correlationId: webhookId,
        externalUserId: testCustomerId,
        levelName: "basic-kyc-level",
        type: "applicantCreated",
        reviewStatus: "init",
        createdAtMs: new Date().toISOString(),
        sandboxMode: true,
      },
    }).then((response) => {
      expect(response.status).to.eq(200)
    })

    // Simulate KYC approval via SumSub applicantReviewed webhook
    cy.request({
      method: "POST",
      url: "http://localhost:5253/webhook/sumsub",
      headers: {
        "Content-Type": "application/json",
      },
      body: {
        applicantId,
        inspectionId: `test-inspection-${webhookId}`,
        correlationId: webhookId,
        externalUserId: testCustomerId,
        levelName: "basic-kyc-level",
        type: "applicantReviewed",
        reviewResult: {
          reviewAnswer: "GREEN",
        },
        reviewStatus: "completed",
        createdAtMs: new Date().toISOString(),
        sandboxMode: true,
      },
    }).then((response) => {
      expect(response.status).to.eq(200)
    })

    // Convert prospect to customer synchronously via prospectConvert mutation.
    // The webhooks above test the webhook endpoint; prospectConvert ensures the
    // customer exists without depending on async inbox job processing.
    cy.graphqlRequest<{
      data: { prospectConvert: { customer: { customerId: string; publicId: string } } }
    }>(
      `mutation ProspectConvert($input: ProspectConvertInput!) {
        prospectConvert(input: $input) {
          customer { customerId publicId }
        }
      }`,
      { input: { prospectId: testCustomerId } },
    ).then((res) => {
      testCustomerPublicId = res.data.prospectConvert.customer.publicId
    })
  })

  it("should show newly created customer in the list", () => {
    cy.visit("/customers")
    cy.contains(testEmail).should("be.visible")
    cy.takeScreenshot("11_verify_customer_in_list")
  })

  it("should have a deposit account for the customer", () => {
    // Query backend to determine if deposit account already exists
    cy.graphqlRequest<{
      data: {
        customerByPublicId: { depositAccount: { depositAccountId: string } | null }
      }
    }>(
      `query CheckDepositAccount($id: PublicId!) {
        customerByPublicId(id: $id) { depositAccount { depositAccountId } }
      }`,
      { id: testCustomerPublicId },
    ).then((res) => {
      if (!res.data.customerByPublicId.depositAccount) {
        // No deposit account — create via GraphQL to avoid UI race conditions
        cy.graphqlRequest<{
          data: {
            customerByPublicId: { customerId: string }
          }
        }>(
          `query GetCustomerId($id: PublicId!) {
            customerByPublicId(id: $id) { customerId }
          }`,
          { id: testCustomerPublicId },
        ).then((customerRes) => {
          cy.createDepositAccount(
            customerRes.data.customerByPublicId.customerId,
          )
        })
      }
    })

    // Verify deposit account is visible on the page
    cy.visit(`/customers/${testCustomerPublicId}`)
    cy.contains(t("Customers.CustomerDetails.depositAccount.title")).should(
      "be.visible",
    )
    cy.takeScreenshot("customer_deposit_account_created")
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
})
