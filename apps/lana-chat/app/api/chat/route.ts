import { openai } from "@ai-sdk/openai";
import { streamText } from "ai";
import { getCustomerDetailsTool } from "./tools/get-customer-details";
import { getCustomerCreditFacilitiesTool } from "./tools/get-customer-credit-facility";
import { getCreditFacilitiesTool } from "./tools/get-credit-facilities";
import { getCreditFacilityDetailsTool } from "./tools/get-credit-facility-details";

const systemPrompt = ` 
    You are an assistant exclusively designed to help users explore and understand data from a banking and Bitcoin-focused application. 
    You must:
    1. Focus solely on the banking and functions/tools response and there data, including accounts, credit facilities, transactions, approvals, and Bitcoin.
    2. Use tables to present lists and structured data clearly, when applicable.
    3. Support users in queries, pagination, filtering, and sorting based on the schema.
    You must not answer questions or provide assistance unrelated to this schema or the app.`;

export async function POST(req: Request) {
  const { messages } = await req.json();

  const result = streamText({
    model: openai("gpt-4o-mini"),
    system: systemPrompt,
    messages,
    maxSteps: 5,
    tools: {
      getCustomerDetails: getCustomerDetailsTool,
      getCustomerCreditFacilities: getCustomerCreditFacilitiesTool,
      getCreditFacilities: getCreditFacilitiesTool,
      getCreditFacilityDetails: getCreditFacilityDetailsTool,
    },
  });

  return result.toDataStreamResponse({
    getErrorMessage: (err) => {
      console.error(err);
      return "An error occurred while processing the request.";
    },
  });
}
