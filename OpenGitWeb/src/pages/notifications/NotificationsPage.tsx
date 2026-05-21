import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Bell, Check, CheckCheck, Bookmark, BookmarkX, Trash2 } from "lucide-react";
import { PageLayout } from "../../components/layout/PageLayout";
import { Button } from "../../components/ui/Button";
import { Badge } from "../../components/ui/Badge";
import { PageSpinner } from "../../components/ui/Spinner";
import { apiClient } from "../../api/client";
import { relativeTime, cn } from "../../lib/utils";

const REASON_LABELS: Record<string, string> = {
    assign:           "Assigned",
    author:           "Author",
    comment:          "Commented",
    mention:          "Mentioned",
    review_requested: "Review requested",
    subscribed:       "Subscribed",
    state_change:     "State changed",
    ci_activity:      "CI",
};

export default function NotificationsPage() {
    const [filter, setFilter] = useState<"unread" | "saved" | "all">("unread");
    const queryClient = useQueryClient();

    const { data, isLoading } = useQuery({
        queryKey: ["notifications", filter],
        queryFn:  () => apiClient.get("/notifications", {
            params: {
                all:   filter === "all",
                saved: filter === "saved",
            }
        }).then((r) => r.data),
    });

    const notifications = data?.notifications ?? [];
    const unreadCount   = data?.unread_count ?? 0;

    const markAllRead = useMutation({
        mutationFn: () => apiClient.put("/notifications", {}),
        onSuccess:  () => queryClient.invalidateQueries({ queryKey: ["notifications"] }),
    });

    const markRead = useMutation({
        mutationFn: (id: string) => apiClient.patch(`/notifications/${id}/read`),
        onSuccess:  () => queryClient.invalidateQueries({ queryKey: ["notifications"] }),
    });

    const saveNotification = useMutation({
        mutationFn: ({ id, saved }: { id: string; saved: boolean }) =>
            saved
                ? apiClient.put(`/notifications/${id}/save`)
                : apiClient.delete(`/notifications/${id}/save`),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ["notifications"] }),
    });

    const deleteNotification = useMutation({
        mutationFn: (id: string) => apiClient.delete(`/notifications/${id}`),
        onSuccess:  () => queryClient.invalidateQueries({ queryKey: ["notifications"] }),
    });

    return (
        <PageLayout narrow>
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-2">
                    <Bell className="w-5 h-5 text-gray-700 dark:text-gray-300" />
                    <h1 className="text-lg font-bold text-gray-900 dark:text-white">Notifications</h1>
                    {unreadCount > 0 && <Badge variant="danger">{unreadCount}</Badge>}
                </div>
                {unreadCount > 0 && (
                    <Button
                        variant="ghost"
                        size="sm"
                        icon={<CheckCheck className="w-4 h-4" />}
                        loading={markAllRead.isPending}
                        onClick={() => markAllRead.mutate()}
                    >
                        Mark all read
                    </Button>
                )}
            </div>

            {/* filter tabs */}
            <div className="flex gap-1 mb-4 border-b border-gray-200 dark:border-gray-800">
                {([
                    { key: "unread", label: "Unread" },
                    { key: "saved",  label: "Saved"  },
                    { key: "all",    label: "All"    },
                ] as const).map((t) => (
                    <button
                        key={t.key}
                        onClick={() => setFilter(t.key)}
                        className={cn(
                            "px-4 py-2.5 text-sm font-medium border-b-2 -mb-px transition-colors",
                            filter === t.key
                                ? "border-blue-500 text-blue-600 dark:text-blue-400"
                                : "border-transparent text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
                        )}
                    >
                        {t.label}
                    </button>
                ))}
            </div>

            {isLoading ? (
                <PageSpinner />
            ) : notifications.length === 0 ? (
                <div className="text-center py-20">
                    <Bell className="w-12 h-12 mx-auto text-gray-200 mb-4" />
                    <p className="text-gray-500">
                        {filter === "unread" ? "No unread notifications" :
                            filter === "saved"  ? "No saved notifications"  : "No notifications"}
                    </p>
                </div>
            ) : (
                <div className="space-y-1">
                    {notifications.map((n: any) => (
                        <div
                            key={n.id}
                            className={cn(
                                "flex items-start gap-3 p-4 rounded-xl border transition-colors",
                                n.is_read
                                    ? "bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800"
                                    : "bg-blue-50/50 dark:bg-blue-900/10 border-blue-100 dark:border-blue-900/30"
                            )}
                        >
                            {!n.is_read && (
                                <div className="w-2 h-2 rounded-full bg-blue-500 flex-shrink-0 mt-2" />
                            )}

                            <div className="flex-1 min-w-0">
                                <div className="flex items-start justify-between gap-2">
                                    <p className="text-sm text-gray-900 dark:text-gray-100 font-medium line-clamp-1">
                                        {n.subject_title}
                                    </p>
                                    <span className="text-xs text-gray-400 flex-shrink-0">
                    {relativeTime(n.updated_at)}
                  </span>
                                </div>
                                <div className="flex items-center gap-2 mt-1">
                                    <span className="text-xs text-gray-500">{n.subject_type}</span>
                                    {n.reason && (
                                        <Badge size="sm" variant="default">
                                            {REASON_LABELS[n.reason] ?? n.reason}
                                        </Badge>
                                    )}
                                </div>
                            </div>

                            <div className="flex items-center gap-1 flex-shrink-0">
                                {!n.is_read && (
                                    <button
                                        onClick={() => markRead.mutate(n.id)}
                                        title="Mark as read"
                                        className="p-1.5 rounded text-gray-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/20"
                                    >
                                        <Check className="w-3.5 h-3.5" />
                                    </button>
                                )}
                                <button
                                    onClick={() => saveNotification.mutate({ id: n.id, saved: !n.is_saved })}
                                    title={n.is_saved ? "Unsave" : "Save"}
                                    className={cn(
                                        "p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-800",
                                        n.is_saved ? "text-yellow-500" : "text-gray-400 hover:text-yellow-500"
                                    )}
                                >
                                    {n.is_saved
                                        ? <BookmarkX className="w-3.5 h-3.5" />
                                        : <Bookmark className="w-3.5 h-3.5" />}
                                </button>
                                <button
                                    onClick={() => deleteNotification.mutate(n.id)}
                                    title="Delete"
                                    className="p-1.5 rounded text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"
                                >
                                    <Trash2 className="w-3.5 h-3.5" />
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </PageLayout>
    );
}