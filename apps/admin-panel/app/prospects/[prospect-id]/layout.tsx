"use client"

import { gql } from "@apollo/client"
import { use, useEffect, useState } from "react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import { Button } from "@lana/web/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@lana/web/ui/dialog"

import { ProspectDetailsCard } from "./details"
import { ProspectKycStatus } from "./kyc-status"

import {
  ProspectStatus,
  useGetProspectBasicDetailsQuery,
  useProspectCloseMutation,
} from "@/lib/graphql/generated"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { PublicIdBadge } from "@/components/public-id-badge"

gql`
  fragment ProspectDetailsFragment on Prospect {
    id
    prospectId
    email
    telegramHandle
    status
    kycStatus
    level
    applicantId
    customerType
    createdAt
    publicId
  }

  query GetProspectBasicDetails($id: PublicId!) {
    prospectByPublicId(id: $id) {
      ...ProspectDetailsFragment
    }
  }

  mutation ProspectClose($input: ProspectCloseInput!) {
    prospectClose(input: $input) {
      prospect {
        id
        prospectId
        status
        kycStatus
      }
    }
  }
`

export default function ProspectLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ "prospect-id": string }>
}) {
  const t = useTranslations("Prospects.ProspectDetails.layout")
  const navTranslations = useTranslations("Sidebar.navItems")

  const { "prospect-id": prospectId } = use(params)

  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const [closeDialogOpen, setCloseDialogOpen] = useState(false)

  const { data, loading, error, refetch } = useGetProspectBasicDetailsQuery({
    variables: { id: prospectId },
  })

  const [closeProspect, { loading: closing }] = useProspectCloseMutation()

  const handleClose = async () => {
    if (!data?.prospectByPublicId) return
    try {
      await closeProspect({
        variables: {
          input: { prospectId: data.prospectByPublicId.prospectId },
        },
      })
      setCloseDialogOpen(false)
      toast.success(t("actions.closeSuccess"))
      refetch()
    } catch (err) {
      if (err instanceof Error) {
        toast.error(err.message)
      }
    }
  }

  useEffect(() => {
    if (data?.prospectByPublicId) {
      setCustomLinks([
        { title: navTranslations("prospects"), href: "/prospects" },
        {
          title: <PublicIdBadge publicId={data.prospectByPublicId.publicId} />,
          href: `/prospects/${prospectId}`,
        },
      ])
    }
    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.prospectByPublicId])

  if (loading && !data) return <DetailsPageSkeleton detailItems={3} tabs={1} />
  if (error) return <div className="text-destructive">{t("errors.error")}</div>
  if (!data || !data.prospectByPublicId) return null

  const prospect = data.prospectByPublicId

  return (
    <main className="max-w-7xl m-auto">
      <div className="flex justify-between items-start">
        <ProspectDetailsCard prospect={prospect} />
        {prospect.status === ProspectStatus.Open && (
          <Dialog open={closeDialogOpen} onOpenChange={setCloseDialogOpen}>
            <DialogTrigger asChild>
              <Button variant="destructive" data-testid="close-prospect-btn">
                {t("actions.close")}
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>{t("actions.closeConfirmTitle")}</DialogTitle>
                <DialogDescription>
                  {t("actions.closeConfirmDescription")}
                </DialogDescription>
              </DialogHeader>
              <DialogFooter>
                <Button
                  variant="outline"
                  onClick={() => setCloseDialogOpen(false)}
                >
                  {t("actions.cancel")}
                </Button>
                <Button
                  variant="destructive"
                  onClick={handleClose}
                  disabled={closing}
                  data-testid="confirm-close-prospect-btn"
                >
                  {closing ? t("actions.closing") : t("actions.confirmClose")}
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        )}
      </div>
      <div className="flex flex-col md:flex-row w-full gap-2 my-2">
        <ProspectKycStatus
          prospectId={prospect.prospectId}
          kycStatus={prospect.kycStatus}
          level={prospect.level}
          applicantId={prospect.applicantId}
        />
      </div>
      {children}
    </main>
  )
}
