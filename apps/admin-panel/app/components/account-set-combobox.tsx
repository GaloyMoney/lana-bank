"use client"

import { Button } from "@lana/web/ui/button"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@lana/web/ui/command"
import { Popover, PopoverContent, PopoverTrigger } from "@lana/web/ui/popover"
import { Tooltip, TooltipContent, TooltipTrigger } from "@lana/web/ui/tooltip"
import { cn } from "@lana/web/utils"
import { CheckIcon, ChevronsUpDownIcon } from "lucide-react"
import { useState } from "react"

export type AccountSetOptionBase = {
  accountSetId: string
  code: string
  name: string
}

export type AccountSetComboboxProps<TOption extends AccountSetOptionBase> = {
  id?: string
  value: string
  options: TOption[]
  onChange: (value: string) => void
  disabled?: boolean
  placeholder: string
  searchPlaceholder: string
  emptyLabel: string
}

export const getOptionLabel = (option: AccountSetOptionBase) => {
  return option.code ? `${option.name} - ${option.code}` : option.name
}

export function AccountSetCombobox<TOption extends AccountSetOptionBase>({
  id,
  value,
  options,
  onChange,
  disabled,
  placeholder,
  searchPlaceholder,
  emptyLabel,
}: AccountSetComboboxProps<TOption>) {
  const [open, setOpen] = useState(false)
  const selectedOption = options.find((option) => option.code === value)
  const displayValue = selectedOption ? getOptionLabel(selectedOption) : value
  const displayText = displayValue || placeholder
  const showTooltip = Boolean(displayValue)

  return (
    <Popover open={open} onOpenChange={setOpen} modal>
      <PopoverTrigger asChild>
        <Button
          id={id}
          variant="outline"
          role="combobox"
          aria-expanded={open}
          disabled={disabled}
          className="w-full justify-between font-normal"
        >
          {showTooltip ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="min-w-0 flex-1 truncate text-left uppercase">
                  {displayText}
                </span>
              </TooltipTrigger>
              <TooltipContent className="uppercase">{displayValue}</TooltipContent>
            </Tooltip>
          ) : (
            <span className="min-w-0 flex-1 truncate text-left text-muted-foreground uppercase">
              {displayText}
            </span>
          )}
          <ChevronsUpDownIcon className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className="w-[--radix-popover-trigger-width] p-0"
        align="start"
      >
        <Command>
          <CommandInput placeholder={searchPlaceholder} />
          <CommandList>
            <CommandEmpty>{emptyLabel}</CommandEmpty>
            <CommandGroup>
              {options.map((option) => {
                const optionLabel = getOptionLabel(option)
                const keywords = [option.name, option.code].filter(
                  (keyword): keyword is string => Boolean(keyword),
                )

                return (
                  <CommandItem
                    key={option.accountSetId}
                    value={option.code}
                    keywords={keywords}
                    onSelect={() => {
                      onChange(option.code)
                      setOpen(false)
                    }}
                  >
                    <CheckIcon
                      className={cn(
                        "mr-2 h-4 w-4",
                        value === option.code ? "opacity-100" : "opacity-0",
                      )}
                    />
                    <span className="truncate uppercase">{optionLabel}</span>
                  </CommandItem>
                )
              })}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}

export const formatOptionValue = (
  value: string,
  options: AccountSetOptionBase[],
) => {
  if (!value) return ""
  const match = options.find((option) => option.code === value)
  if (!match) return value
  return getOptionLabel(match)
}
