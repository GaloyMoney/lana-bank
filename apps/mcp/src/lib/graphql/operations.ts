import { createRequire } from "module";
import { graphqlClient } from "../client.js";

const require = createRequire(import.meta.url);
const { gql } = require("@apollo/client");

// Credit Facilities query
export const creditFacilitiesQuery = gql`
  query CreditFacilities(
    $first: Int!
    $after: String
    $sort: CreditFacilitiesSort
    $filter: CreditFacilitiesFilter
  ) {
    creditFacilities(
      first: $first
      after: $after
      sort: $sort
      filter: $filter
    ) {
      edges {
        cursor
        node {
          creditFacilityId
          collateralizationState
          createdAt
          status
          facilityAmount
          collateral
          currentCvl {
            disbursed
            total
          }
          balance {
            collateral {
              btcBalance
            }
            outstanding {
              usdBalance
            }
          }
          customer {
            customerId
            email
          }
        }
      }
      pageInfo {
        endCursor
        hasNextPage
      }
    }
  }
`;

// Credit Facility Details query
export const creditFacilityDetailsQuery = gql`
  query GetCreditFacilityDetails($id: UUID!) {
    creditFacility(id: $id) {
      creditFacilityId
      status
      collateralizationState
      createdAt
      facilityAmount
      collateral
      currentCvl {
        disbursed
        total
      }
      balance {
        collateral {
          btcBalance
        }
        outstanding {
          usdBalance
        }
        dueOutstanding {
          usdBalance
        }
        disbursed {
          outstanding {
            usdBalance
          }
          dueOutstanding {
            usdBalance
          }
          total {
            usdBalance
          }
        }
        interest {
          outstanding {
            usdBalance
          }
          dueOutstanding {
            usdBalance
          }
          total {
            usdBalance
          }
        }
        facilityRemaining {
          usdBalance
        }
      }
      creditFacilityTerms {
        annualRate
        oneTimeFeeRate
        marginCallCvl
        liquidationCvl
        initialCvl
        accrualInterval
        incurrenceInterval
        duration {
          period
          units
        }
      }
      repaymentPlan {
        repaymentType
        status
        initial
        outstanding
        accrualAt
        dueAt
      }
      transactions {
        __typename
        ... on CreditFacilityOrigination {
          recordedAt
          cents
          txId
        }
        ... on CreditFacilityDisbursalExecuted {
          recordedAt
          cents
          txId
        }
        ... on CreditFacilityInterestAccrued {
          recordedAt
          cents
          days
          txId
        }
        ... on CreditFacilityIncrementalPayment {
          recordedAt
          cents
          txId
        }
        ... on CreditFacilityCollateralUpdated {
          recordedAt
          action
          satoshis
          txId
        }
        ... on CreditFacilityCollateralizationUpdated {
          recordedAt
          state
          collateral
          outstandingDisbursal
          outstandingInterest
          price
        }
      }
      customer {
        customerId
        email
      }
    }
  }
`;

// Customer Credit Facility query
export const customerCreditFacilityQuery = gql`
  query GetCustomerCreditFacilityByEmail($email: String!) {
    customerByEmail(email: $email) {
      customerId
      email
      creditFacilities {
        creditFacilityId
        status
        collateralizationState
        createdAt
        facilityAmount
        collateral
        currentCvl {
          disbursed
          total
        }
        balance {
          collateral {
            btcBalance
          }
          outstanding {
            usdBalance
          }
          dueOutstanding {
            usdBalance
          }
          disbursed {
            outstanding {
              usdBalance
            }
            dueOutstanding {
              usdBalance
            }
            total {
              usdBalance
            }
          }
          interest {
            outstanding {
              usdBalance
            }
            dueOutstanding {
              usdBalance
            }
            total {
              usdBalance
            }
          }
          facilityRemaining {
            usdBalance
          }
        }
      }
    }
  }
`;

// Customer Details query
export const customerDetailsQuery = gql`
  query GetCustomerByEmail($email: String!) {
    customerByEmail(email: $email) {
      customerId
      email
      status
      level
      customerType
      createdAt
      applicantId
      telegramId
      depositAccount {
        depositAccountId
        balance {
          settled
          pending
        }
      }
    }
  }
`;

// Wrapper functions for executing the queries
export async function creditFacilities({ first, after, sort, filter }: any) {
  try {
    return await graphqlClient.query({
      query: creditFacilitiesQuery,
      variables: { first, after, sort, filter },
    });
  } catch (error) {
    console.error("Error executing creditFacilities query:", error);
    return { error: error instanceof Error ? error.message : String(error) };
  }
}

export async function getCreditFacilityDetails({ id }: { id: string }) {
  try {
    return await graphqlClient.query({
      query: creditFacilityDetailsQuery,
      variables: { id },
    });
  } catch (error) {
    console.error("Error executing getCreditFacilityDetails query:", error);
    return { error: error instanceof Error ? error.message : String(error) };
  }
}

export async function getCustomerCreditFacilityByEmail({
  email,
}: {
  email: string;
}) {
  try {
    return await graphqlClient.query({
      query: customerCreditFacilityQuery,
      variables: { email },
    });
  } catch (error) {
    console.error(
      "Error executing getCustomerCreditFacilityByEmail query:",
      error
    );
    return { error: error instanceof Error ? error.message : String(error) };
  }
}

export async function getCustomerByEmail({ email }: { email: string }) {
  try {
    return await graphqlClient.query({
      query: customerDetailsQuery,
      variables: { email },
    });
  } catch (error) {
    console.error("Error executing getCustomerByEmail query:", error);
    return { error: error instanceof Error ? error.message : String(error) };
  }
}
