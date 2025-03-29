"use client";

import React from "react";
import Link from "next/link";
import { ArrowRight } from "lucide-react";
import { useRouter } from "next/navigation";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@lana/web/ui/table";
import { Button } from "@lana/web/ui/button";
import { Skeleton } from "@lana/web/ui/skeleton";
import { Card } from "@lana/web/ui/card";
import { useBreakpointDown } from "@lana/web/hooks";
import { cn } from "@lana/web/utils";

export type Column<T> = {
  [K in keyof T]: {
    key: K;
    header: string | React.ReactNode;
    width?: string;
    align?: "left" | "center" | "right";
    render?: (value: T[K], record: T) => React.ReactNode;
  };
}[keyof T];

interface DataTableProps<T> {
  data: T[];
  columns: Column<T>[];
  className?: string;
  headerClassName?: string;
  rowClassName?: string | ((item: T, index: number) => string);
  cellClassName?: string | ((column: Column<T>, item: T) => string);
  onRowClick?: (item: T) => void;
  emptyMessage?: React.ReactNode;
  loading?: boolean;
  navigateTo?: (record: T) => string | null;
}

const DataTable = <T,>({
  data,
  columns,
  className,
  headerClassName,
  rowClassName,
  cellClassName,
  onRowClick,
  emptyMessage = "No data to display",
  loading = false,
  navigateTo,
}: DataTableProps<T>) => {
  const isMobile = useBreakpointDown("md");
  const router = useRouter();

  const getNavigationUrl = (item: T): string | null => {
    return navigateTo ? navigateTo(item) : null;
  };

  const shouldShowNavigation = (item: T): boolean => {
    if (!navigateTo) return false;
    const url = getNavigationUrl(item);
    return url !== null && url !== "";
  };

  if (loading && !data.length) {
    return isMobile ? (
      <div className="space-y-4" data-testid="loading-skeleton">
        {Array.from({ length: 5 }).map((_, idx) => (
          <Card key={idx} className="p-4 space-y-3">
            {columns.map((_, colIndex) => (
              <Skeleton key={colIndex} className="h-4 w-full" />
            ))}
          </Card>
        ))}
      </div>
    ) : (
      <div
        className="overflow-x-auto border rounded-md"
        data-testid="loading-skeleton"
      >
        <Table className={cn("table-fixed w-full", className)}>
          <TableHeader className="bg-secondary [&_tr:hover]:!bg-secondary">
            <TableRow className={headerClassName}>
              {columns.map((column, index) => (
                <TableHead
                  key={index}
                  className={cn(
                    column.align === "center" && "!text-center",
                    column.align === "right" && "!text-right"
                  )}
                  style={{ width: column.width }}
                >
                  <div
                    className={cn(
                      "flex items-center space-x-2",
                      column.align === "center" && "justify-center",
                      column.align === "right" && "justify-end"
                    )}
                  >
                    <span>{column.header}</span>
                  </div>
                </TableHead>
              ))}
              {navigateTo && <TableHead className="w-24" />}
            </TableRow>
          </TableHeader>
          <TableBody>
            {Array.from({ length: 5 }).map((_, rowIndex) => (
              <TableRow key={rowIndex}>
                {columns.map((_, colIndex) => (
                  <TableCell key={colIndex}>
                    <Skeleton className="h-9 w-full" />
                  </TableCell>
                ))}
                {navigateTo && (
                  <TableCell>
                    <Skeleton className="h-9 w-full" />
                  </TableCell>
                )}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    );
  }

  if (!data.length) {
    return <div className="text-sm">{emptyMessage}</div>;
  }

  if (isMobile) {
    return (
      <div className="space-y-4">
        {data.map((item, index) => (
          <Card
            key={index}
            className={cn(
              "p-4 space-y-3",
              typeof rowClassName === "function"
                ? rowClassName(item, index)
                : rowClassName,
              onRowClick && "cursor-pointer"
            )}
            onClick={() => onRowClick?.(item)}
          >
            {columns.map((column, colIndex) => {
              const hasHeader =
                typeof column.header === "string" &&
                column.header.trim() !== "";
              return (
                <div
                  key={colIndex}
                  className={cn(
                    "flex items-start gap-4",
                    hasHeader ? "justify-between" : "w-full",
                    typeof cellClassName === "function"
                      ? cellClassName(column, item)
                      : cellClassName
                  )}
                >
                  {hasHeader && (
                    <div className="text-sm font-medium text-muted-foreground">
                      {column.header}
                    </div>
                  )}
                  <div
                    className={cn(
                      "text-sm",
                      !hasHeader && "w-full",
                      column.align === "center" && "!text-center",
                      column.align === "right" && "!text-right"
                    )}
                  >
                    {column.render
                      ? column.render(item[column.key], item)
                      : String(item[column.key])}
                  </div>
                </div>
              );
            })}
            {shouldShowNavigation(item) && (
              <div className="pt-2">
                <Link href={getNavigationUrl(item) || ""}>
                  <Button
                    variant="outline"
                    className="w-full flex items-center justify-center"
                  >
                    View
                    <ArrowRight className="h-4 w-4" />
                  </Button>
                </Link>
              </div>
            )}
          </Card>
        ))}
      </div>
    );
  }

  return (
    <div className="overflow-x-auto border rounded-md">
      <Table className={cn("table-fixed w-full", className)}>
        <TableHeader className="bg-secondary [&_tr:hover]:!bg-secondary">
          <TableRow className={headerClassName}>
            {columns.map((column, index) => (
              <TableHead
                key={index}
                className={cn(
                  column.align === "center" && "!text-center",
                  column.align === "right" && "!text-right"
                )}
                style={{ width: column.width }}
              >
                <div
                  className={cn(
                    "flex items-center space-x-2",
                    column.align === "center" && "justify-center",
                    column.align === "right" && "justify-end"
                  )}
                >
                  <span>{column.header}</span>
                </div>
              </TableHead>
            ))}
            {navigateTo && <TableHead className="w-24" />}
          </TableRow>
        </TableHeader>
        <TableBody>
          {data.map((item, rowIndex) => (
            <TableRow
              key={rowIndex}
              onClick={() => onRowClick?.(item)}
              className={cn(
                typeof rowClassName === "function"
                  ? rowClassName(item, rowIndex)
                  : rowClassName,
                onRowClick && "cursor-pointer",
                "hover:bg-muted/50 transition-colors"
              )}
            >
              {columns.map((column, colIndex) => (
                <TableCell
                  key={colIndex}
                  className={cn(
                    "whitespace-normal break-words",
                    column.align === "center" && "!text-center",
                    column.align === "right" && "!text-right",
                    typeof cellClassName === "function"
                      ? cellClassName(column, item)
                      : cellClassName
                  )}
                >
                  <div
                    className={cn(
                      "w-full",
                      column.align === "center" && "!text-center",
                      column.align === "right" && "!text-right"
                    )}
                  >
                    {column.render
                      ? column.render(item[column.key], item)
                      : String(item[column.key])}
                  </div>
                </TableCell>
              ))}
              {shouldShowNavigation(item) && (
                <TableCell>
                  <Link href={getNavigationUrl(item) || ""}>
                    <Button
                      variant="outline"
                      className="w-full flex items-center justify-between"
                    >
                      View
                      <ArrowRight />
                    </Button>
                  </Link>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
};

export default DataTable;
