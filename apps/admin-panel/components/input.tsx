"use client"

import { HTMLInputTypeAttribute, useState } from "react"

type InputProps = {
  label: string
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

const Input: React.FC<InputProps> = ({
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
}) => {
  const [_displayValue, setDisplayValue] = useState(defaultValue)
  let displayValue = _displayValue

  const isNumeric = numeric && type === "number"

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
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

  return (
    <div className="flex flex-col space-y-1 w-full">
      <label className="text-title-sm" htmlFor={name}>
        {label}
      </label>
      <div className="relative">
        {leftNode && (
          <div className="absolute left-3 top-1/2 transform -translate-y-1/2">
            {leftNode}
          </div>
        )}
        <input
          className={`border border-default rounded-md p-2 ${
            leftNode ? "pl-12" : "pl-2"
          } ${
            rightNode ? "pr-10" : "pr-2"
          } focus:outline-none focus:border-primary w-full`}
          type={isNumeric ? "text" : type}
          value={displayValue}
          onChange={handleChange}
          id={name}
          name={name}
          placeholder={placeholder}
          autoFocus={autofocus}
          required={required}
        />
        {rightNode && (
          <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
            {rightNode}
          </div>
        )}
      </div>
    </div>
  )
}

export default Input
