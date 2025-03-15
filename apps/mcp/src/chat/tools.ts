import { z } from "zod";
// Use CommonJS compatible import pattern for @apollo/client
import { createRequire } from "module";
const require = createRequire(import.meta.url);
const { gql } = require("@apollo/client");
// Import from our operations module instead
import { 
  getCustomerByEmail, 
  creditFacilities, 
  getCreditFacilityDetails, 
  getCustomerCreditFacilityByEmail 
} from "../lib/graphql/operations.js";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";

// ----- Utility Functions -----

// Handle missing currency utilities
const centsToUSD = (cents: number): string => `$${(cents / 100).toFixed(2)}`;
const satoshisToBTC = (satoshis: number): string =>
  `${(satoshis / 100000000).toFixed(8)} BTC`;
const toPercentage = (value: number): string => `${(value * 100).toFixed(2)}%`;

// ----- Enum Values -----

// Define enum values directly to avoid dependency issues
const CollateralizationState = {
  NORMAL: "NORMAL",
  UNDER_COLLATERALIZED: "UNDER_COLLATERALIZED",
  SEVERELY_UNDER_COLLATERALIZED: "SEVERELY_UNDER_COLLATERALIZED",
  LIQUIDATION: "LIQUIDATION",
};

const CreditFacilityStatus = {
  ACTIVE: "ACTIVE",
  PENDING: "PENDING",
  CLOSED: "CLOSED",
  LIQUIDATED: "LIQUIDATED",
};

const CreditFacilitiesFilterBy = {
  STATUS: "STATUS",
  COLLATERALIZATION_STATE: "COLLATERALIZATION_STATE",
};

const CreditFacilitiesSortBy = {
  CREATED_AT: "CREATED_AT",
  FACILITY_AMOUNT: "FACILITY_AMOUNT",
};

const SortDirection = {
  ASC: "ASC",
  DESC: "DESC",
};

// ----- Schema Definitions -----

const CollateralizationStateSchema = z
  .nativeEnum(CollateralizationState as any)
  .describe(`Collateralization states`);

const CreditFacilityStatusSchema = z
  .nativeEnum(CreditFacilityStatus as any)
  .describe(`Credit facility status values`);

// ----- GraphQL Queries -----

// Customer Details Query
gql`
  query GetCustomerByEmail($email: String!) {
    customerByEmail(email: $email) {
      customerId
      status
      level
      createdAt
      email
      telegramId
      applicantId
      depositAccount {
        depositAccountId
        balance {
          pending
          settled
        }
        withdrawals {
          amount
          createdAt
          reference
          status
          withdrawalId
        }
        deposits {
          amount
          createdAt
          reference
          depositId
        }
      }
      documents {
        documentId
        status
        filename
      }
    }
  }
`;

// Credit Facilities Query
gql`
  query CreditFacilities(
    $limit: Int
    $offset: Int
    $filterBy: CreditFacilitiesFilterBy
    $filterValue: String
    $sortBy: CreditFacilitiesSortBy
    $sortDirection: SortDirection
  ) {
    creditFacilities(
      limit: $limit
      offset: $offset
      filterBy: $filterBy
      filterValue: $filterValue
      sortBy: $sortBy
      sortDirection: $sortDirection
    ) {
      creditFacilityId
      customerId
      customer {
        email
      }
      status
      facilityAmount
      collateral
      collateralizationState
      currentCvl {
        total
        disbursed
      }
    }
  }
`;

// Credit Facility Details Query
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
          term
          recordedAt
          txId
        }
        ... on CreditFacilityExpiration {
          recordedAt
          txId
        }
        ... on CreditFacilityLiquidation {
          recordedAt
          reason
          txId
        }
        ... on CreditFacilityOneTimeFee {
          cents
          recordedAt
          txId
        }
        ... on CreditFacilityInterestIncurred {
          cents
          recordedAt
          txId
        }
      }
    }
  }
`;

// Customer Credit Facility Query
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
            term
            recordedAt
            txId
          }
          ... on CreditFacilityExpiration {
            recordedAt
            txId
          }
          ... on CreditFacilityLiquidation {
            recordedAt
            reason
            txId
          }
          ... on CreditFacilityOneTimeFee {
            cents
            recordedAt
            txId
          }
          ... on CreditFacilityInterestIncurred {
            cents
            recordedAt
            txId
          }
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
        currentCvl {
          total
          disbursed
        }
      }
    }
  }
`;

// ----- Tool Registration Functions -----

export function registerCustomerDetailsTool(server: McpServer) {
  server.tool(
    "get-customer-details",
    "Retrieve general information about a customer by their email address. This includes account balances, deposit history, withdrawal history, KYC document statuses, and basic customer information. **Note**: This does not include any details about the customer's credit facility.",
    {
      email: z
        .string()
        .describe(
          "The email address of the customer whose general details (excluding credit facility) are being requested."
        ),
    },
    async ({ email }: { email: string }) => {
      try {
        // This is a placeholder in case getCustomerByEmail is not available
        const getCustomerByEmailFunc =
          getCustomerByEmail ||
          (async (params: any) => {
            console.error(
              "getCustomerByEmail function not available, returning mock data"
            );
            return {
              data: {
                customerByEmail: null,
              },
            };
          });

        const response = await getCustomerByEmailFunc({ email });

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
          depositAccount: {
            ...customer.depositAccount,
            balance: {
              pendingUSD: centsToUSD(customer.depositAccount.balance.pending),
              settledUSD: centsToUSD(customer.depositAccount.balance.settled),
            },
            withdrawals: customer.depositAccount.withdrawals.map((w: any) => ({
              ...w,
              amountUSD: centsToUSD(w.amount),
            })),
            deposits: customer.depositAccount.deposits.map((d: any) => ({
              ...d,
              amountUSD: centsToUSD(d.amount),
            })),
          },
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
        console.error("Error in get-customer-details tool:", error);
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

export function registerCreditFacilitiesTool(server: McpServer) {
  server.tool(
    "get-credit-facilities",
    "Retrieve a list of credit facilities matching the specified criteria. This tool allows searching, filtering, and sorting of credit facilities across all customers.",
    {
      limit: z
        .number()
        .optional()
        .describe(
          "The maximum number of credit facilities to retrieve (defaults to 10)."
        ),
      offset: z
        .number()
        .optional()
        .describe(
          "The number of credit facilities to skip before starting to retrieve results."
        ),
      filterBy: z
        .nativeEnum(CreditFacilitiesFilterBy as any)
        .optional()
        .describe(
          "Filter credit facilities by a specific attribute (STATUS or COLLATERALIZATION_STATE)."
        ),
      filterValue: z
        .string()
        .optional()
        .describe(
          "The value to filter by. When filterBy is STATUS, must be one of: ACTIVE, PENDING, CLOSED, LIQUIDATED. When filterBy is COLLATERALIZATION_STATE, must be one of: NORMAL, UNDER_COLLATERALIZED, SEVERELY_UNDER_COLLATERALIZED, LIQUIDATION."
        ),
      sortBy: z
        .nativeEnum(CreditFacilitiesSortBy as any)
        .optional()
        .describe(
          "Sort credit facilities by a specific attribute (CREATED_AT or FACILITY_AMOUNT)."
        ),
      sortDirection: z
        .nativeEnum(SortDirection as any)
        .optional()
        .describe("Direction to sort results (ASC or DESC)."),
    },
    async (params: {
      limit?: number;
      offset?: number;
      filterBy?: string;
      filterValue?: string;
      sortBy?: string;
      sortDirection?: string;
    }) => {
      try {
        // Default values
        const queryParams = {
          limit: params.limit || 10,
          offset: params.offset || 0,
          filterBy: params.filterBy,
          filterValue: params.filterValue,
          sortBy: params.sortBy || CreditFacilitiesSortBy.CREATED_AT,
          sortDirection: params.sortDirection || SortDirection.DESC,
        };

        // Validate values if filtering is applied
        if (queryParams.filterBy && queryParams.filterValue) {
          if (
            queryParams.filterBy === CreditFacilitiesFilterBy.STATUS &&
            !(queryParams.filterValue in CreditFacilityStatus)
          ) {
            return {
              content: [
                {
                  type: "text",
                  text: `Invalid filter value for STATUS. Must be one of: ${Object.values(
                    CreditFacilityStatus
                  ).join(", ")}`,
                },
              ],
            };
          }

          if (
            queryParams.filterBy ===
              CreditFacilitiesFilterBy.COLLATERALIZATION_STATE &&
            !(queryParams.filterValue in CollateralizationState)
          ) {
            return {
              content: [
                {
                  type: "text",
                  text: `Invalid filter value for COLLATERALIZATION_STATE. Must be one of: ${Object.values(
                    CollateralizationState
                  ).join(", ")}`,
                },
              ],
            };
          }
        }

        // This is a placeholder in case creditFacilities is not available
        const creditFacilitiesFunc =
          creditFacilities ||
          (async (params: any) => {
            console.error(
              "creditFacilities function not available, returning mock data"
            );
            return {
              data: {
                creditFacilities: [],
              },
            };
          });

        const response = await creditFacilitiesFunc(queryParams);

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

        const facilities = response.data.creditFacilities;

        if (!facilities || facilities.length === 0) {
          return {
            content: [
              {
                type: "text",
                text: "No credit facilities found matching the criteria.",
              },
            ],
          };
        }

        // Format the results for display
        const formattedFacilities = facilities.map((facility: any) => ({
          ...facility,
          facilityAmountUSD: centsToUSD(facility.facilityAmount),
          collateralBTC: satoshisToBTC(facility.collateral),
          currentCvl: {
            totalPercentage: toPercentage(facility.currentCvl.total),
            disbursedPercentage: toPercentage(facility.currentCvl.disbursed),
          },
        }));

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(formattedFacilities, null, 2),
            },
          ],
        };
      } catch (error) {
        console.error("Error in get-credit-facilities tool:", error);
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

export function registerCreditFacilityDetailsTool(server: McpServer) {
  server.tool(
    "get-credit-facility-details",
    "Retrieve detailed information about a specific credit facility by its ID. This includes terms, status, collateralization, and transaction history.",
    {
      id: z
        .string()
        .uuid()
        .describe("The UUID of the credit facility to retrieve details for."),
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

        // Process and format the credit facility data
        const formattedFacility = {
          ...facility,
          facilityAmountUSD: centsToUSD(facility.facilityAmount),
          collateralBTC: satoshisToBTC(facility.collateral),
          creditFacilityTerms: {
            ...facility.creditFacilityTerms,
            annualRatePercentage: toPercentage(
              facility.creditFacilityTerms.annualRate
            ),
            oneTimeFeeRatePercentage: toPercentage(
              facility.creditFacilityTerms.oneTimeFeeRate
            ),
            liquidationCvlPercentage: toPercentage(
              facility.creditFacilityTerms.liquidationCvl
            ),
            marginCallCvlPercentage: toPercentage(
              facility.creditFacilityTerms.marginCallCvl
            ),
            initialCvlPercentage: toPercentage(
              facility.creditFacilityTerms.initialCvl
            ),
          },
          currentCvl: {
            totalPercentage: toPercentage(facility.currentCvl.total),
            disbursedPercentage: toPercentage(facility.currentCvl.disbursed),
          },
          transactions: facility.transactions.map((tx: any) => {
            // Common transaction fields
            const baseTx = {
              recordedAt: tx.recordedAt,
              txId: tx.txId,
            };

            // Handle specific transaction types
            if ("cents" in tx) {
              return {
                ...baseTx,
                type: "cents" in tx && "reason" in tx ? "fee" : "payment",
                amountUSD: centsToUSD(tx.cents),
              };
            } else if ("satoshis" in tx) {
              return {
                ...baseTx,
                type: "collateral",
                action: tx.action,
                amountBTC: satoshisToBTC(tx.satoshis),
              };
            } else if ("state" in tx) {
              return {
                ...baseTx,
                type: "collateralization",
                state: tx.state,
                collateralBTC: satoshisToBTC(tx.collateral),
                outstandingInterestUSD: centsToUSD(tx.outstandingInterest),
                term: tx.term,
              };
            } else if ("reason" in tx) {
              return {
                ...baseTx,
                type: "liquidation",
                reason: tx.reason,
              };
            } else {
              return {
                ...baseTx,
                type: "expiration",
              };
            }
          }),
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

export function registerCustomerCreditFacilitiesTool(server: McpServer) {
  server.tool(
    "get-customer-credit-facility",
    "Retrieve detailed information about a customer's credit facilities by their email address. This includes terms, status, collateralization, and transaction history for all of the customer's credit facilities.",
    {
      email: z
        .string()
        .describe(
          "The email address of the customer whose credit facilities are being requested."
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

        if (!customer.creditFacilities || customer.creditFacilities.length === 0) {
          return {
            content: [
              {
                type: "text",
                text: `Customer ${email} has no credit facilities.`,
              },
            ],
          };
        }

        // Process and format all credit facilities
        const formattedFacilities = customer.creditFacilities.map(
          (facility: any) => {
            const formattedFacility = {
              ...facility,
              facilityAmountUSD: centsToUSD(facility.facilityAmount),
              collateralBTC: satoshisToBTC(facility.collateral),
              creditFacilityTerms: facility.creditFacilityTerms
                ? {
                    ...facility.creditFacilityTerms,
                    annualRatePercentage: toPercentage(
                      facility.creditFacilityTerms.annualRate
                    ),
                    oneTimeFeeRatePercentage: toPercentage(
                      facility.creditFacilityTerms.oneTimeFeeRate
                    ),
                    liquidationCvlPercentage: toPercentage(
                      facility.creditFacilityTerms.liquidationCvl
                    ),
                    marginCallCvlPercentage: toPercentage(
                      facility.creditFacilityTerms.marginCallCvl
                    ),
                    initialCvlPercentage: toPercentage(
                      facility.creditFacilityTerms.initialCvl
                    ),
                  }
                : null,
              currentCvl: facility.currentCvl
                ? {
                    totalPercentage: toPercentage(facility.currentCvl.total),
                    disbursedPercentage: toPercentage(
                      facility.currentCvl.disbursed
                    ),
                  }
                : null,
              transactions: facility.transactions.map((tx: any) => {
                // Common transaction fields
                const baseTx = {
                  recordedAt: tx.recordedAt,
                  txId: tx.txId,
                };

                // Handle specific transaction types
                if ("cents" in tx) {
                  return {
                    ...baseTx,
                    type: "cents" in tx && "reason" in tx ? "fee" : "payment",
                    amountUSD: centsToUSD(tx.cents),
                  };
                } else if ("satoshis" in tx) {
                  return {
                    ...baseTx,
                    type: "collateral",
                    action: tx.action,
                    amountBTC: satoshisToBTC(tx.satoshis),
                  };
                } else if ("state" in tx) {
                  return {
                    ...baseTx,
                    type: "collateralization",
                    state: tx.state,
                    collateralBTC: satoshisToBTC(tx.collateral),
                    outstandingInterestUSD: centsToUSD(tx.outstandingInterest),
                    term: tx.term,
                  };
                } else if ("reason" in tx) {
                  return {
                    ...baseTx,
                    type: "liquidation",
                    reason: tx.reason,
                  };
                } else {
                  return {
                    ...baseTx,
                    type: "expiration",
                  };
                }
              }),
            };

            return formattedFacility;
          }
        );

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  customerId: customer.customerId,
                  creditFacilities: formattedFacilities,
                },
                null,
                2
              ),
            },
          ],
        };
      } catch (error) {
        console.error("Error in get-customer-credit-facility tool:", error);
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

// Function to register all tools at once
export function registerAllTools(server: McpServer) {
  registerCustomerDetailsTool(server);
  registerCreditFacilitiesTool(server);
  registerCreditFacilityDetailsTool(server);
  registerCustomerCreditFacilitiesTool(server);
} 