"use client"

import { LogOut, Globe, ChevronsUpDown } from "lucide-react"
import { useLocale, useTranslations } from "next-intl"

import { Skeleton } from "@lana/web/ui/skeleton"
import { Badge } from "@lana/web/ui/badge"
import { Button } from "@lana/web/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@lana/web/ui/dropdown-menu"

import { ID } from "@/components/id"
import { useAvatarQuery } from "@/lib/graphql/generated"
import { useLogout } from "@/app/auth/session"

export function HeaderUserMenu() {
  const { logout } = useLogout()
  const { data, loading } = useAvatarQuery()
  const locale = useLocale()
  const t = useTranslations("Sidebar.footer")

  const switchLocale = (newLocale: string) => {
    document.cookie = `NEXT_LOCALE=${newLocale};path=/`
    window.location.reload()
  }

  if (loading && !data) {
    return <Skeleton className="h-8 w-8 rounded-lg" />
  }

  if (!data?.me.user) return null
  const { email, role, userId } = data.me.user
  const userName = email.split("@")[0]
  const initials = userName[0].toUpperCase()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="flex items-center gap-2 px-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
            <span className="text-sm font-medium">{initials}</span>
          </div>
          <span className="hidden md:inline text-sm font-medium capitalize">
            {userName}
          </span>
          <ChevronsUpDown className="h-4 w-4 text-muted-foreground/70" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="min-w-56" align="end" sideOffset={4}>
        <DropdownMenuLabel className="font-normal">
          <div className="flex flex-col gap-2 p-1">
            <div className="flex flex-wrap gap-1">
              {role && (
                <Badge variant="secondary" className="capitalize">
                  {role.name}
                </Badge>
              )}
            </div>
            <div className="text-sm">{email}</div>
            <ID type="Your" id={userId} />
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuLabel className="font-normal text-sm">
          {t("language")}
        </DropdownMenuLabel>
        <DropdownMenuItem
          onClick={() => switchLocale("en")}
          className={locale === "en" ? "bg-accent" : ""}
        >
          <Globe className="mr-2 h-4 w-4" />
          English
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={() => switchLocale("es")}
          className={locale === "es" ? "bg-accent" : ""}
        >
          <Globe className="mr-2 h-4 w-4" />
          Español
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          className="text-destructive focus:text-destructive cursor-pointer"
          onClick={logout}
        >
          <LogOut className="mr-2 h-4 w-4" />
          {t("logOut")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
