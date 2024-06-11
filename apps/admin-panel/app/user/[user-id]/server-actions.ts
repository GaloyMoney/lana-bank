"use server"
import { userDeposit } from "@/lib/graphql/mutation/user-deposit"
import { userPledgeCollateral } from "@/lib/graphql/mutation/user-pledge-colletral"
import { currencyConverter } from "@/lib/utils"

export const userPledgeCollateralServerAction = async ({
  userId,
  amount,
  reference,
}: {
  userId: string
  amount: number
  reference: string
}) => {
  if (!amount || !reference || !userId) {
    return {
      error: true,
      message: "Invalid Input",
    }
  }

  const response = await userPledgeCollateral({
    amount,
    reference,
    userId,
  })

  if (response instanceof Error) {
    return {
      error: true,
      message: response.message,
    }
  }

  return {
    error: false,
    message: "User collateral pledged successfully",
  }
}

export const userDepositServerAction = async ({
  userId,
  amount,
  reference,
}: {
  userId: string
  amount: number
  reference: string
}) => {
  if (!amount || !reference || !userId) {
    return {
      error: true,
      message: "Invalid Input",
    }
  }

  const response = await userDeposit({
    amount: currencyConverter.usdToCents(amount),
    reference,
    userId,
  })

  if (response instanceof Error) {
    return {
      error: true,
      message: response.message,
    }
  }

  return {
    error: false,
    message: "success",
  }
}
