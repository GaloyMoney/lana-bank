import { z } from "zod";
import { tool } from "ai";
import { gql } from "@apollo/client";
import {
  GetCustomerByEmailQueryVariables,
  getCustomerByEmail,
} from "@/lib/graphql/generated";
import { centsToUSD } from "@/lib/utils/currency";

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

export const getCustomerDetailsTool = tool({
  type: "function",
  description:
    "Retrieve general information about a customer by their email address. This includes account balances, deposit history, withdrawal history, KYC document statuses, and basic customer information. **Note**: This does not include any details about the customer's credit facility.",
  parameters: z.object({
    email: z
      .string()
      .describe(
        "The email address of the customer whose general details (excluding credit facility) are being requested."
      ),
  }),
  execute: async ({ email }) => {
    const response = await getCustomerByEmail({ email });

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
          depositAccount: {
            ...customer.depositAccount,
            balance: {
              pendingUSD: centsToUSD(customer.depositAccount.balance.pending),
              settledUSD: centsToUSD(customer.depositAccount.balance.settled),
            },
            withdrawals: customer.depositAccount.withdrawals.map((w) => ({
              ...w,
              amountUSD: centsToUSD(w.amount),
            })),
            deposits: customer.depositAccount.deposits.map((d) => ({
              ...d,
              amountUSD: centsToUSD(d.amount),
            })),
          },
        },
      },
    };
  },
});
