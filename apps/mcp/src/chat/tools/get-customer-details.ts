import { z } from "zod";
// Use CommonJS compatible import pattern for @apollo/client
import { createRequire } from "module";
const require = createRequire(import.meta.url);
const { gql } = require("@apollo/client");
// Import from our operations module instead
import { getCustomerByEmail } from "../../lib/graphql/operations.js";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";

// Handle missing currency utilities
const centsToUSD = (cents: number): string => `$${(cents / 100).toFixed(2)}`;

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
