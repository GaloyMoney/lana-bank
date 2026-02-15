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

import { UserBlock } from "./user-block"
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
    <Sidebar variant="inset" {...props}>
      <SidebarHeader>
        <UserBlock />
      </SidebarHeader>
      <SidebarContent className="mt-4">
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
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" asChild tabIndex={-1}>
              <Link href="/dashboard">
                <div className="flex aspect-square size-10 items-center justify-center rounded-lg">
                  <Logo className="size-10" />
                </div>
                <div className="grid flex-1 text-left text-sm leading-tight">
                  <span className="truncate font-semibold">{t("footer.appName")}</span>
                  <span className="truncate text-xs">
                    {t("footer.version", { version: appVersion || "0.0.0-dev" })}
                  </span>
                </div>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  )
}
export * from "./nav-section"
export * from "./user-block"
export * from "./nav-items"
