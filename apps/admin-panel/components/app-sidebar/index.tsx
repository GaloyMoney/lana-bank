"use client"

import type { ComponentProps } from "react"

import Link from "next/link"
import { useTranslations } from "next-intl"

import {
  Sidebar,
  SidebarContent,
  SidebarHeader,
  SidebarFooter,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from "@lana/web/ui/sidebar"

import { NavSection } from "./nav-section"
import { useNavItems } from "./nav-items"

import { Logo } from "@/components/logo"

interface AppSidebarProps extends ComponentProps<typeof Sidebar> {
  appVersion?: string
}

export function AppSidebar({ appVersion, ...props }: AppSidebarProps) {
  const t = useTranslations("Sidebar")

  const {
    navDashboardItems,
    navCustomerItems,
    navLoansItems,
    navTransactionItems,
    navAdminItems,
    navFinanceItems,
    navAccountingItems,
    navGovernanceItems,
  } = useNavItems()

  return (
    <Sidebar variant="sidebar" {...props}>
      <SidebarHeader>
        <Link href="/dashboard" className="flex items-center gap-2 px-2 py-1">
          <Logo width={18} className="shrink-0" />
          <span className="truncate font-semibold text-lg">{t("footer.appName")}</span>
        </Link>
      </SidebarHeader>
      <SidebarContent>
        <NavSection items={navDashboardItems} />
        <NavSection items={navCustomerItems} label={t("labels.customers")} />
        <NavSection items={navLoansItems} label={t("labels.loans")} />
        <NavSection items={navTransactionItems} label={t("labels.transactions")} />
        <NavSection items={navAdminItems} label={t("labels.administration")} />
        <NavSection items={navGovernanceItems} label={t("labels.governance")} />
        <NavSection items={navAccountingItems} label={t("labels.accounting")} />
        <NavSection items={navFinanceItems} label={t("labels.financialReports")} />
      </SidebarContent>
      <SidebarFooter>
        <div className="px-3 pb-2">
          <span className="text-[10px] text-muted-foreground">
            {t("footer.version", { version: appVersion || "0.0.0-dev" })}
          </span>
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}
export * from "./nav-section"
export * from "./user-block"
export * from "./nav-items"
