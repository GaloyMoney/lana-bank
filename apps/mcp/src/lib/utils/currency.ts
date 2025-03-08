export const centsToUSD = (cents: number): string => {
  return `${cents / 100} USD`;
};

export const satoshisToBTC = (satoshis: number): string => {
  return `${satoshis / 100000000} BTC`;
};

export const toPercentage = (value: number): string => {
  return `${value}%`;
};
