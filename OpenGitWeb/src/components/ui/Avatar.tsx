import { cn } from "../../lib/utils";

interface AvatarProps {
    src?:      string | null;
    username:  string;
    size?:     "xs" | "sm" | "md" | "lg" | "xl";
    className?: string;
}

const sizes = {
    xs: "w-5 h-5 text-xs",
    sm: "w-7 h-7 text-xs",
    md: "w-9 h-9 text-sm",
    lg: "w-12 h-12 text-base",
    xl: "w-20 h-20 text-2xl",
};

export function Avatar({ src, username, size = "md", className }: AvatarProps) {
    const initials = username.slice(0, 2).toUpperCase();
    const colors = [
        "bg-blue-500", "bg-green-500", "bg-purple-500",
        "bg-orange-500", "bg-pink-500", "bg-teal-500",
    ];
    const color = colors[username.charCodeAt(0) % colors.length];

    if (src) {
        return (
            <img
                src={src}
                alt={username}
                className={cn("rounded-full object-cover", sizes[size], className)}
            />
        );
    }

    return (
        <div className={cn(
            "rounded-full flex items-center justify-center text-white font-semibold flex-shrink-0",
            color,
            sizes[size],
            className
        )}>
            {initials}
        </div>
    );
}