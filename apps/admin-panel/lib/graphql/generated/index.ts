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
  Satoshis: { input: any; output: any; }
  UUID: { input: string; output: string; }
  UsdCents: { input: any; output: any; }
};

export type AccountBalancesByCurrency = {
  __typename?: 'AccountBalancesByCurrency';
  btc: LayeredBtcAccountBalances;
  usd: LayeredUsdAccountBalances;
  usdt: LayeredUsdAccountBalances;
};

export type AccountLedgerSummary = {
  __typename?: 'AccountLedgerSummary';
  name: Scalars['String']['output'];
  totalBalance: AccountBalancesByCurrency;
};

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

export type LayeredBtcAccountBalances = {
  __typename?: 'LayeredBtcAccountBalances';
  encumbrance: BtcAccountBalance;
  pending: BtcAccountBalance;
  settled: BtcAccountBalance;
};

export type LayeredUsdAccountBalances = {
  __typename?: 'LayeredUsdAccountBalances';
  encumbrance: UsdAccountBalance;
  pending: UsdAccountBalance;
  settled: UsdAccountBalance;
};

export type LoanOutstanding = {
  __typename?: 'LoanOutstanding';
  usdBalance: Scalars['UsdCents']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  shareholderEquityAdd: SuccessPayload;
};


export type MutationShareholderEquityAddArgs = {
  input: ShareholderEquityAddInput;
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

export type Query = {
  __typename?: 'Query';
  loan?: Maybe<FixedTermLoan>;
  trialBalance?: Maybe<AccountLedgerSummary>;
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
  balance: UserBalance;
  btcDepositAddress: Scalars['String']['output'];
  email: Scalars['String']['output'];
  loans: Array<FixedTermLoan>;
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

export type GetLoanDetailsQueryVariables = Exact<{
  id: Scalars['UUID']['input'];
}>;


export type GetLoanDetailsQuery = { __typename?: 'Query', loan?: { __typename?: 'FixedTermLoan', loanId: string, user: { __typename?: 'User', userId: string }, balance: { __typename?: 'FixedTermLoanBalance', collateral: { __typename?: 'Collateral', btcBalance: any }, outstanding: { __typename?: 'LoanOutstanding', usdBalance: any }, interestIncurred: { __typename?: 'InterestIncome', usdBalance: any } } } | null };

export type GetLoansForUserQueryVariables = Exact<{
  id: Scalars['UUID']['input'];
}>;


export type GetLoansForUserQuery = { __typename?: 'Query', user?: { __typename?: 'User', userId: string, loans: Array<{ __typename?: 'FixedTermLoan', loanId: string, balance: { __typename?: 'FixedTermLoanBalance', collateral: { __typename?: 'Collateral', btcBalance: any }, outstanding: { __typename?: 'LoanOutstanding', usdBalance: any }, interestIncurred: { __typename?: 'InterestIncome', usdBalance: any } } }> } | null };

export type GetUserByUserIdQueryVariables = Exact<{
  id: Scalars['UUID']['input'];
}>;


export type GetUserByUserIdQuery = { __typename?: 'Query', user?: { __typename?: 'User', userId: string, email: string, btcDepositAddress: string, ustDepositAddress: string, balance: { __typename?: 'UserBalance', unallocatedCollateral: { __typename?: 'UnallocatedCollateral', settled: { __typename?: 'BtcBalance', btcBalance: any } }, checking: { __typename?: 'Checking', settled: { __typename?: 'UsdBalance', usdBalance: any }, pending: { __typename?: 'UsdBalance', usdBalance: any } } } } | null };

export type UsersQueryVariables = Exact<{
  first: Scalars['Int']['input'];
  after?: InputMaybe<Scalars['String']['input']>;
}>;


export type UsersQuery = { __typename?: 'Query', users: { __typename?: 'UserConnection', nodes: Array<{ __typename?: 'User', userId: string, email: string, btcDepositAddress: string, ustDepositAddress: string, balance: { __typename?: 'UserBalance', unallocatedCollateral: { __typename?: 'UnallocatedCollateral', settled: { __typename?: 'BtcBalance', btcBalance: any } }, checking: { __typename?: 'Checking', settled: { __typename?: 'UsdBalance', usdBalance: any }, pending: { __typename?: 'UsdBalance', usdBalance: any } } } }>, pageInfo: { __typename?: 'PageInfo', endCursor?: string | null, startCursor?: string | null, hasNextPage: boolean, hasPreviousPage: boolean } } };


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
export const GetUserByUserIdDocument = gql`
    query getUserByUserId($id: UUID!) {
  user(id: $id) {
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