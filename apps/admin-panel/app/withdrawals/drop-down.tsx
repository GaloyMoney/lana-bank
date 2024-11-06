"use client"
import { useRouter } from "next/navigation"
import { IoEllipsisHorizontal } from "react-icons/io5"

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/primitive/dropdown-menu"
import { Button } from "@/components/primitive/button"
import { WithdrawalStatus } from "@/lib/graphql/generated"

interface WithdrawalDropdownProps {
  withdrawal: WithdrawalWithCustomer
  onConfirm: () => void
  onCancel: () => void
}

const WithdrawalDropdown: React.FC<WithdrawalDropdownProps> = ({
  withdrawal,
  onConfirm,
  onCancel,
}) => {
  const router = useRouter()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <Button variant="ghost">
          <IoEllipsisHorizontal className="w-4 h-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="text-sm">
        <DropdownMenuItem
          onClick={() => {
            router.push(`/withdrawals/${withdrawal.withdrawalId}`)
          }}
        >
          View Withdrawal Details
        </DropdownMenuItem>
        {withdrawal.status === WithdrawalStatus.PendingConfirmation && (
          <>
            <DropdownMenuItem onClick={onConfirm}>Confirm Withdraw</DropdownMenuItem>
            <DropdownMenuItem onClick={onCancel}>Cancel Withdraw</DropdownMenuItem>
          </>
        )}
        <DropdownMenuItem
          onClick={() => router.push(`/customers/${withdrawal.customer?.customerId}`)}
        >
          View Customer Details
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

export default WithdrawalDropdown
