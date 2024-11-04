import { useState, useRef, useEffect } from "react"
import { motion, AnimatePresence } from "framer-motion"

export type TimeRange =
  | "LastDay"
  | "LastWeek"
  | "LastMonth"
  | "LastQuarter"
  | "LastYear"
  | "AllTime"

type TimeRangeSelectProps = {
  range: TimeRange
  setRange: React.Dispatch<React.SetStateAction<TimeRange>>
}

const TimeRangeSelect: React.FC<TimeRangeSelectProps> = ({ range, setRange }) => {
  const [isOpen, setIsOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    document.addEventListener("mousedown", handleClickOutside)
    return () => document.removeEventListener("mousedown", handleClickOutside)
  }, [])

  const timeRanges: { label: string; value: TimeRange }[] = [
    { label: "Last Day", value: "LastDay" },
    { label: "Last Week", value: "LastWeek" },
    { label: "Last Month", value: "LastMonth" },
    { label: "Last Quarter", value: "LastQuarter" },
    { label: "Last Year", value: "LastYear" },
    { label: "All Time", value: "AllTime" },
  ]

  const selectedLabel = timeRanges.find((tr) => tr.value === range)?.label || ""

  return (
    <div
      className="absolute top-0 right-0 inline-block text-left self-end -mr-[1px] -mt-[1px] z-10"
      ref={containerRef}
    >
      <button
        onClick={() => setIsOpen((prev) => !prev)}
        className="inline-flex justify-between items-center px-2 py-1 text-title-xs font-medium border bg-page rounded-tl-none rounded-br-none rounded-md hover:bg-gray-50 focus:outline-none"
      >
        {selectedLabel}
        <svg
          className={`w-5 h-5 ml-2 transition-transform duration-200 ${
            isOpen ? "transform rotate-180" : ""
          }`}
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          aria-hidden="true"
        >
          <path
            fillRule="evenodd"
            d="M5.23 7.21a.75.75 0 011.06.02L10 10.97l3.71-3.74a.75.75 0 111.08 1.04l-4.25 4.25a.75.75 0 01-1.08 0L5.21 8.27a.75.75 0 01.02-1.06z"
            clipRule="evenodd"
          />
        </svg>
      </button>

      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, scaleY: 0.95, originY: 0 }}
            animate={{ opacity: 1, scaleY: 1 }}
            exit={{ opacity: 0, scaleY: 0.95 }}
            transition={{ duration: 0.2 }}
            className="absolute right-0 w-48 mt-2 origin-top-right bg-page border divide-y divide-gray-100 rounded-md shadow-lg outline-none z-10"
          >
            <div className="py-1">
              {timeRanges.map((tr) => (
                <button
                  key={tr.value}
                  onClick={() => {
                    setRange(tr.value)
                    setIsOpen(false)
                  }}
                  className={`block w-full text-left px-4 py-2 text-sm ${
                    range === tr.value
                      ? "bg-gray-100 text-gray-900"
                      : "text-gray-700 hover:bg-gray-100"
                  }`}
                >
                  {tr.label}
                </button>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}

export default TimeRangeSelect
