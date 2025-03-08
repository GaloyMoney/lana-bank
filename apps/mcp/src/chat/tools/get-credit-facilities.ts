import { z } from "zod";
// Use CommonJS compatible import pattern for @apollo/client
import { createRequire } from "module";
const require = createRequire(import.meta.url);
const { gql } = require("@apollo/client");
// Import from our operations module instead
import { creditFacilities } from "../../lib/graphql/operations.js";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";

// Handle missing currency utilities
const centsToUSD = (cents: number): string => `$${(cents / 100).toFixed(2)}`;
const satoshisToBTC = (satoshis: number): string =>
  `${(satoshis / 100000000).toFixed(8)} BTC`;
const toPercentage = (value: number): string => `${(value * 100).toFixed(2)}%`;

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

const CollateralizationStateSchema = z
  .nativeEnum(CollateralizationState as any)
  .describe(`Collateralization states`);

const CreditFacilityStatusSchema = z
  .nativeEnum(CreditFacilityStatus as any)
  .describe(`Facility statuses`);

const CreditFacilitiesFilterBySchema = z
  .nativeEnum(CreditFacilitiesFilterBy as any)
  .describe(`Filter by fields`);

const CreditFacilitiesSortBySchema = z
  .nativeEnum(CreditFacilitiesSortBy as any)
  .describe(`Sort by fields`);

const SortDirectionSchema = z
  .nativeEnum(SortDirection as any)
  .describe(`Sort directions`);

const CreditFacilitiesFilterSchema = z
  .object({
    collateralizationState: CollateralizationStateSchema.optional().describe(
      "Filter facilities by collateralization state"
    ),
    field: CreditFacilitiesFilterBySchema.describe(
      "Required field to filter facilities by"
    ),
    status: CreditFacilityStatusSchema.optional().describe(
      "Filter facilities by status"
    ),
  })
  .nullish()
  .describe("Optional filters for credit facilities query");

const CreditFacilitiesSortSchema = z.object({
  by: CreditFacilitiesSortBySchema.optional().describe(
    "Field to sort facilities by"
  ),
  direction: SortDirectionSchema.optional().describe(
    "Sort direction (ascending/descending)"
  ),
});

gql`
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

// Export a function that registers the tool with the server
export function registerCreditFacilitiesTool(server: McpServer) {
  server.tool(
    "get-credit-facilities",
    "Retrieve list of credit facilities with filtering and sorting. When using filter with status or collateralizationState, you must also specify field as 'STATUS' or 'COLLATERALIZATION_STATE' respectively. Limited to 5 facilities per request.",
    {
      first: z
        .number()
        .min(1)
        .max(5)
        .describe("Number of facilities to fetch (5 max: cannot exceed 5)"),
      after: z
        .string()
        .optional()
        .describe("Pagination cursor. Null/undefined for first page"),
      sort: CreditFacilitiesSortSchema.optional(),
      filter: CreditFacilitiesFilterSchema.optional(),
    },
    async ({ first, after, sort, filter }: any) => {
      try {
        // This is a placeholder in case creditFacilities is not available
        const creditFacilitiesFunc =
          creditFacilities ||
          (async (params: any) => {
            console.error(
              "creditFacilities function not available, returning mock data"
            );
            return {
              data: {
                creditFacilities: {
                  edges: [],
                  pageInfo: { endCursor: null, hasNextPage: false },
                },
              },
            };
          });

        const response = await creditFacilitiesFunc({
          first,
          after,
          sort,
          filter,
        });

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

        // Transform the data
        const facilitiesData =
          response.data.creditFacilities?.edges?.map((edge: any) => ({
            cursor: edge.cursor,
            node: {
              creditFacilityId: edge.node.creditFacilityId,
              collateralizationState: edge.node.collateralizationState,
              createdAt: edge.node.createdAt,
              status: edge.node.status,
              facilityAmountUSD: centsToUSD(edge.node.facilityAmount),
              collateralBTC: satoshisToBTC(edge.node.collateral),
              currentCvl: {
                disbursed: toPercentage(edge.node.currentCvl.disbursed),
                total: toPercentage(edge.node.currentCvl.total),
              },
              balance: {
                collateral: {
                  btcBalance: satoshisToBTC(
                    edge.node.balance.collateral.btcBalance
                  ),
                },
                outstanding: {
                  usdBalance: centsToUSD(
                    edge.node.balance.outstanding.usdBalance
                  ),
                },
              },
              customer: {
                customerId: edge.node.customer.customerId,
                email: edge.node.customer.email,
              },
            },
          })) || [];

        const pageInfo = response.data.creditFacilities?.pageInfo;

        // Format the response for server.tool format
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  edges: facilitiesData,
                  pageInfo: pageInfo,
                },
                null,
                2
              ),
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
