import type { MockedResponse } from "@apollo/client/testing"

import {
  GetCustomerBasicDetailsDocument,
  GetCustomerCreditFacilitiesDocument,
  GetCustomerDocumentsDocument,
} from "@/lib/graphql/generated"
import {
  mockCreditFacility,
  mockCustomer,
  mockCustomerDocument,
} from "@/lib/graphql/generated/mocks"

const CUSTOMER_PUBLIC_ID = "CUS-001"

const requestFor = (query: MockedResponse["request"]["query"]) => ({
  query,
  variables: { id: CUSTOMER_PUBLIC_ID },
})

const createMock = (
  query: MockedResponse["request"]["query"],
  data: Record<string, unknown>,
): MockedResponse => ({
  request: requestFor(query),
  result: { data },
  newData: () => ({ data }),
})

const createLoadingMock = (
  query: MockedResponse["request"]["query"],
): MockedResponse => ({
  request: requestFor(query),
  delay: Infinity,
})

const baseCustomer = mockCustomer()
const creditFacilities = [mockCreditFacility(), mockCreditFacility()]
const documents = [mockCustomerDocument(), mockCustomerDocument()]

export const buildParams = () =>
  Promise.resolve({
    "customer-id": CUSTOMER_PUBLIC_ID,
  })

export const customerDetailsMock = createMock(GetCustomerBasicDetailsDocument, {
  customerByPublicId: baseCustomer,
})

export const customerDetailsLoadingMock = createLoadingMock(
  GetCustomerBasicDetailsDocument,
)

export const creditFacilitiesMock = createMock(GetCustomerCreditFacilitiesDocument, {
  customerByPublicId: {
    __typename: "Customer",
    creditFacilities,
  },
})

export const emptyCreditFacilitiesMock = createMock(GetCustomerCreditFacilitiesDocument, {
  customerByPublicId: {
    __typename: "Customer",
    creditFacilities: [],
  },
})

export const customerDocumentsMock = createMock(GetCustomerDocumentsDocument, {
  customerByPublicId: {
    __typename: "Customer",
    documents,
  },
})

export const emptyCustomerDocumentsMock = createMock(GetCustomerDocumentsDocument, {
  customerByPublicId: {
    __typename: "Customer",
    documents: [],
  },
})
