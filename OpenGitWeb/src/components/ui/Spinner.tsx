import { Loader2 } from "lucide-react";
import { cn } from "../../lib/utils";

export function Spinner({ className }: { className?: string }) {
    return (
        <Loader2 className={cn("animate-spin text-blue-500", className)} />
    );
}

export function PageSpinner() {
    return (
        <div className="flex items-center justify-center min-h-64">
            <Spinner className="w-8 h-8" />
        </div>
    );
}