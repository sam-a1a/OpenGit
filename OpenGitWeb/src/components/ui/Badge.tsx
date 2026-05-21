import { cn } from "../../lib/utils";

interface BadgeProps {
    children:  React.ReactNode;
    variant?:  "default" | "success" | "warning" | "danger" | "info" | "purple";
    size?:     "sm" | "md";
    dot?:      boolean;
    className?: string;
}

export function Badge({
                          children,
                          variant  = "default",
                          size     = "md",
                          dot      = false,
                          className,
                      }: BadgeProps) {
    const variants = {
        default: "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300",
        success: "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400",
        warning: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400",
        danger:  "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400",
        info:    "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400",
        purple:  "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400",
    };

    const sizes = {
        sm: "text-xs px-1.5 py-0.5",
        md: "text-xs px-2.5 py-1",
    };

    return (
        <span className={cn(
            "inline-flex items-center gap-1 font-medium rounded-full",
            variants[variant],
            sizes[size],
            className
        )}>
      {dot && <span className={cn("w-1.5 h-1.5 rounded-full", {
          "bg-gray-500":   variant === "default",
          "bg-green-500":  variant === "success",
          "bg-yellow-500": variant === "warning",
          "bg-red-500":    variant === "danger",
          "bg-blue-500":   variant === "info",
          "bg-purple-500": variant === "purple",
      })} />}
            {children}
    </span>
    );
}