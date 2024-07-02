const BTC_PER_SATOSHI = 100000000
const USD_PER_CENT = 100
const USDT_PER_CENT = 100

const formatAmount = (amount: number, currency: Currency) => {
  const formatter = new Intl.NumberFormat("en-US")

  switch (currency) {
    case "btc": {
      const btc = Math.floor(amount / BTC_PER_SATOSHI)
      const sats = amount % BTC_PER_SATOSHI
      return `${formatter.format(btc)}.${sats.toString().padStart(8, "0")}`
    }
    case "usd": {
      const dollars = Math.floor(amount / USD_PER_CENT)
      const cents = amount % USD_PER_CENT
      return `${formatter.format(dollars)}.${cents.toString().padStart(2, "0")}`
    }
    case "usdt": {
      const dollars = Math.floor(amount / USDT_PER_CENT)
      const cents = amount % USDT_PER_CENT
      return `${formatter.format(dollars)}.${cents.toString().padStart(2, "0")}`
    }
  }
}

export type Currency = "btc" | "usd" | "usdt"
type BalanceProps = {
  amount: number
  currency: Currency
}
const Balance: React.FC<BalanceProps> = ({ amount, currency }) => {
  const isNegative = amount < 0
  const formattedAmount = formatAmount(Math.abs(amount), currency)
  const formattedAmountWithSymbol = isNegative
    ? `(${formattedAmount})`
    : `${formattedAmount}`

  return (
    <div className="flex justify-end space-x-1">
      <div className="font-mono">{formattedAmountWithSymbol}</div>
    </div>
  )
}

export default Balance
