import { z } from "zod";
import { tool } from "ai";
import { gql } from "@apollo/client";
import {
  getCreditFacilityDetails,
  GetCreditFacilityDetailsQueryVariables,
} from "@/lib/graphql/generated";
import { centsToUSD, satoshisToBTC, toPercentage } from "@/lib/utils/currency";

gql`
  query GetCreditFacilityDetails($id: UUID!) {
    creditFacility(id: $id) {
      creditFacilityId
      activatedAt
      expiresAt
      createdAt
      collateralizationState
      facilityAmount
      collateral
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
      status
      currentCvl {
        total
        disbursed
      }
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
      customer {
        email
      }
    }
  }
`;

export const getCreditFacilityDetailsTool = tool({
  type: "function",
  description: `Retrieve comprehensive details for a single credit facility. USE ONLY: when complete facility details are asked.`,
  parameters: z.object({
    id: z
      .string()
      .uuid()
      .describe(
        "UUID of the credit facility to fetch detailed information for"
      ),
  }),
  execute: async ({ id }) => {
    const response = await getCreditFacilityDetails({ id });

    if ("error" in response) {
      return response;
    }

    const facility = response.data.creditFacility;
    if (!facility) return response;

    return {
      ...response,
      data: {
        creditFacility: {
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
        },
      },
    };
  },
});
