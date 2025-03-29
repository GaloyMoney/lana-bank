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
  Grid2x2,
  Cog,
  ScrollIcon,
} from "lucide-react"
import { useTranslations } from "next-intl"

import type { NavItem } from "./nav-section"

export function useNavItems() {
  const t = useTranslations("Sidebar.navItems")

  const navDashboardItems: NavItem[] = [
    { title: t("dashboard"), url: "/dashboard", icon: Home },
    { title: t("actions"), url: "/actions", icon: TriangleAlert },
  ]

  const navLoansItems: NavItem[] = [
    { title: t("creditFacilities"), url: "/credit-facilities", icon: Grid2x2 },
    { title: t("disbursals"), url: "/disbursals", icon: ClipboardList },
    { title: t("termTemplates"), url: "/terms-templates", icon: LayoutTemplate },
  ]

  const navCustomersItems: NavItem[] = [
    { title: t("customers"), url: "/customers", icon: Users },
  ]

  const navTransactionItems: NavItem[] = [
    { title: t("deposits"), url: "/deposits", icon: ArrowDownCircle },
    { title: t("withdrawals"), url: "/withdrawals", icon: ArrowUpCircle },
  ]

  const navAdminItems: NavItem[] = [
    { title: t("auditLogs"), url: "/audit", icon: BookText },
    { title: t("users"), url: "/users", icon: UserCircle },
  ]

  const navFinanceItems: NavItem[] = [
    { title: t("balanceSheet"), url: "/balance-sheet", icon: PieChart },
    { title: t("cashFlow"), url: "/cash-flow", icon: ArrowUpCircle },
    { title: t("profitAndLoss"), url: "/profit-and-loss", icon: DollarSign },
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
    { title: t("journal"), url: "/journal", icon: ScrollIcon },
    { title: t("trialBalance"), url: "/trial-balance", icon: LineChart },
    { title: t("modules"), url: "/modules", icon: Cog },
  ]

  const allNavItems: NavItem[] = [
    ...navDashboardItems,
    ...navLoansItems,
    ...navCustomersItems,
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
    navCustomersItems,
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
