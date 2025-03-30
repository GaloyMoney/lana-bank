// this file is autogenerated by codegen
/* eslint-disable */
import { gql } from '@apollo/client';
import * as Apollo from '@apollo/client';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
const defaultOptions = {} as const;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  AnnualRatePct: { input: any; output: any; }
  CVLPct: { input: any; output: any; }
  DisbursalIdx: { input: any; output: any; }
  OneTimeFeeRatePct: { input: any; output: any; }
  Satoshis: { input: any; output: any; }
  Timestamp: { input: any; output: any; }
  UUID: { input: any; output: any; }
  UsdCents: { input: any; output: any; }
};

export enum AccountStatus {
  Active = 'ACTIVE',
  Inactive = 'INACTIVE'
}

export type CancelledWithdrawalEntry = {
  __typename?: 'CancelledWithdrawalEntry';
  recordedAt: Scalars['Timestamp']['output'];
  withdrawal: Withdrawal;
};

export type Collateral = {
  __typename?: 'Collateral';
  btcBalance: Scalars['Satoshis']['output'];
};

export enum CollateralAction {
  Add = 'ADD',
  Remove = 'REMOVE'
}

export enum CollateralizationState {
  FullyCollateralized = 'FULLY_COLLATERALIZED',
  NoCollateral = 'NO_COLLATERAL',
  UnderLiquidationThreshold = 'UNDER_LIQUIDATION_THRESHOLD',
  UnderMarginCallThreshold = 'UNDER_MARGIN_CALL_THRESHOLD'
}

export type CreditFacility = {
  __typename?: 'CreditFacility';
  activatedAt?: Maybe<Scalars['Timestamp']['output']>;
  balance: CreditFacilityBalance;
  collateral: Scalars['Satoshis']['output'];
  collateralizationState: CollateralizationState;
  createdAt: Scalars['Timestamp']['output'];
  creditFacilityId: Scalars['UUID']['output'];
  creditFacilityTerms: TermValues;
  currentCvl: FacilityCvl;
  disbursals: Array<CreditFacilityDisbursal>;
  maturesAt?: Maybe<Scalars['Timestamp']['output']>;
  facilityAmount: Scalars['UsdCents']['output'];
  id: Scalars['ID']['output'];
  repaymentPlan: Array<CreditFacilityRepaymentInPlan>;
  status: CreditFacilityStatus;
  transactions: Array<CreditFacilityHistoryEntry>;
};

export type CreditFacilityBalance = {
  __typename?: 'CreditFacilityBalance';
  collateral: Collateral;
  disbursed: Disbursed;
  dueOutstanding: Outstanding;
  facilityRemaining: FacilityRemaining;
  interest: Interest;
  outstanding: Outstanding;
};

export type CreditFacilityCollateralUpdated = {
  __typename?: 'CreditFacilityCollateralUpdated';
  action: CollateralAction;
  recordedAt: Scalars['Timestamp']['output'];
  satoshis: Scalars['Satoshis']['output'];
  txId: Scalars['UUID']['output'];
};

export type CreditFacilityCollateralizationUpdated = {
  __typename?: 'CreditFacilityCollateralizationUpdated';
  collateral: Scalars['Satoshis']['output'];
  outstandingDisbursal: Scalars['UsdCents']['output'];
  outstandingInterest: Scalars['UsdCents']['output'];
  price: Scalars['UsdCents']['output'];
  recordedAt: Scalars['Timestamp']['output'];
  state: CollateralizationState;
};

export type CreditFacilityDisbursal = {
  __typename?: 'CreditFacilityDisbursal';
  amount: Scalars['UsdCents']['output'];
  createdAt: Scalars['Timestamp']['output'];
  disbursalId: Scalars['UUID']['output'];
  id: Scalars['ID']['output'];
  index: Scalars['DisbursalIdx']['output'];
  status: DisbursalStatus;
};

export type CreditFacilityDisbursalExecuted = {
  __typename?: 'CreditFacilityDisbursalExecuted';
  cents: Scalars['UsdCents']['output'];
  recordedAt: Scalars['Timestamp']['output'];
  txId: Scalars['UUID']['output'];
};

export type CreditFacilityHistoryEntry = CreditFacilityCollateralUpdated | CreditFacilityCollateralizationUpdated | CreditFacilityDisbursalExecuted | CreditFacilityIncrementalPayment | CreditFacilityInterestAccrued | CreditFacilityOrigination;

export type CreditFacilityIncrementalPayment = {
  __typename?: 'CreditFacilityIncrementalPayment';
  cents: Scalars['UsdCents']['output'];
  recordedAt: Scalars['Timestamp']['output'];
  txId: Scalars['UUID']['output'];
};

export type CreditFacilityInterestAccrued = {
  __typename?: 'CreditFacilityInterestAccrued';
  cents: Scalars['UsdCents']['output'];
  days: Scalars['Int']['output'];
  recordedAt: Scalars['Timestamp']['output'];
  txId: Scalars['UUID']['output'];
};

export type CreditFacilityOrigination = {
  __typename?: 'CreditFacilityOrigination';
  cents: Scalars['UsdCents']['output'];
  recordedAt: Scalars['Timestamp']['output'];
  txId: Scalars['UUID']['output'];
};

export type CreditFacilityPayment = {
  __typename?: 'CreditFacilityPayment';
  createdAt: Scalars['Timestamp']['output'];
  creditFacility: CreditFacility;
  disbursalAmount: Scalars['UsdCents']['output'];
  id: Scalars['ID']['output'];
  interestAmount: Scalars['UsdCents']['output'];
  paymentId: Scalars['UUID']['output'];
};

export type CreditFacilityRepaymentInPlan = {
  __typename?: 'CreditFacilityRepaymentInPlan';
  accrualAt: Scalars['Timestamp']['output'];
  dueAt: Scalars['Timestamp']['output'];
  initial: Scalars['UsdCents']['output'];
  outstanding: Scalars['UsdCents']['output'];
  repaymentType: CreditFacilityRepaymentType;
  status: CreditFacilityRepaymentStatus;
};

export enum CreditFacilityRepaymentStatus {
  Due = 'DUE',
  Overdue = 'OVERDUE',
  Paid = 'PAID',
  Upcoming = 'UPCOMING'
}

export enum CreditFacilityRepaymentType {
  Disbursal = 'DISBURSAL',
  Interest = 'INTEREST'
}

export enum CreditFacilityStatus {
  Active = 'ACTIVE',
  Closed = 'CLOSED',
  Matured = 'MATURED',
  PendingApproval = 'PENDING_APPROVAL',
  PendingCollateralization = 'PENDING_COLLATERALIZATION'
}

export type Customer = {
  __typename?: 'Customer';
  createdAt: Scalars['Timestamp']['output'];
  creditFacilities: Array<CreditFacility>;
  customerId: Scalars['UUID']['output'];
  depositAccount: DepositAccount;
  email: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  level: KycLevel;
  status: AccountStatus;
  telegramId: Scalars['String']['output'];
};

export type Deposit = {
  __typename?: 'Deposit';
  accountId: Scalars['UUID']['output'];
  amount: Scalars['UsdCents']['output'];
  createdAt: Scalars['Timestamp']['output'];
  depositId: Scalars['UUID']['output'];
  id: Scalars['ID']['output'];
  reference: Scalars['String']['output'];
};

export type DepositAccount = {
  __typename?: 'DepositAccount';
  balance: DepositAccountBalance;
  createdAt: Scalars['Timestamp']['output'];
  customerId: Scalars['UUID']['output'];
  depositAccountId: Scalars['UUID']['output'];
  deposits: Array<Deposit>;
  history: DepositAccountHistoryEntryConnection;
  id: Scalars['ID']['output'];
  withdrawals: Array<Withdrawal>;
};


export type DepositAccountHistoryArgs = {
  after?: InputMaybe<Scalars['String']['input']>;
  first: Scalars['Int']['input'];
};

export type DepositAccountBalance = {
  __typename?: 'DepositAccountBalance';
  pending: Scalars['UsdCents']['output'];
  settled: Scalars['UsdCents']['output'];
};

export type DepositAccountHistoryEntry = CancelledWithdrawalEntry | DepositEntry | DisbursalEntry | PaymentEntry | UnknownEntry | WithdrawalEntry;

export type DepositAccountHistoryEntryConnection = {
  __typename?: 'DepositAccountHistoryEntryConnection';
  /** A list of edges. */
  edges: Array<DepositAccountHistoryEntryEdge>;
  /** A list of nodes. */
  nodes: Array<DepositAccountHistoryEntry>;
  /** Information to aid in pagination. */
  pageInfo: PageInfo;
};

/** An edge in a connection. */
export type DepositAccountHistoryEntryEdge = {
  __typename?: 'DepositAccountHistoryEntryEdge';
  /** A cursor for use in pagination */
  cursor: Scalars['String']['output'];
  /** The item at the end of the edge */
  node: DepositAccountHistoryEntry;
};

export type DepositEntry = {
  __typename?: 'DepositEntry';
  deposit: Deposit;
  recordedAt: Scalars['Timestamp']['output'];
};

export type DisbursalEntry = {
  __typename?: 'DisbursalEntry';
  disbursal: CreditFacilityDisbursal;
  recordedAt: Scalars['Timestamp']['output'];
};

export enum DisbursalStatus {
  Approved = 'APPROVED',
  Confirmed = 'CONFIRMED',
  Denied = 'DENIED',
  New = 'NEW'
}

export type Disbursed = {
  __typename?: 'Disbursed';
  dueOutstanding: Outstanding;
  outstanding: Outstanding;
  total: Total;
};

export type Duration = {
  __typename?: 'Duration';
  period: Period;
  units: Scalars['Int']['output'];
};

export type FacilityCvl = {
  __typename?: 'FacilityCVL';
  disbursed: Scalars['CVLPct']['output'];
  total: Scalars['CVLPct']['output'];
};

export type FacilityRemaining = {
  __typename?: 'FacilityRemaining';
  usdBalance: Scalars['UsdCents']['output'];
};

export type Interest = {
  __typename?: 'Interest';
  dueOutstanding: Outstanding;
  outstanding: Outstanding;
  total: Total;
};

export enum InterestInterval {
  EndOfDay = 'END_OF_DAY',
  EndOfMonth = 'END_OF_MONTH'
}

export enum KycLevel {
  Advanced = 'ADVANCED',
  Basic = 'BASIC',
  NotKyced = 'NOT_KYCED'
}

export type Outstanding = {
  __typename?: 'Outstanding';
  usdBalance: Scalars['UsdCents']['output'];
};

/** Information about pagination in a connection */
export type PageInfo = {
  __typename?: 'PageInfo';
  /** When paginating forwards, the cursor to continue. */
  endCursor?: Maybe<Scalars['String']['output']>;
  /** When paginating forwards, are there more items? */
  hasNextPage: Scalars['Boolean']['output'];
  /** When paginating backwards, are there more items? */
  hasPreviousPage: Scalars['Boolean']['output'];
  /** When paginating backwards, the cursor to continue. */
  startCursor?: Maybe<Scalars['String']['output']>;
};

export type PaymentEntry = {
  __typename?: 'PaymentEntry';
  payment: CreditFacilityPayment;
  recordedAt: Scalars['Timestamp']['output'];
};

export enum Period {
  Months = 'MONTHS'
}

export type Query = {
  __typename?: 'Query';
  creditFacility?: Maybe<CreditFacility>;
  me: Subject;
  realtimePrice: RealtimePrice;
};


export type QueryCreditFacilityArgs = {
  id: Scalars['UUID']['input'];
};

export type RealtimePrice = {
  __typename?: 'RealtimePrice';
  usdCentsPerBtc: Scalars['UsdCents']['output'];
};

export type Subject = {
  __typename?: 'Subject';
  customer: Customer;
};

export type TermValues = {
  __typename?: 'TermValues';
  accrualCycleInterval: InterestInterval;
  annualRate: Scalars['AnnualRatePct']['output'];
  duration: Duration;
  accrualInterval: InterestInterval;
  initialCvl: Scalars['CVLPct']['output'];
  liquidationCvl: Scalars['CVLPct']['output'];
  marginCallCvl: Scalars['CVLPct']['output'];
  oneTimeFeeRate: Scalars['OneTimeFeeRatePct']['output'];
};

export type Total = {
  __typename?: 'Total';
  usdBalance: Scalars['UsdCents']['output'];
};

export type UnknownEntry = {
  __typename?: 'UnknownEntry';
  recordedAt: Scalars['Timestamp']['output'];
  txId: Scalars['UUID']['output'];
};

export type Withdrawal = {
  __typename?: 'Withdrawal';
  accountId: Scalars['UUID']['output'];
  amount: Scalars['UsdCents']['output'];
  createdAt: Scalars['Timestamp']['output'];
  id: Scalars['ID']['output'];
  reference: Scalars['String']['output'];
  status: WithdrawalStatus;
  withdrawalId: Scalars['UUID']['output'];
};

export type WithdrawalEntry = {
  __typename?: 'WithdrawalEntry';
  recordedAt: Scalars['Timestamp']['output'];
  withdrawal: Withdrawal;
};

export enum WithdrawalStatus {
  Cancelled = 'CANCELLED',
  Confirmed = 'CONFIRMED',
  Denied = 'DENIED',
  PendingApproval = 'PENDING_APPROVAL',
  PendingConfirmation = 'PENDING_CONFIRMATION'
}

export type GetCreditFacilityQueryVariables = Exact<{
  id: Scalars['UUID']['input'];
}>;


export type GetCreditFacilityQuery = { __typename?: 'Query', creditFacility?: { __typename?: 'CreditFacility', id: string, creditFacilityId: any, facilityAmount: any, collateral: any, collateralizationState: CollateralizationState, status: CreditFacilityStatus, createdAt: any, activatedAt?: any | null, maturesAt?: any | null, disbursals: Array<{ __typename?: 'CreditFacilityDisbursal', id: string, disbursalId: any, index: any, amount: any, status: DisbursalStatus, createdAt: any }>, creditFacilityTerms: { __typename?: 'TermValues', annualRate: any, accrualCycleInterval: InterestInterval, accrualInterval: InterestInterval, oneTimeFeeRate: any, liquidationCvl: any, marginCallCvl: any, initialCvl: any, duration: { __typename?: 'Duration', period: Period, units: number } }, balance: { __typename?: 'CreditFacilityBalance', facilityRemaining: { __typename?: 'FacilityRemaining', usdBalance: any }, disbursed: { __typename?: 'Disbursed', total: { __typename?: 'Total', usdBalance: any }, outstanding: { __typename?: 'Outstanding', usdBalance: any }, dueOutstanding: { __typename?: 'Outstanding', usdBalance: any } }, interest: { __typename?: 'Interest', total: { __typename?: 'Total', usdBalance: any }, outstanding: { __typename?: 'Outstanding', usdBalance: any }, dueOutstanding: { __typename?: 'Outstanding', usdBalance: any } }, collateral: { __typename?: 'Collateral', btcBalance: any }, dueOutstanding: { __typename?: 'Outstanding', usdBalance: any }, outstanding: { __typename?: 'Outstanding', usdBalance: any } }, currentCvl: { __typename?: 'FacilityCVL', total: any, disbursed: any }, repaymentPlan: Array<{ __typename?: 'CreditFacilityRepaymentInPlan', repaymentType: CreditFacilityRepaymentType, status: CreditFacilityRepaymentStatus, initial: any, outstanding: any, accrualAt: any, dueAt: any }>, transactions: Array<{ __typename?: 'CreditFacilityCollateralUpdated', satoshis: any, recordedAt: any, action: CollateralAction, txId: any } | { __typename?: 'CreditFacilityCollateralizationUpdated', state: CollateralizationState, collateral: any, outstandingInterest: any, outstandingDisbursal: any, recordedAt: any, price: any } | { __typename?: 'CreditFacilityDisbursalExecuted', cents: any, recordedAt: any, txId: any } | { __typename?: 'CreditFacilityIncrementalPayment', cents: any, recordedAt: any, txId: any } | { __typename?: 'CreditFacilityInterestAccrued', cents: any, recordedAt: any, txId: any, days: number } | { __typename?: 'CreditFacilityOrigination', cents: any, recordedAt: any, txId: any }> } | null };

export type MeQueryVariables = Exact<{ [key: string]: never; }>;


export type MeQuery = { __typename?: 'Query', me: { __typename?: 'Subject', customer: { __typename?: 'Customer', id: string, customerId: any, status: AccountStatus, level: KycLevel, createdAt: any, email: string, telegramId: string, depositAccount: { __typename?: 'DepositAccount', id: string, depositAccountId: any, customerId: any, createdAt: any, balance: { __typename?: 'DepositAccountBalance', settled: any, pending: any }, deposits: Array<{ __typename?: 'Deposit', id: string, depositId: any, accountId: any, amount: any, createdAt: any, reference: string }>, withdrawals: Array<{ __typename?: 'Withdrawal', id: string, withdrawalId: any, accountId: any, amount: any, createdAt: any, reference: string, status: WithdrawalStatus }> }, creditFacilities: Array<{ __typename?: 'CreditFacility', id: string, creditFacilityId: any, collateralizationState: CollateralizationState, status: CreditFacilityStatus, createdAt: any, balance: { __typename?: 'CreditFacilityBalance', collateral: { __typename?: 'Collateral', btcBalance: any }, outstanding: { __typename?: 'Outstanding', usdBalance: any } } }> } } };

export type GetRealtimePriceUpdatesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetRealtimePriceUpdatesQuery = { __typename?: 'Query', realtimePrice: { __typename?: 'RealtimePrice', usdCentsPerBtc: any } };

export type GetTransactionHistoryQueryVariables = Exact<{
  first: Scalars['Int']['input'];
  after?: InputMaybe<Scalars['String']['input']>;
}>;


export type GetTransactionHistoryQuery = { __typename?: 'Query', me: { __typename?: 'Subject', customer: { __typename?: 'Customer', depositAccount: { __typename?: 'DepositAccount', history: { __typename?: 'DepositAccountHistoryEntryConnection', pageInfo: { __typename?: 'PageInfo', hasNextPage: boolean, endCursor?: string | null, hasPreviousPage: boolean, startCursor?: string | null }, edges: Array<{ __typename?: 'DepositAccountHistoryEntryEdge', cursor: string, node: { __typename?: 'CancelledWithdrawalEntry', recordedAt: any, withdrawal: { __typename?: 'Withdrawal', id: string, withdrawalId: any, accountId: any, amount: any, createdAt: any, reference: string, status: WithdrawalStatus } } | { __typename?: 'DepositEntry', recordedAt: any, deposit: { __typename?: 'Deposit', id: string, depositId: any, accountId: any, amount: any, createdAt: any, reference: string } } | { __typename?: 'DisbursalEntry', recordedAt: any, disbursal: { __typename?: 'CreditFacilityDisbursal', id: string, disbursalId: any, index: any, amount: any, createdAt: any, status: DisbursalStatus } } | { __typename?: 'PaymentEntry', recordedAt: any, payment: { __typename?: 'CreditFacilityPayment', id: string, paymentId: any, interestAmount: any, disbursalAmount: any, createdAt: any } } | { __typename?: 'UnknownEntry' } | { __typename?: 'WithdrawalEntry', recordedAt: any, withdrawal: { __typename?: 'Withdrawal', id: string, withdrawalId: any, accountId: any, amount: any, createdAt: any, reference: string, status: WithdrawalStatus } } }> } } } } };


export const GetCreditFacilityDocument = gql`
    query GetCreditFacility($id: UUID!) {
  creditFacility(id: $id) {
    id
    creditFacilityId
    facilityAmount
    collateral
    collateralizationState
    status
    createdAt
    activatedAt
    maturesAt
    disbursals {
      id
      disbursalId
      index
      amount
      status
      createdAt
    }
    creditFacilityTerms {
      annualRate
      accrualCycleInterval
      accrualInterval
      oneTimeFeeRate
      duration {
        period
        units
      }
      liquidationCvl
      marginCallCvl
      initialCvl
    }
    balance {
      facilityRemaining {
        usdBalance
      }
      disbursed {
        total {
          usdBalance
        }
        outstanding {
          usdBalance
        }
        dueOutstanding {
          usdBalance
        }
      }
      interest {
        total {
          usdBalance
        }
        outstanding {
          usdBalance
        }
        dueOutstanding {
          usdBalance
        }
      }
      collateral {
        btcBalance
      }
      dueOutstanding {
        usdBalance
      }
      outstanding {
        usdBalance
      }
    }
    currentCvl {
      total
      disbursed
    }
    repaymentPlan {
      repaymentType
      status
      initial
      outstanding
      accrualAt
      dueAt
    }
    transactions {
      ... on CreditFacilityIncrementalPayment {
        cents
        recordedAt
        txId
      }
      ... on CreditFacilityCollateralUpdated {
        satoshis
        recordedAt
        action
        txId
      }
      ... on CreditFacilityOrigination {
        cents
        recordedAt
        txId
      }
      ... on CreditFacilityCollateralizationUpdated {
        state
        collateral
        outstandingInterest
        outstandingDisbursal
        recordedAt
        price
      }
      ... on CreditFacilityDisbursalExecuted {
        cents
        recordedAt
        txId
      }
      ... on CreditFacilityInterestAccrued {
        cents
        recordedAt
        txId
        days
      }
    }
  }
}
    `;

/**
 * __useGetCreditFacilityQuery__
 *
 * To run a query within a React component, call `useGetCreditFacilityQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetCreditFacilityQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetCreditFacilityQuery({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useGetCreditFacilityQuery(baseOptions: Apollo.QueryHookOptions<GetCreditFacilityQuery, GetCreditFacilityQueryVariables>) {
  const options = { ...defaultOptions, ...baseOptions }
  return Apollo.useQuery<GetCreditFacilityQuery, GetCreditFacilityQueryVariables>(GetCreditFacilityDocument, options);
}
export function useGetCreditFacilityLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetCreditFacilityQuery, GetCreditFacilityQueryVariables>) {
  const options = { ...defaultOptions, ...baseOptions }
  return Apollo.useLazyQuery<GetCreditFacilityQuery, GetCreditFacilityQueryVariables>(GetCreditFacilityDocument, options);
}
export type GetCreditFacilityQueryHookResult = ReturnType<typeof useGetCreditFacilityQuery>;
export type GetCreditFacilityLazyQueryHookResult = ReturnType<typeof useGetCreditFacilityLazyQuery>;
export type GetCreditFacilityQueryResult = Apollo.QueryResult<GetCreditFacilityQuery, GetCreditFacilityQueryVariables>;
export const MeDocument = gql`
    query me {
  me {
    customer {
      id
      customerId
      status
      level
      createdAt
      email
      telegramId
      depositAccount {
        id
        depositAccountId
        customerId
        createdAt
        balance {
          settled
          pending
        }
        deposits {
          id
          depositId
          accountId
          amount
          createdAt
          reference
        }
        withdrawals {
          id
          withdrawalId
          accountId
          amount
          createdAt
          reference
          status
        }
      }
      creditFacilities {
        id
        creditFacilityId
        collateralizationState
        status
        createdAt
        balance {
          collateral {
            btcBalance
          }
          outstanding {
            usdBalance
          }
        }
      }
    }
  }
}
    `;

/**
 * __useMeQuery__
 *
 * To run a query within a React component, call `useMeQuery` and pass it any options that fit your needs.
 * When your component renders, `useMeQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useMeQuery({
 *   variables: {
 *   },
 * });
 */
export function useMeQuery(baseOptions?: Apollo.QueryHookOptions<MeQuery, MeQueryVariables>) {
  const options = { ...defaultOptions, ...baseOptions }
  return Apollo.useQuery<MeQuery, MeQueryVariables>(MeDocument, options);
}
export function useMeLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<MeQuery, MeQueryVariables>) {
  const options = { ...defaultOptions, ...baseOptions }
  return Apollo.useLazyQuery<MeQuery, MeQueryVariables>(MeDocument, options);
}
export type MeQueryHookResult = ReturnType<typeof useMeQuery>;
export type MeLazyQueryHookResult = ReturnType<typeof useMeLazyQuery>;
export type MeQueryResult = Apollo.QueryResult<MeQuery, MeQueryVariables>;
export const GetRealtimePriceUpdatesDocument = gql`
    query GetRealtimePriceUpdates {
  realtimePrice {
    usdCentsPerBtc
  }
}
    `;

/**
 * __useGetRealtimePriceUpdatesQuery__
 *
 * To run a query within a React component, call `useGetRealtimePriceUpdatesQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetRealtimePriceUpdatesQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetRealtimePriceUpdatesQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetRealtimePriceUpdatesQuery(baseOptions?: Apollo.QueryHookOptions<GetRealtimePriceUpdatesQuery, GetRealtimePriceUpdatesQueryVariables>) {
  const options = { ...defaultOptions, ...baseOptions }
  return Apollo.useQuery<GetRealtimePriceUpdatesQuery, GetRealtimePriceUpdatesQueryVariables>(GetRealtimePriceUpdatesDocument, options);
}
export function useGetRealtimePriceUpdatesLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetRealtimePriceUpdatesQuery, GetRealtimePriceUpdatesQueryVariables>) {
  const options = { ...defaultOptions, ...baseOptions }
  return Apollo.useLazyQuery<GetRealtimePriceUpdatesQuery, GetRealtimePriceUpdatesQueryVariables>(GetRealtimePriceUpdatesDocument, options);
}
export type GetRealtimePriceUpdatesQueryHookResult = ReturnType<typeof useGetRealtimePriceUpdatesQuery>;
export type GetRealtimePriceUpdatesLazyQueryHookResult = ReturnType<typeof useGetRealtimePriceUpdatesLazyQuery>;
export type GetRealtimePriceUpdatesQueryResult = Apollo.QueryResult<GetRealtimePriceUpdatesQuery, GetRealtimePriceUpdatesQueryVariables>;
export const GetTransactionHistoryDocument = gql`
    query GetTransactionHistory($first: Int!, $after: String) {
  me {
    customer {
      depositAccount {
        history(first: $first, after: $after) {
          pageInfo {
            hasNextPage
            endCursor
            hasPreviousPage
            startCursor
          }
          edges {
            cursor
            node {
              ... on DepositEntry {
                recordedAt
                deposit {
                  id
                  depositId
                  accountId
                  amount
                  createdAt
                  reference
                }
              }
              ... on WithdrawalEntry {
                recordedAt
                withdrawal {
                  id
                  withdrawalId
                  accountId
                  amount
                  createdAt
                  reference
                  status
                }
              }
              ... on CancelledWithdrawalEntry {
                recordedAt
                withdrawal {
                  id
                  withdrawalId
                  accountId
                  amount
                  createdAt
                  reference
                  status
                }
              }
              ... on DisbursalEntry {
                recordedAt
                disbursal {
                  id
                  disbursalId
                  index
                  amount
                  createdAt
                  status
                }
              }
              ... on PaymentEntry {
                recordedAt
                payment {
                  id
                  paymentId
                  interestAmount
                  disbursalAmount
                  createdAt
                }
              }
            }
          }
        }
      }
    }
  }
}
    `;

/**
 * __useGetTransactionHistoryQuery__
 *
 * To run a query within a React component, call `useGetTransactionHistoryQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetTransactionHistoryQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetTransactionHistoryQuery({
 *   variables: {
 *      first: // value for 'first'
 *      after: // value for 'after'
 *   },
 * });
 */
export function useGetTransactionHistoryQuery(baseOptions: Apollo.QueryHookOptions<GetTransactionHistoryQuery, GetTransactionHistoryQueryVariables>) {
  const options = { ...defaultOptions, ...baseOptions }
  return Apollo.useQuery<GetTransactionHistoryQuery, GetTransactionHistoryQueryVariables>(GetTransactionHistoryDocument, options);
}
export function useGetTransactionHistoryLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetTransactionHistoryQuery, GetTransactionHistoryQueryVariables>) {
  const options = { ...defaultOptions, ...baseOptions }
  return Apollo.useLazyQuery<GetTransactionHistoryQuery, GetTransactionHistoryQueryVariables>(GetTransactionHistoryDocument, options);
}
export type GetTransactionHistoryQueryHookResult = ReturnType<typeof useGetTransactionHistoryQuery>;
export type GetTransactionHistoryLazyQueryHookResult = ReturnType<typeof useGetTransactionHistoryLazyQuery>;
export type GetTransactionHistoryQueryResult = Apollo.QueryResult<GetTransactionHistoryQuery, GetTransactionHistoryQueryVariables>;