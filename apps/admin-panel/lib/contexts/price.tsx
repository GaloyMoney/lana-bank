"use client"

import { createContext, useCallback, useContext, useState } from "react"

type PriceContextType = {
  price: number
  satsToCents: null | ((sats: number) => number)
}

const initialPriceValues: PriceContextType = {
  price: 60000_00, // 1BTC = 6000000 cents
  satsToCents: null,
}

const PriceContext = createContext(initialPriceValues)
const PriceProvider: React.FC<React.PropsWithChildren> = ({ children }) => {
  // TODO: some subscription that sets the price
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [price, _setPrice] = useState(initialPriceValues.price)

  const satsToCents = useCallback(
    (sats: number) => sats * (1 / 100000000) * price,
    [price],
  )

  return (
    <PriceContext.Provider value={{ price, satsToCents }}>
      {children}
    </PriceContext.Provider>
  )
}

const usePrice = () => useContext(PriceContext)

export { PriceProvider, usePrice }
