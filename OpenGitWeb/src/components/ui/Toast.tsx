import React, { createContext, useContext, useState, useCallback } from "react";
import { X, CheckCircle, XCircle, AlertCircle, Info } from "lucide-react";
import { cn } from "../../lib/utils";

type ToastType = "success" | "error" | "warning" | "info";

interface Toast {
    id:      string;
    type:    ToastType;
    title?:  string;
    message: string;
}

interface ToastContextType {
    toast: (message: string, type?: ToastType, title?: string) => void;
    success: (message: string, title?: string) => void;
    error:   (message: string, title?: string) => void;
    warning: (message: string, title?: string) => void;
    info:    (message: string, title?: string) => void;
}

const ToastContext = createContext<ToastContextType | null>(null);

export function ToastProvider({ children }: { children: React.ReactNode }) {
    const [toasts, setToasts] = useState<Toast[]>([]);

    const remove = useCallback((id: string) => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
    }, []);

    const add = useCallback((
        message: string,
        type: ToastType = "info",
        title?: string
    ) => {
        const id = Math.random().toString(36).slice(2);
        setToasts((prev) => [...prev, { id, type, message, title }]);
        setTimeout(() => remove(id), 4000);
    }, [remove]);

    const value: ToastContextType = {
        toast:   add,
        success: (m, t) => add(m, "success", t),
        error:   (m, t) => add(m, "error",   t),
        warning: (m, t) => add(m, "warning", t),
        info:    (m, t) => add(m, "info",    t),
    };

    const icons = {
        success: <CheckCircle className="w-5 h-5 text-green-500 flex-shrink-0" />,
        error:   <XCircle     className="w-5 h-5 text-red-500   flex-shrink-0" />,
        warning: <AlertCircle className="w-5 h-5 text-yellow-500 flex-shrink-0" />,
        info:    <Info        className="w-5 h-5 text-blue-500  flex-shrink-0" />,
    };

    const styles = {
        success: "border-green-200 dark:border-green-800 bg-white dark:bg-gray-900",
        error:   "border-red-200   dark:border-red-800   bg-white dark:bg-gray-900",
        warning: "border-yellow-200 dark:border-yellow-800 bg-white dark:bg-gray-900",
        info:    "border-blue-200  dark:border-blue-800  bg-white dark:bg-gray-900",
    };

    return (
        <ToastContext.Provider value={value}>
            {children}
            <div className="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 w-80">
                {toasts.map((t) => (
                    <div
                        key={t.id}
                        className={cn(
                            "flex items-start gap-3 p-4 rounded-xl border shadow-lg",
                            "animate-in slide-in-from-bottom-2 fade-in duration-200",
                            styles[t.type]
                        )}
                    >
                        {icons[t.type]}
                        <div className="flex-1 min-w-0">
                            {t.title && (
                                <p className="text-sm font-semibold text-gray-900 dark:text-white">{t.title}</p>
                            )}
                            <p className="text-sm text-gray-600 dark:text-gray-400">{t.message}</p>
                        </div>
                        <button
                            onClick={() => remove(t.id)}
                            className="flex-shrink-0 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                        >
                            <X className="w-4 h-4" />
                        </button>
                    </div>
                ))}
            </div>
        </ToastContext.Provider>
    );
}

export function useToast() {
    const ctx = useContext(ToastContext);
    if (!ctx) throw new Error("useToast must be used within ToastProvider");
    return ctx;
}