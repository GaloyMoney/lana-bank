"use client"

import { useRouter } from "next/navigation"
import { useTranslations } from "next-intl"
import type { LucideIcon } from "lucide-react"

import { CommandGroup, CommandItem, CommandSeparator } from "@lana/web/ui/command"

import { useNavItems } from "@/components/app-sidebar/nav-items"

type MenuItem = {
  label: string
  icon: LucideIcon
  action: () => void
  allowedPaths: (string | RegExp)[]
  condition?: () => boolean | undefined
}

export type groups = "main" | "navigation" | "actions"

interface MenuSectionsProps {
  currentPage: groups
  availableItems: MenuItem[]
  onClose: () => void
}

function KeyboardControlHeading({
  heading,
  combination,
}: {
  heading: string
  combination: string
}) {
  return (
    <div className="flex items-center justify-between">
      <span>{heading}</span>
      <kbd className="ml-auto pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
        <span className="text-xs">{combination}</span>
      </kbd>
    </div>
  )
}

export function MenuSections({
  currentPage,
  availableItems,
  onClose,
}: MenuSectionsProps) {
  const router = useRouter()
  const t = useTranslations("CommandMenu")

  const { allNavItems } = useNavItems()

  if (currentPage === "main") {
    return (
      <>
        {availableItems.length > 0 && (
          <>
            <CommandSeparator />
            <CommandGroup
              heading={
                <KeyboardControlHeading
                  heading={t("headings.availableActions")}
                  combination="Shift + A"
                />
              }
            >
              {availableItems.map((item) => (
                <CommandItem
                  key={item.label}
                  disabled={item.condition && !item.condition()}
                  onSelect={() => {
                    item.action()
                  }}
                >
                  <item.icon className="mr-2 h-4 w-4" />
                  {item.label}
                </CommandItem>
              ))}
            </CommandGroup>
          </>
        )}
        <CommandSeparator />
        <CommandGroup
          heading={
            <KeyboardControlHeading
              heading={t("headings.navigation")}
              combination="Shift + N"
            />
          }
        >
          {allNavItems.map((item) => (
            <CommandItem
              key={item.url}
              onSelect={() => {
                router.push(item.url)
                onClose()
              }}
              className="flex items-center gap-2"
            >
              <item.icon className="h-4 w-4" />
              <span>{item.title}</span>
            </CommandItem>
          ))}
        </CommandGroup>
      </>
    )
  }

  if (currentPage === "actions") {
    return (
      <CommandGroup heading={t("headings.availableActions")}>
        {availableItems.map((item) => (
          <CommandItem
            key={item.label}
            disabled={item.condition && !item.condition()}
            onSelect={() => {
              item.action()
            }}
          >
            <item.icon className="mr-2 h-4 w-4" />
            {item.label}
          </CommandItem>
        ))}
      </CommandGroup>
    )
  }

  if (currentPage === "navigation") {
    return (
      <CommandGroup heading={t("headings.navigation")}>
        {allNavItems.map((item) => (
          <CommandItem
            key={item.url}
            onSelect={() => {
              onClose()
              router.push(item.url)
            }}
            className="flex items-center gap-2"
          >
            <item.icon className="h-4 w-4" />
            <span>{item.title}</span>
          </CommandItem>
        ))}
      </CommandGroup>
    )
  }

  return null
}
