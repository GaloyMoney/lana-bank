import { z } from "zod";
import { tool } from "ai";
import { gql } from "@apollo/client";
import {
  CreditFacilitiesQueryVariables,
  CreditFacilitiesFilterBy,
  CreditFacilitiesSortBy,
  CollateralizationState,
  CreditFacilityStatus,
  SortDirection,
  creditFacilities,
} from "@/lib/graphql/generated";
import { centsToUSD, satoshisToBTC, toPercentage } from "@/lib/utils/currency";

const CollateralizationStateSchema = z
  .nativeEnum(CollateralizationState)
  .describe(`Collateralization states`);

const CreditFacilityStatusSchema = z
  .nativeEnum(CreditFacilityStatus)
  .describe(`Facility statuses`);

const CreditFacilitiesFilterBySchema = z
  .nativeEnum(CreditFacilitiesFilterBy)
  .describe(`Filter by fields`);

const CreditFacilitiesSortBySchema = z
  .nativeEnum(CreditFacilitiesSortBy)
  .describe(`Sort by fields`);

const SortDirectionSchema = z
  .nativeEnum(SortDirection)
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

export const getCreditFacilitiesTool = tool({
  type: "function",
  description:
    "Retrieve list of credit facilities with filtering and sorting. When using filter with status or collateralizationState, you must also specify field as 'STATUS' or 'COLLATERALIZATION_STATE' respectively. Limited to 5 facilities per request.",
  parameters: z.object({
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
  }),
  execute: async ({ first, after, sort, filter }) => {
    const response = await creditFacilities({ first, after, sort, filter });

    if ("error" in response) {
      return response;
    }

    //  temporary fix for currency conversion this should happen from backend.
    return {
      edges: response.data.creditFacilities?.edges?.map((edge) => ({
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
              usdBalance: centsToUSD(edge.node.balance.outstanding.usdBalance),
            },
          },
          customer: {
            customerId: edge.node.customer.customerId,
            email: edge.node.customer.email,
          },
        },
      })),
      pageInfo: response.data.creditFacilities?.pageInfo,
    };
  },
});
