"use client"

import {
  Home,
  TriangleAlert,
  Users,
  UserPlus,
  ClipboardList,
  UserCircle,
  ArrowDownCircle,
  ArrowUpCircle,
  Globe,
  PieChart,
  DollarSign,
  LineChart,
  Users2,
  GanttChart,
  BookText,
  FileText,
  LayoutTemplate,
  Cog,
  ScrollIcon,
  SquareAsterisk,
  ShieldAlert,
  Building,
  Building2,
  FileSignature,
  Clock,
  Calendar,
} from "lucide-react"
import { useTranslations } from "next-intl"

import type { NavItem } from "./nav-section"

import { useAvatarQuery, type VisibleNavigationItems } from "@/lib/graphql/generated"

export function useNavItems() {
  const t = useTranslations("Sidebar.navItems")
  const { data } = useAvatarQuery()

  const navDashboardItems: NavItem[] = [
    { title: t("dashboard"), url: "/dashboard", icon: Home },
    { title: t("actions"), url: "/actions", icon: TriangleAlert },
  ]

  const navCustomerItems: NavItem[] = [
    { title: t("prospects"), url: "/prospects", icon: UserPlus },
    { title: t("customers"), url: "/customers", icon: Users },
  ]

  const navLoansItems: NavItem[] = [
    {
      title: t("creditFacilityProposals"),
      url: "/credit-facility-proposals",
      icon: FileSignature,
    },
    {
      title: t("pendingCreditFacilities"),
      url: "/pending-credit-facilities",
      icon: Clock,
    },
    { title: t("creditFacilities"), url: "/credit-facilities", icon: Building2 },
    { title: t("disbursals"), url: "/disbursals", icon: ClipboardList },
    { title: t("liquidations"), url: "/liquidations", icon: TriangleAlert },
    { title: t("termTemplates"), url: "/terms-templates", icon: LayoutTemplate },
  ]

  const navTransactionItems: NavItem[] = [
    { title: t("depositAccounts"), url: "/deposit-accounts", icon: DollarSign },
    { title: t("deposits"), url: "/deposits", icon: ArrowDownCircle },
    { title: t("withdrawals"), url: "/withdrawals", icon: ArrowUpCircle },
  ]

  const navAdminItems: NavItem[] = [
    { title: t("auditLogs"), url: "/audit", icon: BookText },
    { title: t("users"), url: "/users", icon: UserCircle },
    { title: t("rolesAndPermissions"), url: "/roles-and-permissions", icon: ShieldAlert },
    { title: t("custodians"), url: "/custodians", icon: Building },
    { title: t("configurations"), url: "/configurations", icon: Cog },
  ]

  const navFinanceItems: NavItem[] = [
    { title: t("balanceSheet"), url: "/balance-sheet", icon: PieChart },
    { title: t("profitAndLoss"), url: "/profit-and-loss", icon: DollarSign },
    { title: t("trialBalance"), url: "/trial-balance", icon: LineChart },
    {
      title: t("regulatoryReporting"),
      url: "/regulatory-reporting",
      icon: FileText,
    },
  ]

  const navGovernanceItems: NavItem[] = [
    { title: t("committees"), url: "/committees", icon: Users2 },
    { title: t("policies"), url: "/policies", icon: GanttChart },
  ]

  const navAccountingItems: NavItem[] = [
    { title: t("chartOfAccounts"), url: "/chart-of-accounts", icon: Globe },
    { title: t("fiscalYears"), url: "/fiscal-years", icon: Calendar },
    { title: t("ledgerAccounts"), url: "/ledger-accounts", icon: BookText },
    { title: t("ledgerTransactions"), url: "/ledger-transactions", icon: FileText },
    { title: t("journal"), url: "/journal", icon: ScrollIcon },
    { title: t("modules"), url: "/modules", icon: Cog },
    {
      title: t("transactionTemplates"),
      url: "/transaction-templates",
      icon: SquareAsterisk,
    },
  ]

  const visibility = data?.me?.visibleNavigationItems

  const filteredDashboardItems = navDashboardItems
  const filteredCustomerItems = filterByVisibility(navCustomerItems, visibility)
  const filteredLoansItems = filterByVisibility(navLoansItems, visibility)
  const filteredTransactionItems = filterByVisibility(navTransactionItems, visibility)
  const filteredAdminItems = filterByVisibility(navAdminItems, visibility)
  const filteredFinanceItems = filterByVisibility(navFinanceItems, visibility)
  const filteredGovernanceItems = filterGovernanceItems(navGovernanceItems, visibility)
  const filteredAccountingItems = filterByVisibility(navAccountingItems, visibility)

  const allNavItems: NavItem[] = [
    ...filteredDashboardItems,
    ...filteredCustomerItems,
    ...filteredLoansItems,
    ...filteredTransactionItems,
    ...filteredAdminItems,
    ...filteredFinanceItems,
    ...filteredGovernanceItems,
    ...filteredAccountingItems,
  ]

  const navItemsByUrl = new Map<string, NavItem>()
  allNavItems.forEach((item) => {
    navItemsByUrl.set(item.url, item)
  })

  const findNavItemByUrl = (url: string): NavItem | undefined => {
    return navItemsByUrl.get(url)
  }

  return {
    navDashboardItems: filteredDashboardItems,
    navCustomerItems: filteredCustomerItems,
    navLoansItems: filteredLoansItems,
    navTransactionItems: filteredTransactionItems,
    navAdminItems: filteredAdminItems,
    navFinanceItems: filteredFinanceItems,
    navGovernanceItems: filteredGovernanceItems,
    navAccountingItems: filteredAccountingItems,

    allNavItems,
    navItemsByUrl,
    findNavItemByUrl,
  }
}

function filterByVisibility(
  items: NavItem[],
  visibility: VisibleNavigationItems | undefined,
): NavItem[] {
  if (!visibility) return items

  const urlVisibilityMap: Record<string, (v: VisibleNavigationItems) => boolean> = {
    "/prospects": (v) => v.customer,
    "/customers": (v) => v.customer,
    "/credit-facility-proposals": (v) => v.creditFacilities,
    "/pending-credit-facilities": (v) => v.creditFacilities,
    "/credit-facilities": (v) => v.creditFacilities,
    "/disbursals": (v) => v.creditFacilities,
    "/liquidations": (v) => v.creditFacilities,
    "/terms-templates": (v) => v.term,
    "/deposit-accounts": (v) => v.deposit,
    "/deposits": (v) => v.deposit,
    "/withdrawals": (v) => v.withdraw,
    "/audit": (v) => v.audit,
    "/users": (v) => v.user,
    "/roles-and-permissions": (v) => v.user,
    "/custodians": (v) => v.user,
    "/configurations": (v) => v.user,
    "/balance-sheet": (v) => v.financials,
    "/profit-and-loss": (v) => v.financials,
    "/trial-balance": (v) => v.financials,
    "/regulatory-reporting": (v) => v.financials,
    "/chart-of-accounts": (v) => v.financials,
    "/fiscal-years": (v) => v.financials,
    "/ledger-accounts": (v) => v.financials,
    "/ledger-transactions": (v) => v.financials,
    "/journal": (v) => v.financials,
    "/modules": (v) => v.financials,
    "/transaction-templates": (v) => v.financials,
  }

  return items.filter((item) => {
    const checkVisibility = urlVisibilityMap[item.url]
    if (!checkVisibility) return true
    return checkVisibility(visibility)
  })
}

function filterGovernanceItems(
  items: NavItem[],
  visibility: VisibleNavigationItems | undefined,
): NavItem[] {
  if (!visibility) return items

  const governance = visibility.governance

  return items.filter((item) => {
    switch (item.url) {
      case "/committees":
        return governance.committee
      case "/policies":
        return governance.policy
      default:
        return true
    }
  })
}
