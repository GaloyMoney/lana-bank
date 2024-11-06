import { cva, VariantProps } from "class-variance-authority"

import { cn, CENTS_PER_USD, SATS_PER_BTC } from "@/lib/utils"

const formatAmount = (amount: number, currency: Currency) => {
  const formatter = new Intl.NumberFormat("en-US")

  switch (currency) {
    case "btc": {
      const btc = Math.floor(amount / SATS_PER_BTC)
      const sats = Math.floor(amount % SATS_PER_BTC)
      return `${formatter.format(btc)}.${sats.toString().padStart(8, "0")}`
    }
    case "usd": {
      const dollars = Math.floor(amount / CENTS_PER_USD)
      const cents = Math.floor(amount % CENTS_PER_USD)
      return `${formatter.format(dollars)}.${cents.toString().padStart(2, "0")}`
    }
  }
}

export type Currency = "btc" | "usd"

type BalanceProps = {
  amount: number
  currency: Currency
  className?: string
} & VariantProps<typeof balanceVariants>

const balanceVariants = cva("", {
  variants: {
    align: {
      end: "flex justify-end space-x-1",
      start: "flex justify-start space-x-1",
      center: "flex justify-center space-x-1",
      right: "flex justify-right space-x-1",
    },
  },
})

const Balance: React.FC<BalanceProps> = ({ amount, currency, className, align }) => {
  const isNegative = amount < 0
  const formattedAmount = formatAmount(Math.abs(amount), currency)
  const formattedAmountWithSymbol = isNegative
    ? `(${formattedAmount})`
    : `${formattedAmount}`

  return (
    <div className={cn(balanceVariants({ align }), "flex gap-0.5", className)}>
      {currency === "usd" && <div>$</div>}
      <div>
        {formattedAmountWithSymbol}
        {currency === "btc" && " BTC"}
      </div>
    </div>
  )
}
export default Balance
