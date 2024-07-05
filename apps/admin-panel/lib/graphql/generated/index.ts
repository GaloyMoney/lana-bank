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
  LoanAnnualRate: { input: any; output: any; }
  LoanCVLPct: { input: any; output: any; }
  Satoshis: { input: any; output: any; }
  Timestamp: { input: any; output: any; }
  UUID: { input: string; output: string; }
  UsdCents: { input: any; output: any; }
};

export type AccountBalance = {
  __typename?: 'AccountBalance';
  balance: AccountBalancesByCurrency;
  name: Scalars['String']['output'];
};

export type AccountBalancesByCurrency = {
  __typename?: 'AccountBalancesByCurrency';
  btc: LayeredBtcAccountBalances;
  usd: LayeredUsdAccountBalances;
  usdt: LayeredUsdAccountBalances;
};

export type AccountSetAndMemberBalances = {
  __typename?: 'AccountSetAndMemberBalances';
  balance: AccountBalancesByCurrency;
  memberBalances: Array<AccountSetMemberBalance>;
  name: Scalars['String']['output'];
};

export type AccountSetBalance = {
  __typename?: 'AccountSetBalance';
  balance: AccountBalancesByCurrency;
  name: Scalars['String']['output'];
};

export type AccountSetMemberBalance = AccountBalance | AccountSetBalance;

export enum AccountStatus {
  Active = 'ACTIVE',
  Inactive = 'INACTIVE'
}

export type BtcAccountBalance = {
  __typename?: 'BtcAccountBalance';
  credit: Scalars['Satoshis']['output'];
  debit: Scalars['Satoshis']['output'];
  net: Scalars['Satoshis']['output'];
};

export type BtcBalance = {
  __typename?: 'BtcBalance';
  btcBalance: Scalars['Satoshis']['output'];
};

export type Checking = {
  __typename?: 'Checking';
  pending: UsdBalance;
  settled: UsdBalance;
};

export type Collateral = {
  __typename?: 'Collateral';
  btcBalance: Scalars['Satoshis']['output'];
};

export type CurrentTermsUpdateInput = {
  annualRate: Scalars['LoanAnnualRate']['input'];
  duration: LoanDurationInput;
  initialCvl: Scalars['LoanCVLPct']['input'];
  interval: InterestInterval;
  liquidationCvl: Scalars['LoanCVLPct']['input'];
  marginCallCvl: Scalars['LoanCVLPct']['input'];
};

export type CurrentTermsUpdatePayload = {
  __typename?: 'CurrentTermsUpdatePayload';
  terms: Terms;
};

export type FixedTermLoan = {
  __typename?: 'FixedTermLoan';
  balance: FixedTermLoanBalance;
  loanId: Scalars['UUID']['output'];
  user: User;
};

export type FixedTermLoanBalance = {
  __typename?: 'FixedTermLoanBalance';
  collateral: Collateral;
  interestIncurred: InterestIncome;
  outstanding: LoanOutstanding;
};

export type InterestIncome = {
  __typename?: 'InterestIncome';
  usdBalance: Scalars['UsdCents']['output'];
};

export enum InterestInterval {
  EndOfMonth = 'END_OF_MONTH'
}

export enum KycLevel {
  One = 'ONE',
  Two = 'TWO',
  Zero = 'ZERO'
}

export type LayeredBtcAccountBalances = {
  __typename?: 'LayeredBtcAccountBalances';
  all: BtcAccountBalance;
  encumbrance: BtcAccountBalance;
  pending: BtcAccountBalance;
  settled: BtcAccountBalance;
};

export type LayeredUsdAccountBalances = {
  __typename?: 'LayeredUsdAccountBalances';
  all: UsdAccountBalance;
  encumbrance: UsdAccountBalance;
  pending: UsdAccountBalance;
  settled: UsdAccountBalance;
};

export type Loan = {
  __typename?: 'Loan';
  id: Scalars['ID']['output'];
  loanId: Scalars['UUID']['output'];
  startDate: Scalars['Timestamp']['output'];
};

export type LoanCreateInput = {
  desiredPrincipal: Scalars['UsdCents']['input'];
  userId: Scalars['UUID']['input'];
};

export type LoanCreatePayload = {
  __typename?: 'LoanCreatePayload';
  loan: Loan;
};

export type LoanDuration = {
  __typename?: 'LoanDuration';
  period: Period;
  units: Scalars['Int']['output'];
};

export type LoanDurationInput = {
  period: Period;
  units: Scalars['Int']['input'];
};

export type LoanOutstanding = {
  __typename?: 'LoanOutstanding';
  usdBalance: Scalars['UsdCents']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  currentTermsUpdate: CurrentTermsUpdatePayload;
  loanCreate: LoanCreatePayload;
  shareholderEquityAdd: SuccessPayload;
  sumsubPermalinkCreate: SumsubPermalinkCreatePayload;
};


export type MutationCurrentTermsUpdateArgs = {
  input: CurrentTermsUpdateInput;
};


export type MutationLoanCreateArgs = {
  input: LoanCreateInput;
};


export type MutationShareholderEquityAddArgs = {
  input: ShareholderEquityAddInput;
};


export type MutationSumsubPermalinkCreateArgs = {
  input: SumsubPermalinkCreateInput;
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

export enum Period {
  Months = 'MONTHS'
}

export type Query = {
  __typename?: 'Query';
  loan?: Maybe<FixedTermLoan>;
  trialBalance?: Maybe<AccountSetAndMemberBalances>;
  user?: Maybe<User>;
  users: UserConnection;
};


export type QueryLoanArgs = {
  id: Scalars['UUID']['input'];
};


export type QueryUserArgs = {
  id: Scalars['UUID']['input'];
};


export type QueryUsersArgs = {
  after?: InputMaybe<Scalars['String']['input']>;
  first: Scalars['Int']['input'];
};

export type ShareholderEquityAddInput = {
  amount: Scalars['UsdCents']['input'];
  reference: Scalars['String']['input'];
};

export type SuccessPayload = {
  __typename?: 'SuccessPayload';
  success: Scalars['Boolean']['output'];
};

export type SumsubPermalinkCreateInput = {
  userId: Scalars['String']['input'];
};

export type SumsubPermalinkCreatePayload = {
  __typename?: 'SumsubPermalinkCreatePayload';
  url: Scalars['String']['output'];
};

export type TermValues = {
  __typename?: 'TermValues';
  annualRate: Scalars['LoanAnnualRate']['output'];
  duration: LoanDuration;
  initialCvl: Scalars['LoanCVLPct']['output'];
  interval: InterestInterval;
  liquidationCvl: Scalars['LoanCVLPct']['output'];
  marginCallCvl: Scalars['LoanCVLPct']['output'];
};

export type Terms = {
  __typename?: 'Terms';
  id: Scalars['ID']['output'];
  termsId: Scalars['UUID']['output'];
  values: TermValues;
};

export type UnallocatedCollateral = {
  __typename?: 'UnallocatedCollateral';
  settled: BtcBalance;
};

export type UsdAccountBalance = {
  __typename?: 'UsdAccountBalance';
  credit: Scalars['UsdCents']['output'];
  debit: Scalars['UsdCents']['output'];
  net: Scalars['UsdCents']['output'];
};

export type UsdBalance = {
  __typename?: 'UsdBalance';
  usdBalance: Scalars['UsdCents']['output'];
};

export type User = {
  __typename?: 'User';
  applicantId?: Maybe<Scalars['String']['output']>;
  balance: UserBalance;
  btcDepositAddress: Scalars['String']['output'];
  email: Scalars['String']['output'];
  level: KycLevel;
  loans: Array<FixedTermLoan>;
  status: AccountStatus;
  userId: Scalars['UUID']['output'];
  ustDepositAddress: Scalars['String']['output'];
};

export type UserBalance = {
  __typename?: 'UserBalance';
  checking: Checking;
  unallocatedCollateral: UnallocatedCollateral;
};

export type UserConnection = {
  __typename?: 'UserConnection';
  /** A list of edges. */
  edges: Array<UserEdge>;
  /** A list of nodes. */
  nodes: Array<User>;
  /** Information to aid in pagination. */
  pageInfo: PageInfo;
};

/** An edge in a connection. */
export type UserEdge = {
  __typename?: 'UserEdge';
  /** A cursor for use in pagination */
  cursor: Scalars['String']['output'];
  /** The item at the end of the edge */
  node: User;
};

export type SumsubPermalinkCreateMutationVariables = Exact<{
  input: SumsubPermalinkCreateInput;
}>;


export type SumsubPermalinkCreateMutation = { __typename?: 'Mutation', sumsubPermalinkCreate: { __typename?: 'SumsubPermalinkCreatePayload', url: string } };

export type GetLoanDetailsQueryVariables = Exact<{
  id: Scalars['UUID']['input'];
}>;


export type GetLoanDetailsQuery = { __typename?: 'Query', loan?: { __typename?: 'FixedTermLoan', loanId: string, user: { __typename?: 'User', userId: string }, balance: { __typename?: 'FixedTermLoanBalance', collateral: { __typename?: 'Collateral', btcBalance: any }, outstanding: { __typename?: 'LoanOutstanding', usdBalance: any }, interestIncurred: { __typename?: 'InterestIncome', usdBalance: any } } } | null };

export type GetLoansForUserQueryVariables = Exact<{
  id: Scalars['UUID']['input'];
}>;


export type GetLoansForUserQuery = { __typename?: 'Query', user?: { __typename?: 'User', userId: string, loans: Array<{ __typename?: 'FixedTermLoan', loanId: string, balance: { __typename?: 'FixedTermLoanBalance', collateral: { __typename?: 'Collateral', btcBalance: any }, outstanding: { __typename?: 'LoanOutstanding', usdBalance: any }, interestIncurred: { __typename?: 'InterestIncome', usdBalance: any } } }> } | null };

export type GetTrialBalanceQueryVariables = Exact<{ [key: string]: never; }>;


export type GetTrialBalanceQuery = { __typename?: 'Query', trialBalance?: { __typename?: 'AccountSetAndMemberBalances', name: string, balance: { __typename?: 'AccountBalancesByCurrency', btc: { __typename?: 'LayeredBtcAccountBalances', all: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any } }, usd: { __typename?: 'LayeredUsdAccountBalances', all: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any } }, usdt: { __typename?: 'LayeredUsdAccountBalances', all: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any } } }, memberBalances: Array<{ __typename?: 'AccountBalance', name: string, balance: { __typename?: 'AccountBalancesByCurrency', btc: { __typename?: 'LayeredBtcAccountBalances', all: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any } }, usd: { __typename?: 'LayeredUsdAccountBalances', all: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any } }, usdt: { __typename?: 'LayeredUsdAccountBalances', all: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any } } } } | { __typename?: 'AccountSetBalance', name: string, balance: { __typename?: 'AccountBalancesByCurrency', btc: { __typename?: 'LayeredBtcAccountBalances', all: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any } }, usd: { __typename?: 'LayeredUsdAccountBalances', all: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any } }, usdt: { __typename?: 'LayeredUsdAccountBalances', all: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any } } } }> } | null };

export type BalancesByCurrencyFragment = { __typename?: 'AccountBalancesByCurrency', btc: { __typename?: 'LayeredBtcAccountBalances', all: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any } }, usd: { __typename?: 'LayeredUsdAccountBalances', all: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any } }, usdt: { __typename?: 'LayeredUsdAccountBalances', all: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any } } };

export type BtcBalancesFragment = { __typename?: 'LayeredBtcAccountBalances', all: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'BtcAccountBalance', net: any, debit: any, credit: any } };

export type UsdBalancesFragment = { __typename?: 'LayeredUsdAccountBalances', all: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, settled: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, pending: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any }, encumbrance: { __typename?: 'UsdAccountBalance', net: any, debit: any, credit: any } };

export type GetUserByUserIdQueryVariables = Exact<{
  id: Scalars['UUID']['input'];
}>;


export type GetUserByUserIdQuery = { __typename?: 'Query', user?: { __typename?: 'User', userId: string, email: string, status: AccountStatus, level: KycLevel, applicantId?: string | null, btcDepositAddress: string, ustDepositAddress: string, balance: { __typename?: 'UserBalance', unallocatedCollateral: { __typename?: 'UnallocatedCollateral', settled: { __typename?: 'BtcBalance', btcBalance: any } }, checking: { __typename?: 'Checking', settled: { __typename?: 'UsdBalance', usdBalance: any }, pending: { __typename?: 'UsdBalance', usdBalance: any } } } } | null };

export type UsersQueryVariables = Exact<{
  first: Scalars['Int']['input'];
  after?: InputMaybe<Scalars['String']['input']>;
}>;


export type UsersQuery = { __typename?: 'Query', users: { __typename?: 'UserConnection', nodes: Array<{ __typename?: 'User', userId: string, email: string, btcDepositAddress: string, ustDepositAddress: string, balance: { __typename?: 'UserBalance', unallocatedCollateral: { __typename?: 'UnallocatedCollateral', settled: { __typename?: 'BtcBalance', btcBalance: any } }, checking: { __typename?: 'Checking', settled: { __typename?: 'UsdBalance', usdBalance: any }, pending: { __typename?: 'UsdBalance', usdBalance: any } } } }>, pageInfo: { __typename?: 'PageInfo', endCursor?: string | null, startCursor?: string | null, hasNextPage: boolean, hasPreviousPage: boolean } } };

export const BtcBalancesFragmentDoc = gql`
    fragment btcBalances on LayeredBtcAccountBalances {
  all {
    net
    debit
    credit
  }
  settled {
    net
    debit
    credit
  }
  pending {
    net
    debit
    credit
  }
  encumbrance {
    net
    debit
    credit
  }
}
    `;
export const UsdBalancesFragmentDoc = gql`
    fragment usdBalances on LayeredUsdAccountBalances {
  all {
    net
    debit
    credit
  }
  settled {
    net
    debit
    credit
  }
  pending {
    net
    debit
    credit
  }
  encumbrance {
    net
    debit
    credit
  }
}
    `;
export const BalancesByCurrencyFragmentDoc = gql`
    fragment balancesByCurrency on AccountBalancesByCurrency {
  btc: btc {
    ...btcBalances
  }
  usd: usd {
    ...usdBalances
  }
  usdt: usdt {
    ...usdBalances
  }
}
    ${BtcBalancesFragmentDoc}
${UsdBalancesFragmentDoc}`;
export const SumsubPermalinkCreateDocument = gql`
    mutation sumsubPermalinkCreate($input: SumsubPermalinkCreateInput!) {
  sumsubPermalinkCreate(input: $input) {
    url
  }
}
    `;
export type SumsubPermalinkCreateMutationFn = Apollo.MutationFunction<SumsubPermalinkCreateMutation, SumsubPermalinkCreateMutationVariables>;

/**
 * __useSumsubPermalinkCreateMutation__
 *
 * To run a mutation, you first call `useSumsubPermalinkCreateMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useSumsubPermalinkCreateMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [sumsubPermalinkCreateMutation, { data, loading, error }] = useSumsubPermalinkCreateMutation({
 *   variables: {
 *      input: // value for 'input'
 *   },
 * });
 */
export function useSumsubPermalinkCreateMutation(baseOptions?: Apollo.MutationHookOptions<SumsubPermalinkCreateMutation, SumsubPermalinkCreateMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<SumsubPermalinkCreateMutation, SumsubPermalinkCreateMutationVariables>(SumsubPermalinkCreateDocument, options);
      }
export type SumsubPermalinkCreateMutationHookResult = ReturnType<typeof useSumsubPermalinkCreateMutation>;
export type SumsubPermalinkCreateMutationResult = Apollo.MutationResult<SumsubPermalinkCreateMutation>;
export type SumsubPermalinkCreateMutationOptions = Apollo.BaseMutationOptions<SumsubPermalinkCreateMutation, SumsubPermalinkCreateMutationVariables>;
export const GetLoanDetailsDocument = gql`
    query GetLoanDetails($id: UUID!) {
  loan(id: $id) {
    loanId
    user {
      userId
    }
    balance {
      collateral {
        btcBalance
      }
      outstanding {
        usdBalance
      }
      interestIncurred {
        usdBalance
      }
    }
  }
}
    `;

/**
 * __useGetLoanDetailsQuery__
 *
 * To run a query within a React component, call `useGetLoanDetailsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetLoanDetailsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetLoanDetailsQuery({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useGetLoanDetailsQuery(baseOptions: Apollo.QueryHookOptions<GetLoanDetailsQuery, GetLoanDetailsQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetLoanDetailsQuery, GetLoanDetailsQueryVariables>(GetLoanDetailsDocument, options);
      }
export function useGetLoanDetailsLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetLoanDetailsQuery, GetLoanDetailsQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetLoanDetailsQuery, GetLoanDetailsQueryVariables>(GetLoanDetailsDocument, options);
        }
export type GetLoanDetailsQueryHookResult = ReturnType<typeof useGetLoanDetailsQuery>;
export type GetLoanDetailsLazyQueryHookResult = ReturnType<typeof useGetLoanDetailsLazyQuery>;
export type GetLoanDetailsQueryResult = Apollo.QueryResult<GetLoanDetailsQuery, GetLoanDetailsQueryVariables>;
export const GetLoansForUserDocument = gql`
    query GetLoansForUser($id: UUID!) {
  user(id: $id) {
    userId
    loans {
      loanId
      balance {
        collateral {
          btcBalance
        }
        outstanding {
          usdBalance
        }
        interestIncurred {
          usdBalance
        }
      }
    }
  }
}
    `;

/**
 * __useGetLoansForUserQuery__
 *
 * To run a query within a React component, call `useGetLoansForUserQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetLoansForUserQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetLoansForUserQuery({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useGetLoansForUserQuery(baseOptions: Apollo.QueryHookOptions<GetLoansForUserQuery, GetLoansForUserQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetLoansForUserQuery, GetLoansForUserQueryVariables>(GetLoansForUserDocument, options);
      }
export function useGetLoansForUserLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetLoansForUserQuery, GetLoansForUserQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetLoansForUserQuery, GetLoansForUserQueryVariables>(GetLoansForUserDocument, options);
        }
export type GetLoansForUserQueryHookResult = ReturnType<typeof useGetLoansForUserQuery>;
export type GetLoansForUserLazyQueryHookResult = ReturnType<typeof useGetLoansForUserLazyQuery>;
export type GetLoansForUserQueryResult = Apollo.QueryResult<GetLoansForUserQuery, GetLoansForUserQueryVariables>;
export const GetTrialBalanceDocument = gql`
    query GetTrialBalance {
  trialBalance {
    name
    balance {
      ...balancesByCurrency
    }
    memberBalances {
      ... on AccountBalance {
        name
        balance {
          ...balancesByCurrency
        }
      }
      ... on AccountSetBalance {
        name
        balance {
          ...balancesByCurrency
        }
      }
    }
  }
}
    ${BalancesByCurrencyFragmentDoc}`;

/**
 * __useGetTrialBalanceQuery__
 *
 * To run a query within a React component, call `useGetTrialBalanceQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetTrialBalanceQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetTrialBalanceQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetTrialBalanceQuery(baseOptions?: Apollo.QueryHookOptions<GetTrialBalanceQuery, GetTrialBalanceQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetTrialBalanceQuery, GetTrialBalanceQueryVariables>(GetTrialBalanceDocument, options);
      }
export function useGetTrialBalanceLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetTrialBalanceQuery, GetTrialBalanceQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetTrialBalanceQuery, GetTrialBalanceQueryVariables>(GetTrialBalanceDocument, options);
        }
export type GetTrialBalanceQueryHookResult = ReturnType<typeof useGetTrialBalanceQuery>;
export type GetTrialBalanceLazyQueryHookResult = ReturnType<typeof useGetTrialBalanceLazyQuery>;
export type GetTrialBalanceQueryResult = Apollo.QueryResult<GetTrialBalanceQuery, GetTrialBalanceQueryVariables>;
export const GetUserByUserIdDocument = gql`
    query getUserByUserId($id: UUID!) {
  user(id: $id) {
    userId
    email
    status
    level
    applicantId
    btcDepositAddress
    ustDepositAddress
    balance {
      unallocatedCollateral {
        settled {
          btcBalance
        }
      }
      checking {
        settled {
          usdBalance
        }
        pending {
          usdBalance
        }
      }
    }
  }
}
    `;

/**
 * __useGetUserByUserIdQuery__
 *
 * To run a query within a React component, call `useGetUserByUserIdQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetUserByUserIdQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetUserByUserIdQuery({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useGetUserByUserIdQuery(baseOptions: Apollo.QueryHookOptions<GetUserByUserIdQuery, GetUserByUserIdQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetUserByUserIdQuery, GetUserByUserIdQueryVariables>(GetUserByUserIdDocument, options);
      }
export function useGetUserByUserIdLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetUserByUserIdQuery, GetUserByUserIdQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetUserByUserIdQuery, GetUserByUserIdQueryVariables>(GetUserByUserIdDocument, options);
        }
export type GetUserByUserIdQueryHookResult = ReturnType<typeof useGetUserByUserIdQuery>;
export type GetUserByUserIdLazyQueryHookResult = ReturnType<typeof useGetUserByUserIdLazyQuery>;
export type GetUserByUserIdQueryResult = Apollo.QueryResult<GetUserByUserIdQuery, GetUserByUserIdQueryVariables>;
export const UsersDocument = gql`
    query Users($first: Int!, $after: String) {
  users(first: $first, after: $after) {
    nodes {
      userId
      email
      btcDepositAddress
      ustDepositAddress
      balance {
        unallocatedCollateral {
          settled {
            btcBalance
          }
        }
        checking {
          settled {
            usdBalance
          }
          pending {
            usdBalance
          }
        }
      }
    }
    pageInfo {
      endCursor
      startCursor
      hasNextPage
      hasPreviousPage
    }
  }
}
    `;

/**
 * __useUsersQuery__
 *
 * To run a query within a React component, call `useUsersQuery` and pass it any options that fit your needs.
 * When your component renders, `useUsersQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useUsersQuery({
 *   variables: {
 *      first: // value for 'first'
 *      after: // value for 'after'
 *   },
 * });
 */
export function useUsersQuery(baseOptions: Apollo.QueryHookOptions<UsersQuery, UsersQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<UsersQuery, UsersQueryVariables>(UsersDocument, options);
      }
export function useUsersLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<UsersQuery, UsersQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<UsersQuery, UsersQueryVariables>(UsersDocument, options);
        }
export type UsersQueryHookResult = ReturnType<typeof useUsersQuery>;
export type UsersLazyQueryHookResult = ReturnType<typeof useUsersLazyQuery>;
export type UsersQueryResult = Apollo.QueryResult<UsersQuery, UsersQueryVariables>;