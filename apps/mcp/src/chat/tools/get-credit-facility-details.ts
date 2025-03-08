import { z } from "zod";
// Use CommonJS compatible import pattern for @apollo/client
import { createRequire } from "module";
const require = createRequire(import.meta.url);
const { gql } = require("@apollo/client");
// Import from our operations module instead
import { getCreditFacilityDetails } from "../../lib/graphql/operations.js";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";

// Handle missing currency utilities
const centsToUSD = (cents: number): string => `$${(cents / 100).toFixed(2)}`;
const satoshisToBTC = (satoshis: number): string =>
  `${(satoshis / 100000000).toFixed(8)} BTC`;
const toPercentage = (value: number): string => `${(value * 100).toFixed(2)}%`;

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

export function registerCreditFacilityDetailsTool(server: McpServer) {
  server.tool(
    "get-credit-facility-details",
    "Retrieve comprehensive details for a single credit facility. USE ONLY: when complete facility details are asked.",
    {
      id: z
        .string()
        .uuid()
        .describe(
          "UUID of the credit facility to fetch detailed information for"
        ),
    },
    async ({ id }: { id: string }) => {
      try {
        // This is a placeholder in case getCreditFacilityDetails is not available
        const getCreditFacilityDetailsFunc =
          getCreditFacilityDetails ||
          (async (params: any) => {
            console.error(
              "getCreditFacilityDetails function not available, returning mock data"
            );
            return {
              data: {
                creditFacility: null,
              },
            };
          });

        const response = await getCreditFacilityDetailsFunc({ id });

        if ("error" in response) {
          return {
            content: [
              {
                type: "text",
                text: `Error: ${response.error}`,
              },
            ],
          };
        }

        const facility = response.data.creditFacility;
        if (!facility) {
          return {
            content: [
              {
                type: "text",
                text: `No credit facility found with ID: ${id}`,
              },
            ],
          };
        }

        const formattedFacility = {
          ...facility,
          facilityAmountUSD: centsToUSD(facility.facilityAmount),
          collateralBTC: satoshisToBTC(facility.collateral),
          creditFacilityTerms: {
            ...facility.creditFacilityTerms,
            annualRate: toPercentage(facility.creditFacilityTerms.annualRate),
            oneTimeFeeRate: toPercentage(
              facility.creditFacilityTerms.oneTimeFeeRate
            ),
            liquidationCvl: toPercentage(
              facility.creditFacilityTerms.liquidationCvl
            ),
            marginCallCvl: toPercentage(
              facility.creditFacilityTerms.marginCallCvl
            ),
            initialCvl: toPercentage(facility.creditFacilityTerms.initialCvl),
          },
          currentCvl: {
            total: toPercentage(facility.currentCvl.total),
            disbursed: toPercentage(facility.currentCvl.disbursed),
          },
          transactions: facility.transactions.map((tx: any) => {
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
          disbursals: facility.disbursals.map((d: any) => ({
            ...d,
            amountUSD: centsToUSD(d.amount),
          })),
          balance: {
            facilityRemaining: {
              usdBalance: centsToUSD(
                facility.balance.facilityRemaining.usdBalance
              ),
            },
            disbursed: {
              total: {
                usdBalance: centsToUSD(
                  facility.balance.disbursed.total.usdBalance
                ),
              },
              outstanding: {
                usdBalance: centsToUSD(
                  facility.balance.disbursed.outstanding.usdBalance
                ),
              },
              dueOutstanding: {
                usdBalance: centsToUSD(
                  facility.balance.disbursed.dueOutstanding.usdBalance
                ),
              },
            },
            interest: {
              total: {
                usdBalance: centsToUSD(
                  facility.balance.interest.total.usdBalance
                ),
              },
              outstanding: {
                usdBalance: centsToUSD(
                  facility.balance.interest.outstanding.usdBalance
                ),
              },
              dueOutstanding: {
                usdBalance: centsToUSD(
                  facility.balance.interest.dueOutstanding.usdBalance
                ),
              },
            },
            outstanding: {
              usdBalance: centsToUSD(facility.balance.outstanding.usdBalance),
            },
            dueOutstanding: {
              usdBalance: centsToUSD(
                facility.balance.dueOutstanding.usdBalance
              ),
            },
            collateral: {
              btcBalance: satoshisToBTC(facility.balance.collateral.btcBalance),
            },
          },
        };

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(formattedFacility, null, 2),
            },
          ],
        };
      } catch (error) {
        console.error("Error in get-credit-facility-details tool:", error);
        return {
          content: [
            {
              type: "text",
              text: `Error processing request: ${error instanceof Error ? error.message : String(error)}`,
            },
          ],
        };
      }
    }
  );
}
