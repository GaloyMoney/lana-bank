"use client"

import classNames from "classnames"
import { HTMLInputTypeAttribute, useState, forwardRef } from "react"

type InputProps = {
  label?: string
  type: HTMLInputTypeAttribute
  defaultValue?: string
  onChange?: (text: string) => void
  name?: string
  placeholder?: string
  autofocus?: boolean
  required?: boolean
  leftNode?: React.ReactNode
  rightNode?: React.ReactNode
  // If type is 'number' and numeric is set, the displayed number will contain commas for thousands separators
  numeric?: boolean
}

const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      label,
      type,
      // eslint-disable-next-line no-empty-function
      onChange = () => {},
      defaultValue = "",
      placeholder = "",
      name,
      numeric = false,
      autofocus = false,
      required = false,
      leftNode,
      rightNode,
    },
    ref,
  ) => {
    const [_displayValue, setDisplayValue] = useState(defaultValue)
    let displayValue = _displayValue

    const isNumeric = numeric && type === "number"

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      "use client"
      let value = e.target.value

      if (isNumeric) {
        value = value.replaceAll(",", "").replace(/\D/g, "")
      }

      setDisplayValue(value)
      onChange(value)
    }

    if (isNumeric && _displayValue !== "") {
      displayValue = Number(_displayValue).toLocaleString("en-US")
    }

    const classes = classNames(
      "border-2 border-default rounded-md text-body-md placeholder:text-placeholder p-2 w-full focus:outline-none focus:border-primary box-border",
      {
        "pl-10": leftNode,
        "pl-2": !leftNode,
        "pr-10": rightNode,
        "pr-2": !rightNode,
      },
    )

    return (
      <div className="flex flex-col space-y-1 w-full">
        {label && (
          <label className="text-title-sm" htmlFor={name}>
            {label}
          </label>
        )}
        <div className="relative">
          {leftNode && (
            <div className="absolute left-3 top-1/2 transform -translate-y-1/2">
              {leftNode}
            </div>
          )}
          <input
            ref={ref}
            className={classes}
            type={isNumeric ? "text" : type}
            value={displayValue}
            onChange={handleChange}
            id={name}
            name={name}
            placeholder={placeholder}
            autoFocus={autofocus}
            required={required}
          />
          {rightNode && !displayValue && (
            <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
              {rightNode}
            </div>
          )}
        </div>
      </div>
    )
  },
)

Input.displayName = "Input"

export default Input
