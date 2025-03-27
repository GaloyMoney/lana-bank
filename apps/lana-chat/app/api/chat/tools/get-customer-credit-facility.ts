import { z } from "zod";
import { tool } from "ai";
import { gql } from "@apollo/client";
import {
  getCustomerCreditFacilityByEmail,
  GetCustomerCreditFacilityByEmailQueryVariables,
} from "@/lib/graphql/generated";
import { centsToUSD, satoshisToBTC, toPercentage } from "@/lib/utils/currency";

gql`
  query GetCustomerCreditFacilityByEmail($email: String!) {
    customerByEmail(email: $email) {
      customerId
      creditFacilities {
        creditFacilityId
        activatedAt
        expiresAt
        createdAt
        collateralizationState
        facilityAmount
        collateral
        canBeCompleted
        status
        transactions {
          ... on CreditFacilityIncrementalPayment {
            cents
            recordedAt
            txId
          }
          ... on CreditFacilityCollateralUpdated {
            satoshis
            recordedAt
            action
            txId
          }
          ... on CreditFacilityOrigination {
            cents
            recordedAt
            txId
          }
          ... on CreditFacilityCollateralizationUpdated {
            state
            collateral
            outstandingInterest
            outstandingDisbursal
            recordedAt
            price
          }
          ... on CreditFacilityDisbursalExecuted {
            cents
            recordedAt
            txId
          }
          ... on CreditFacilityInterestAccrued {
            cents
            recordedAt
            txId
            days
          }
        }
        disbursals {
          disbursalId
          amount
          createdAt
          status
        }
        currentCvl {
          total
          disbursed
        }
        creditFacilityTerms {
          annualRate
          accrualInterval
          incurrenceInterval
          oneTimeFeeRate
          liquidationCvl
          marginCallCvl
          initialCvl
          duration {
            period
            units
          }
        }
        balance {
          facilityRemaining {
            usdBalance
          }
          disbursed {
            total {
              usdBalance
            }
            outstanding {
              usdBalance
            }
            dueOutstanding {
              usdBalance
            }
          }
          interest {
            total {
              usdBalance
            }
            outstanding {
              usdBalance
            }
            dueOutstanding {
              usdBalance
            }
          }
          outstanding {
            usdBalance
          }
          dueOutstanding {
            usdBalance
          }
          collateral {
            btcBalance
          }
        }
      }
    }
  }
`;

export const getCustomerCreditFacilitiesTool = tool({
  type: "function",
  description:
    "Retrieve details about the customer's credit facility, including the facility amount, transactions, balance information, collateral, and associated terms. Use this tool to get credit-specific information about the customer.",
  parameters: z.object({
    email: z
      .string()
      .describe(
        "The email address of the customer whose credit facility details are being requested."
      ),
  }),
  execute: async ({ email }) => {
    const response = await getCustomerCreditFacilityByEmail({ email });

    if ("error" in response) {
      return response;
    }

    const customer = response.data.customerByEmail;
    if (!customer) return response;

    return {
      ...response,
      data: {
        customerByEmail: {
          ...customer,
          creditFacilities: customer.creditFacilities.map((facility) => ({
            ...facility,
            facilityAmountUSD: centsToUSD(facility.facilityAmount),
            collateralBTC: satoshisToBTC(facility.collateral),
            creditFacilityTerms: {
              ...facility.creditFacilityTerms,
              annualRate: toPercentage(facility.creditFacilityTerms.annualRate),
              oneTimeFeeRate: toPercentage(facility.creditFacilityTerms.oneTimeFeeRate),
              liquidationCvl: toPercentage(facility.creditFacilityTerms.liquidationCvl),
              marginCallCvl: toPercentage(facility.creditFacilityTerms.marginCallCvl),
              initialCvl: toPercentage(facility.creditFacilityTerms.initialCvl),
            },
            currentCvl: {
              total: toPercentage(facility.currentCvl.total),
              disbursed: toPercentage(facility.currentCvl.disbursed),
            },
            transactions: facility.transactions.map((tx) => {
              if ("cents" in tx) {
                return { ...tx, amountUSD: centsToUSD(tx.cents) };
              }
              if ("satoshis" in tx) {
                return { ...tx, amountBTC: satoshisToBTC(tx.satoshis) };
              }
              if ("outstandingInterest" in tx) {
                return {
                  ...tx,
                  collateralBTC: satoshisToBTC(tx.collateral),
                  outstandingInterestUSD: centsToUSD(tx.outstandingInterest),
                  outstandingDisbursalUSD: centsToUSD(tx.outstandingDisbursal),
                  priceUSD: centsToUSD(tx.price),
                };
              }
              return tx;
            }),
            disbursals: facility.disbursals.map((d) => ({
              ...d,
              amountUSD: centsToUSD(d.amount),
            })),
            balance: {
              facilityRemaining: {
                usdBalance: centsToUSD(facility.balance.facilityRemaining.usdBalance),
              },
              disbursed: {
                total: {
                  usdBalance: centsToUSD(facility.balance.disbursed.total.usdBalance),
                },
                outstanding: {
                  usdBalance: centsToUSD(facility.balance.disbursed.outstanding.usdBalance),
                },
                dueOutstanding: {
                  usdBalance: centsToUSD(facility.balance.disbursed.dueOutstanding.usdBalance),
                },
              },
              interest: {
                total: {
                  usdBalance: centsToUSD(facility.balance.interest.total.usdBalance),
                },
                outstanding: {
                  usdBalance: centsToUSD(facility.balance.interest.outstanding.usdBalance),
                },
                dueOutstanding: {
                  usdBalance: centsToUSD(facility.balance.interest.dueOutstanding.usdBalance),
                },
              },
              outstanding: {
                usdBalance: centsToUSD(facility.balance.outstanding.usdBalance),
              },
              dueOutstanding: {
                usdBalance: centsToUSD(facility.balance.dueOutstanding.usdBalance),
              },
              collateral: {
                btcBalance: satoshisToBTC(facility.balance.collateral.btcBalance),
              },
            },
          })),
        },
      },
    };
  },
});
