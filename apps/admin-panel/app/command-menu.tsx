"use client"

import React, { useEffect, useState } from "react"
import { usePathname } from "next/navigation"
import { useTranslations } from "next-intl"

import {
  CheckCircle2,
  CheckSquare,
  Copy,
  FileEdit,
  Plus,
  Settings,
  Shield,
  Wallet,
  XCircle,
  XSquare,
} from "lucide-react"

import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandInput,
  CommandList,
} from "@lana/web/ui/command"

import { CreateDepositDialog } from "./deposits/create"
import { WithdrawalInitiateDialog } from "./withdrawals/initiate"
import { CreateCreditFacilityProposalDialog } from "./credit-facility-proposals/create"
import { CreditFacilityPartialPaymentDialog } from "./credit-facilities/partial-payment"
import { CreditFacilityCollateralUpdateDialog } from "./credit-facilities/collateral-update"
import { CreateUserDialog } from "./users/create"
import { CreateTermsTemplateDialog } from "./terms-templates/create"
import { CreateCommitteeDialog } from "./committees/create"
import { CreditFacilityDisbursalInitiateDialog } from "./disbursals/create"
import ApprovalDialog from "./actions/approve"
import DenialDialog from "./actions/deny"

import { PATH_CONFIGS, useCreateContext } from "./create"

import { UpdateTermsTemplateDialog } from "./terms-templates/[terms-template-id]/update"

import { WithdrawalConfirmDialog } from "./withdrawals/[withdrawal-id]/confirm"
import { WithdrawalCancelDialog } from "./withdrawals/[withdrawal-id]/cancel"
import CommitteeAssignmentDialog from "./policies/[policy-id]/assign-to-committee"
import AddUserCommitteeDialog from "./committees/add-user"

import {
  ApprovalProcessStatus,
  CreditFacilityStatus,
  WithdrawalStatus,
} from "@/lib/graphql/generated"

import { usePublicIdSearch } from "@/hooks/use-public-id-search"
import { SearchResults } from "@/components/command-menu/search-results"
import { MenuSections, groups } from "@/components/command-menu/menu-sections"

const isItemAllowedOnCurrentPath = (
  allowedPaths: (string | RegExp)[],
  currentPath: string,
) => {
  return allowedPaths.some((path) => {
    if (typeof path === "string") {
      return path === currentPath
    } else if (path instanceof RegExp) {
      return path.test(currentPath)
    }
    return false
  })
}

type ApprovalAction = {
  type: "facility" | "withdraw" | "disbursal" | null
  action: "approve" | "deny" | null
}

interface CommandMenuProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

const CommandMenu = ({ open, onOpenChange }: CommandMenuProps) => {
  const pathName = usePathname()
  const t = useTranslations("CommandMenu")

  const setOpen = (value: boolean | ((prev: boolean) => boolean)) => {
    const newValue = typeof value === "function" ? value(open) : value
    onOpenChange(newValue)
    if (!newValue) {
      search.clearSearch()
    }
  }
  const [pages, setPages] = useState<groups>("main")

  const [createDeposit, setCreateDeposit] = useState(false)
  const [createWithdrawal, setCreateWithdrawal] = useState(false)
  const [createFacility, setCreateFacility] = useState(false)
  const [initiateDisbursal, setInitiateDisbursal] = useState(false)
  const [makePayment, setMakePayment] = useState(false)
  const [openCreateUserDialog, setOpenCreateUserDialog] = useState(false)
  const [openCreateTermsTemplateDialog, setOpenCreateTermsTemplateDialog] =
    useState(false)
  const [openDuplicateTermsTemplateDialog, setOpenDuplicateTermsTemplateDialog] =
    useState(false)
  const [openCreateCommitteeDialog, setOpenCreateCommitteeDialog] = useState(false)
  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] = useState(false)
  const [openUpdateTermsTemplateDialog, setOpenUpdateTermsTemplateDialog] =
    useState(false)
  const [openWithdrawalConfirmDialog, setOpenWithdrawalConfirmDialog] = useState(false)
  const [openWithdrawalCancelDialog, setOpenWithdrawalCancelDialog] = useState(false)
  const [openPolicyAssignDialog, setOpenPolicyAssignDialog] = useState(false)
  const [openAddUserCommitteeDialog, setOpenAddUserCommitteeDialog] = useState(false)

  const [approvalAction, setApprovalAction] = useState<ApprovalAction>({
    type: null,
    action: null,
  })

  const search = usePublicIdSearch()
  const getActiveEntity = () => {
    if (facility) return facility
    if (withdraw) return withdraw
    if (disbursal) return disbursal
    return null
  }

  const getActiveEntityType = (): "facility" | "withdraw" | "disbursal" | null => {
    if (facility) return "facility"
    if (withdraw) return "withdraw"
    if (disbursal) return "disbursal"
    return null
  }

  const { customer, facility, termsTemplate, withdraw, policy, committee, disbursal } =
    useCreateContext()

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        setPages("main")
        search.clearSearch()
        setOpen((open) => !open)
      }
      if (e.shiftKey && e.key === "N") {
        const activeElement = document.activeElement?.tagName?.toLowerCase()
        const ignoredElements = ["input", "textarea", "select"]
        if (activeElement && !ignoredElements.includes(activeElement)) {
          e.preventDefault()
          setOpen((open) => !open)
          setPages("navigation")
        }
      }
      if (e.shiftKey && e.key === "A") {
        const activeElement = document.activeElement?.tagName?.toLowerCase()
        const ignoredElements = ["input", "textarea", "select"]
        if (activeElement && !ignoredElements.includes(activeElement)) {
          e.preventDefault()
          setOpen((open) => !open)
          setPages("actions")
        }
      }
    }
    document.addEventListener("keydown", down)
    return () => document.removeEventListener("keydown", down)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [setOpen])

  const menuItems = [
    {
      label: t("actions.createDeposit"),
      icon: Plus,
      action: () => {
        if (!customer) return
        setCreateDeposit(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("actions.createWithdrawal"),
      icon: Plus,
      action: () => {
        if (!customer) return
        setCreateWithdrawal(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("actions.createCreditFacility"),
      icon: Plus,
      action: () => {
        if (!customer) return
        setCreateFacility(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("actions.updateCollateral"),
      icon: Shield,
      action: () => {
        if (!facility) return
        setOpenCollateralUpdateDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CREDIT_FACILITY_DETAILS],
      condition: () =>
        facility?.userCanUpdateCollateral &&
        facility?.status !== CreditFacilityStatus.Closed &&
        facility?.status !== CreditFacilityStatus.Matured,
    },
    {
      label: t("actions.createDisbursal"),
      icon: Plus,
      action: () => {
        if (!facility) return
        setInitiateDisbursal(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CREDIT_FACILITY_DETAILS],
      condition: () => facility?.status === CreditFacilityStatus.Active,
    },
    {
      label: t("actions.makePayment"),
      icon: Wallet,
      action: () => {
        if (!facility) return
        setMakePayment(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CREDIT_FACILITY_DETAILS],
      condition: () => facility?.status === CreditFacilityStatus.Active,
    },
    {
      label: t("actions.createUser"),
      icon: Plus,
      action: () => {
        setOpenCreateUserDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.USERS, PATH_CONFIGS.USER_DETAILS],
    },
    {
      label: t("actions.updateTermsTemplate"),
      icon: FileEdit,
      action: () => {
        if (!termsTemplate) return
        setOpenUpdateTermsTemplateDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.TERMS_TEMPLATE_DETAILS],
      condition: () => termsTemplate?.userCanUpdateTermsTemplate,
    },
    {
      label: t("actions.createTermsTemplate"),
      icon: Plus,
      action: () => {
        setOpenCreateTermsTemplateDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.TERMS_TEMPLATES, PATH_CONFIGS.TERMS_TEMPLATE_DETAILS],
    },
    {
      label: t("actions.duplicateTermsTemplate"),
      icon: Copy,
      action: () => {
        setOpenDuplicateTermsTemplateDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.TERMS_TEMPLATE_DETAILS],
      condition: () => Boolean(termsTemplate),
    },
    {
      label: t("actions.createCommittee"),
      icon: Plus,
      action: () => {
        setOpenCreateCommitteeDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.COMMITTEES, PATH_CONFIGS.COMMITTEE_DETAILS],
    },
    {
      label: t("actions.confirmWithdrawal"),
      icon: CheckCircle2,
      action: () => {
        if (!withdraw) return
        setOpenWithdrawalConfirmDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.WITHDRAWAL_DETAILS],
      condition: () => withdraw?.status === WithdrawalStatus.PendingConfirmation,
    },
    {
      label: t("actions.cancelWithdrawal"),
      icon: XCircle,
      action: () => {
        if (!withdraw) return
        setOpenWithdrawalCancelDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.WITHDRAWAL_DETAILS],
      condition: () => withdraw?.status === WithdrawalStatus.PendingConfirmation,
    },
    {
      label: t("actions.assignCommittee"),
      icon: Settings,
      action: () => {
        if (!policy) return
        setOpenPolicyAssignDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.POLICY_DETAILS],
    },
    {
      label: t("actions.addCommitteeMember"),
      icon: Plus,
      action: () => {
        if (!committee) return
        setOpenAddUserCommitteeDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.COMMITTEE_DETAILS],
    },
    {
      label: t("actions.approve"),
      icon: CheckSquare,
      action: () => {
        setApprovalAction({ type: getActiveEntityType(), action: "approve" })
        setOpen(false)
      },
      allowedPaths: [
        PATH_CONFIGS.CREDIT_FACILITY_DETAILS,
        PATH_CONFIGS.WITHDRAWAL_DETAILS,
        PATH_CONFIGS.DISBURSAL_DETAILS,
      ],
      condition: () => {
        const entity = getActiveEntity()
        return !!(
          entity &&
          "approvalProcess" in entity &&
          entity?.approvalProcess?.status === ApprovalProcessStatus.InProgress &&
          entity?.approvalProcess?.userCanSubmitDecision
        )
      },
    },
    {
      label: t("actions.deny"),
      icon: XSquare,
      action: () => {
        setApprovalAction({ type: getActiveEntityType(), action: "deny" })
        setOpen(false)
      },
      allowedPaths: [
        PATH_CONFIGS.CREDIT_FACILITY_DETAILS,
        PATH_CONFIGS.WITHDRAWAL_DETAILS,
        PATH_CONFIGS.DISBURSAL_DETAILS,
      ],
      condition: () => {
        const entity = getActiveEntity()
        return !!(
          entity &&
          "approvalProcess" in entity &&
          entity?.approvalProcess?.status === ApprovalProcessStatus.InProgress &&
          entity?.approvalProcess?.userCanSubmitDecision
        )
      },
    },
  ]

  const availableItems = menuItems.filter((item) =>
    isItemAllowedOnCurrentPath(item.allowedPaths, pathName),
  )

  return (
    <>
      <CommandDialog open={open} onOpenChange={setOpen}>
        <Command className="rounded-lg border shadow-md">
          <CommandInput
            placeholder={
              pages === "navigation"
                ? t("placeholders.searchNavigation")
                : pages === "actions"
                  ? t("placeholders.searchActions")
                  : t("placeholders.whatDoYouNeed")
            }
            value={search.searchTerm}
            onValueChange={search.handleSearchInputChange}
          />
          <CommandList>
            {search.isSearchMode && search.isSearching && (
              <CommandEmpty>{t("searching")}</CommandEmpty>
            )}
            {search.isSearchMode && search.showNoResults && (
              <CommandEmpty>{t("noResults")}</CommandEmpty>
            )}
            {search.isSearchMode && search.searchResults && (
              <SearchResults
                results={search.searchResults}
                isSearching={search.isSearching}
                showNoResults={search.showNoResults}
                onResultSelect={() => setOpen(false)}
              />
            )}
            {!search.isSearchMode && (
              <>
                <CommandEmpty>{t("noResults")}</CommandEmpty>
                <MenuSections
                  currentPage={pages}
                  availableItems={availableItems}
                  onClose={() => setOpen(false)}
                />
              </>
            )}
          </CommandList>
        </Command>
      </CommandDialog>

      <CreateUserDialog
        openCreateUserDialog={openCreateUserDialog}
        setOpenCreateUserDialog={setOpenCreateUserDialog}
      />
      <CreateTermsTemplateDialog
        openCreateTermsTemplateDialog={openCreateTermsTemplateDialog}
        setOpenCreateTermsTemplateDialog={setOpenCreateTermsTemplateDialog}
      />
      <CreateTermsTemplateDialog
        openCreateTermsTemplateDialog={openDuplicateTermsTemplateDialog}
        setOpenCreateTermsTemplateDialog={setOpenDuplicateTermsTemplateDialog}
        templateToDuplicate={termsTemplate}
      />
      <CreateCommitteeDialog
        openCreateCommitteeDialog={openCreateCommitteeDialog}
        setOpenCreateCommitteeDialog={setOpenCreateCommitteeDialog}
      />

      {customer && customer.depositAccount && (
        <>
          <CreateDepositDialog
            openCreateDepositDialog={createDeposit}
            setOpenCreateDepositDialog={() => setCreateDeposit(false)}
            depositAccountId={customer.depositAccount.depositAccountId}
          />
          <WithdrawalInitiateDialog
            openWithdrawalInitiateDialog={createWithdrawal}
            setOpenWithdrawalInitiateDialog={() => setCreateWithdrawal(false)}
            depositAccountId={customer.depositAccount.depositAccountId}
          />
          <CreateCreditFacilityProposalDialog
            openCreateCreditFacilityProposalDialog={createFacility}
            setOpenCreateCreditFacilityProposalDialog={() => setCreateFacility(false)}
            customerId={customer.customerId}
          />
        </>
      )}

      {facility && (
        <>
          <CreditFacilityDisbursalInitiateDialog
            creditFacilityId={facility.creditFacilityId}
            openDialog={initiateDisbursal}
            setOpenDialog={() => {
              setInitiateDisbursal(false)
            }}
          />
          <CreditFacilityPartialPaymentDialog
            creditFacilityId={facility.creditFacilityId}
            userCanRecordPaymentWithDate={facility.userCanRecordPaymentWithDate}
            openDialog={makePayment}
            setOpenDialog={() => {
              setMakePayment(false)
            }}
          />
          <CreditFacilityCollateralUpdateDialog
            collateralId={facility.collateralId}
            currentCollateral={facility.balance.collateral.btcBalance}
            collateralToMatchInitialCvl={facility.collateralToMatchInitialCvl}
            openDialog={openCollateralUpdateDialog}
            setOpenDialog={setOpenCollateralUpdateDialog}
          />
        </>
      )}

      {termsTemplate && (
        <UpdateTermsTemplateDialog
          termsTemplate={termsTemplate}
          openUpdateTermsTemplateDialog={openUpdateTermsTemplateDialog}
          setOpenUpdateTermsTemplateDialog={() => setOpenUpdateTermsTemplateDialog(false)}
        />
      )}

      {withdraw && (
        <>
          <WithdrawalConfirmDialog
            withdrawalData={withdraw}
            openWithdrawalConfirmDialog={openWithdrawalConfirmDialog}
            setOpenWithdrawalConfirmDialog={() => setOpenWithdrawalConfirmDialog(false)}
          />
          <WithdrawalCancelDialog
            withdrawalData={withdraw}
            openWithdrawalCancelDialog={openWithdrawalCancelDialog}
            setOpenWithdrawalCancelDialog={() => setOpenWithdrawalCancelDialog(false)}
          />
        </>
      )}

      {policy && (
        <CommitteeAssignmentDialog
          policyId={policy.policyId}
          openAssignDialog={openPolicyAssignDialog}
          setOpenAssignDialog={setOpenPolicyAssignDialog}
        />
      )}

      {committee && (
        <AddUserCommitteeDialog
          committeeId={committee.committeeId}
          openAddUserDialog={openAddUserCommitteeDialog}
          setOpenAddUserDialog={setOpenAddUserCommitteeDialog}
        />
      )}

      {approvalAction.type &&
        (() => {
          const currentApprovalProcess =
            approvalAction.type === "facility"
              ? null // CreditFacility doesn't have approvalProcess
              : approvalAction.type === "withdraw"
                ? withdraw?.approvalProcess
                : approvalAction.type === "disbursal"
                  ? disbursal?.approvalProcess
                  : null

          return currentApprovalProcess ? (
            <>
              <ApprovalDialog
                approvalProcess={currentApprovalProcess}
                openApprovalDialog={approvalAction.action === "approve"}
                setOpenApprovalDialog={() =>
                  setApprovalAction({ type: null, action: null })
                }
              />
              <DenialDialog
                approvalProcess={currentApprovalProcess}
                openDenialDialog={approvalAction.action === "deny"}
                setOpenDenialDialog={() =>
                  setApprovalAction({ type: null, action: null })
                }
              />
            </>
          ) : null
        })()}
    </>
  )
}

export { CommandMenu }
