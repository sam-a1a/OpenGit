import { Header } from "./Header";
import { cn } from "../../lib/utils";

interface PageLayoutProps {
    children:   React.ReactNode;
    className?: string;
    narrow?:    boolean;
}

export function PageLayout({ children, className, narrow }: PageLayoutProps) {
    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            <Header />
            <main className={cn(
                "mx-auto px-4 py-6",
                narrow ? "max-w-3xl" : "max-w-7xl",
                className
            )}>
                {children}
            </main>
        </div>
    );
}