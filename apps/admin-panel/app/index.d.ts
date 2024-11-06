type Layers = "all" | "settled" | "pending"
type TransactionType = "netCredit" | "netDebit" | "debit" | "credit"

type WithdrawalWithCustomer = {
  __typename?: "Withdrawal"
  status: WithdrawalStatus
  customerId: string
  withdrawalId: string
  reference: string
  amount: number
  customer?: {
    __typename?: "Customer"
    customerId: string
    email: string
    applicantId?: string | null
  } | null
}
