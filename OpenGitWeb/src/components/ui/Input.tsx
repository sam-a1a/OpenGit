import { cn } from "../../lib/utils";
import React from "react";

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
    label?:   string;
    error?:   string;
    hint?:    string;
    icon?:    React.ReactNode;
}

export function Input({ label, error, hint, icon, className, id, ...props }: InputProps) {
    const inputId = id ?? label?.toLowerCase().replace(/\s/g, "-");

    return (
        <div className="w-full">
            {label && (
                <label
                    htmlFor={inputId}
                    className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                >
                    {label}
                </label>
            )}
            <div className="relative">
                {icon && (
                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-gray-400">
                        {icon}
                    </div>
                )}
                <input
                    id={inputId}
                    className={cn(
                        "w-full rounded-md border text-sm transition-colors",
                        "bg-white dark:bg-gray-900",
                        "text-gray-900 dark:text-gray-100",
                        "placeholder-gray-400 dark:placeholder-gray-600",
                        "focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                        error
                            ? "border-red-500"
                            : "border-gray-300 dark:border-gray-700",
                        icon ? "pl-10 pr-3 py-2" : "px-3 py-2",
                        className
                    )}
                    {...props}
                />
            </div>
            {error && <p className="mt-1 text-xs text-red-500">{error}</p>}
            {hint && !error && <p className="mt-1 text-xs text-gray-500">{hint}</p>}
        </div>
    );
}