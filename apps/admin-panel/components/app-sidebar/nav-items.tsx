"use client"

import {
  Home,
  TriangleAlert,
  Users,
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

export function useNavItems() {
  const t = useTranslations("Sidebar.navItems")

  const navDashboardItems: NavItem[] = [
    { title: t("dashboard"), url: "/dashboard", icon: Home },
    { title: t("actions"), url: "/actions", icon: TriangleAlert },
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

  const allNavItems: NavItem[] = [
    ...navDashboardItems,
    ...navLoansItems,
    ...navTransactionItems,
    ...navAdminItems,
    ...navFinanceItems,
    ...navGovernanceItems,
    ...navAccountingItems,
  ]

  const navItemsByUrl = new Map<string, NavItem>()
  allNavItems.forEach((item) => {
    navItemsByUrl.set(item.url, item)
  })

  const findNavItemByUrl = (url: string): NavItem | undefined => {
    return navItemsByUrl.get(url)
  }

  return {
    navDashboardItems,
    navLoansItems,
    navTransactionItems,
    navAdminItems,
    navFinanceItems,
    navGovernanceItems,
    navAccountingItems,

    allNavItems,
    navItemsByUrl,
    findNavItemByUrl,
  }
}
