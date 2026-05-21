import { cn } from "../../lib/utils";
import React from "react";

interface Tab {
    key:   string;
    label: string;
    icon?: React.ReactNode;
    count?: number;
}

interface TabsProps {
    tabs:      Tab[];
    active:    string;
    onChange:  (key: string) => void;
    className?: string;
}

export function Tabs({ tabs, active, onChange, className }: TabsProps) {
    return (
        <div className={cn("flex border-b border-gray-200 dark:border-gray-800", className)}>
            {tabs.map((tab) => (
                <button
                    key={tab.key}
                    onClick={() => onChange(tab.key)}
                    className={cn(
                        "flex items-center gap-1.5 px-4 py-2.5 text-sm font-medium border-b-2 -mb-px transition-colors",
                        active === tab.key
                            ? "border-blue-500 text-blue-600 dark:text-blue-400"
                            : "border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                    )}
                >
                    {tab.icon}
                    {tab.label}
                    {tab.count !== undefined && (
                        <span className={cn(
                            "ml-1 text-xs px-1.5 py-0.5 rounded-full",
                            active === tab.key
                                ? "bg-blue-100 text-blue-600 dark:bg-blue-900/40 dark:text-blue-400"
                                : "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400"
                        )}>
              {tab.count}
            </span>
                    )}
                </button>
            ))}
        </div>
    );
}