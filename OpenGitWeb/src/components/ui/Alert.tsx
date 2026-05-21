import { cn } from "../../lib/utils";
import { AlertCircle, CheckCircle, Info, XCircle } from "lucide-react";

interface AlertProps {
    type?:     "info" | "success" | "warning" | "error";
    title?:    string;
    children:  React.ReactNode;
    className?: string;
}

export function Alert({ type = "info", title, children, className }: AlertProps) {
    const styles = {
        info:    { wrap: "bg-blue-50 border-blue-200 dark:bg-blue-900/20 dark:border-blue-800",    icon: <Info className="w-5 h-5 text-blue-500" />,    text: "text-blue-800 dark:text-blue-300" },
        success: { wrap: "bg-green-50 border-green-200 dark:bg-green-900/20 dark:border-green-800", icon: <CheckCircle className="w-5 h-5 text-green-500" />, text: "text-green-800 dark:text-green-300" },
        warning: { wrap: "bg-yellow-50 border-yellow-200 dark:bg-yellow-900/20 dark:border-yellow-800", icon: <AlertCircle className="w-5 h-5 text-yellow-500" />, text: "text-yellow-800 dark:text-yellow-300" },
        error:   { wrap: "bg-red-50 border-red-200 dark:bg-red-900/20 dark:border-red-800",     icon: <XCircle className="w-5 h-5 text-red-500" />,    text: "text-red-800 dark:text-red-300" },
    };

    const s = styles[type];

    return (
        <div className={cn("flex gap-3 p-4 rounded-lg border", s.wrap, className)}>
            <div className="flex-shrink-0 mt-0.5">{s.icon}</div>
            <div className={cn("text-sm", s.text)}>
                {title && <p className="font-semibold mb-1">{title}</p>}
                {children}
            </div>
        </div>
    );
}