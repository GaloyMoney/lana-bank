import { CustomerType } from "@/lib/graphql/generated"
import { removeUnderscore } from "@/lib/utils"

export const CustomerLabel = ({
  email,
  customerType,
}: {
  email: string
  customerType: CustomerType
}) => (
  <span>
    {email}{" "}
    <span className="whitespace-nowrap">({removeUnderscore(customerType)})</span>
  </span>
)
