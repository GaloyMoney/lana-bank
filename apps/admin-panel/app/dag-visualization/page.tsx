"use client"

import React from "react"
import { gql, useQuery } from "@apollo/client"
import { useTranslations } from "next-intl"
import dynamic from "next/dynamic"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { Skeleton } from "@lana/web/ui/skeleton"

const D2Renderer = dynamic(() => import("./d2-renderer"), { ssr: false })

const ACCOUNT_SET_DAG_QUERY = gql`
  query AccountSetDag {
    accountSetDag {
      d2
    }
  }
`

interface AccountSetDagData {
  accountSetDag: {
    d2: string
  }
}

const DagVisualizationPage: React.FC = () => {
  const t = useTranslations("DAGVisualization")
  const { data, loading, error } = useQuery<AccountSetDagData>(ACCOUNT_SET_DAG_QUERY, {
    fetchPolicy: "cache-and-network",
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        {loading && !data && (
          <div className="space-y-2">
            <Skeleton className="h-8 w-full" />
            <Skeleton className="h-64 w-full" />
          </div>
        )}
        {error && <p className="text-destructive">{error.message}</p>}
        {data?.accountSetDag?.d2 && <D2Renderer d2Source={data.accountSetDag.d2} />}
      </CardContent>
    </Card>
  )
}

export default DagVisualizationPage
