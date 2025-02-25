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
    { title: t("policies"), url: "/policies", icon: GanttChart },
  ]

  const navTransactionItems: NavItem[] = [
    { title: t("deposits"), url: "/deposits", icon: ArrowDownCircle },
    { title: t("withdrawals"), url: "/withdrawals", icon: ArrowUpCircle },
  ]

  const navAdminItems: NavItem[] = [
    { title: t("auditLogs"), url: "/audit", icon: BookText },
    { title: t("committees"), url: "/committees", icon: Users2 },
    { title: t("chartOfAccounts"), url: "/chart-of-accounts", icon: Globe },
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
    { title: t("trialBalance"), url: "/trial-balance", icon: LineChart },
  ]

  return {
    navDashboardItems,
    navLoansItems,
    navCustomersItems,
    navTransactionItems,
    navAdminItems,
    navFinanceItems,
  }
}
