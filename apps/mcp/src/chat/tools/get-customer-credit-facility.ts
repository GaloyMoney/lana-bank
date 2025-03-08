import { z } from "zod";
// Use CommonJS compatible import pattern for @apollo/client
import { createRequire } from "module";
const require = createRequire(import.meta.url);
const { gql } = require("@apollo/client");
// Import from our operations module instead
import { getCustomerCreditFacilityByEmail } from "../../lib/graphql/operations.js";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";

// Handle missing currency utilities
const centsToUSD = (cents: number): string => `$${(cents / 100).toFixed(2)}`;
const satoshisToBTC = (satoshis: number): string =>
  `${(satoshis / 100000000).toFixed(8)} BTC`;
const toPercentage = (value: number): string => `${(value * 100).toFixed(2)}%`;

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

export function registerCustomerCreditFacilitiesTool(server: McpServer) {
  server.tool(
    "get-customer-credit-facilities",
    "Retrieve details about the customer's credit facility, including the facility amount, transactions, balance information, collateral, and associated terms. Use this tool to get credit-specific information about the customer.",
    {
      email: z
        .string()
        .describe(
          "The email address of the customer whose credit facility details are being requested."
        ),
    },
    async ({ email }: { email: string }) => {
      try {
        // This is a placeholder in case getCustomerCreditFacilityByEmail is not available
        const getCustomerCreditFacilityByEmailFunc =
          getCustomerCreditFacilityByEmail ||
          (async (params: any) => {
            console.error(
              "getCustomerCreditFacilityByEmail function not available, returning mock data"
            );
            return {
              data: {
                customerByEmail: null,
              },
            };
          });

        const response = await getCustomerCreditFacilityByEmailFunc({ email });

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

        const customer = response.data.customerByEmail;
        if (!customer) {
          return {
            content: [
              {
                type: "text",
                text: `No customer found with email: ${email}`,
              },
            ],
          };
        }

        const formattedCustomer = {
          ...customer,
          creditFacilities: customer.creditFacilities.map((facility: any) => ({
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
                btcBalance: satoshisToBTC(
                  facility.balance.collateral.btcBalance
                ),
              },
            },
          })),
        };

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(formattedCustomer, null, 2),
            },
          ],
        };
      } catch (error) {
        console.error("Error in get-customer-credit-facilities tool:", error);
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
