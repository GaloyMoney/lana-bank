"use client"

import { gql } from "@apollo/client"

import { PageHeading } from "@/components/page-heading"
import { useAuditLogsQuery } from "@/lib/graphql/generated"

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"

gql`
  query AuditLogs($first: Int!, $after: String) {
    audit(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          subject {
            ... on User {
              userId
              email
              roles
            }
            ... on Customer {
              customerId
              email
              status
              level
              applicantId
            }
            ... on System {
              name
            }
          }
          object
          action
          authorized
          createdAt
        }
      }
      pageInfo {
        hasNextPage
        endCursor
      }
    }
  }
`

function LogsPage() {
  const { data, fetchMore, loading } = useAuditLogsQuery({
    variables: { first: 100 },
    fetchPolicy: "cache-and-network",
  })

  const handleFetchMore = async () => {
    if (data?.audit.pageInfo.hasNextPage && !loading) {
      await fetchMore({
        variables: {
          after: data.audit.pageInfo.endCursor,
        },
        updateQuery: (prev, { fetchMoreResult }) => {
          if (!fetchMoreResult) return prev
          return {
            audit: {
              ...fetchMoreResult.audit,
              edges: [...prev.audit.edges, ...fetchMoreResult.audit.edges],
            },
          }
        },
      })
    }
  }
  return (
    <main className="text-white min-h-screen">
      <div className="flex flex-col mb-8">
        <PageHeading className="mb-4 text-white">Audit Logs</PageHeading>
        <div className="overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Type</TableHead>
                <TableHead>Subject</TableHead>
                <TableHead>Object</TableHead>
                <TableHead>Action</TableHead>
                <TableHead>Authorized</TableHead>
                <TableHead>Created At</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data?.audit.edges.map((item) => (
                <TableRow key={item.node.id}>
                  <TableCell>{item.node.subject.__typename}</TableCell>
                  <TableCell>
                    {item.node.subject.__typename === "User"
                      ? item.node.subject.email
                      : ""}
                    {item.node.subject.__typename === "Customer"
                      ? item.node.subject.email
                      : ""}
                    {item.node.subject.__typename === "System"
                      ? item.node.subject.name
                      : ""}
                  </TableCell>
                  <TableCell>{item.node.object}</TableCell>
                  <TableCell>{item.node.action}</TableCell>
                  <TableCell>{item.node.authorized ? "Yes" : "No"}</TableCell>
                  <TableCell>{item.node.createdAt}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
          {data?.audit.pageInfo.hasNextPage && (
            <div className="flex justify-center mt-4">
              <button
                onClick={handleFetchMore}
                className="px-4 py-2 text-sm font-medium text-gray-200 bg-gray-700 rounded hover:bg-gray-600"
                disabled={loading}
              >
                {loading ? "Loading..." : "Show more"}
              </button>
            </div>
          )}
        </div>
      </div>
    </main>
  )
}

export default LogsPage
