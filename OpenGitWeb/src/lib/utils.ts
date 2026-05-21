import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { formatDistanceToNow, format } from "date-fns";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export function relativeTime(date: string | Date): string {
    return formatDistanceToNow(new Date(date), { addSuffix: true });
}

export function formatDate(date: string | Date): string {
    return format(new Date(date), "MMM d, yyyy");
}

export function formatDateTime(date: string | Date): string {
    return format(new Date(date), "MMM d, yyyy 'at' h:mm a");
}

export function truncate(str: string, length: number): string {
    return str.length > length ? str.slice(0, length) + "…" : str;
}

export function pluralize(count: number, word: string): string {
    return `${count} ${word}${count !== 1 ? "s" : ""}`;
}