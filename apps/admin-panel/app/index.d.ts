type Layers = "all" | "settled" | "pending" | "encumbrance"
type TransactionType = "netCredit" | "netDebit" | "debit" | "credit"

type WithdrawalWithCustomer = {
  __typename?: "Withdrawal"
  confirmed: boolean
  customerId: string
  withdrawalId: string
  amount: number
  customer?: {
    __typename?: "Customer"
    customerId: string
    email: string
    applicantId?: string | null
  } | null
}
