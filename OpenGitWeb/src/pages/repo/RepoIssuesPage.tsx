import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { CircleDot, CheckCircle2, MessageSquare, Plus, Search } from "lucide-react";
import { Button } from "../../components/ui/Button";
import { Badge } from "../../components/ui/Badge";
import { Avatar } from "../../components/ui/Avatar";
import { PageSpinner } from "../../components/ui/Spinner";
import { issuesApi } from "../../api/issues";
import { relativeTime, cn } from "../../lib/utils";
import { useAuthStore } from "../../stores/auth";
import type { Repo } from "../../types/repo";
import type { Issue } from "../../types/issue";

interface Props { repo: Repo; owner: string; }

export default function RepoIssuesPage({ repo, owner }: Props) {
    const [state,  setState]  = useState<"open" | "closed">("open");
    const [query,  setQuery]  = useState("");
    const [showNew, setShowNew] = useState(false);
    const currentUser          = useAuthStore((s) => s.user);
    const queryClient          = useQueryClient();

    const { data, isLoading } = useQuery({
        queryKey: ["issues", owner, repo.name, state],
        queryFn:  () => issuesApi.list(owner, repo.name, { state })
            .then((r) => r.data),
    });

    const issues: Issue[] = (data?.issues ?? []).filter((i: Issue) =>
        !query || i.title.toLowerCase().includes(query.toLowerCase())
    );

    return (
        <div>
            <div className="flex items-center gap-3 mb-4 flex-wrap">
                <div className="relative flex-1 min-w-48">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
                    <input
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder="Search issues..."
                        className="w-full pl-9 pr-3 py-2 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                </div>
                {currentUser && (
                    <Button
                        size="sm"
                        icon={<Plus className="w-4 h-4" />}
                        onClick={() => setShowNew(true)}
                    >
                        New issue
                    </Button>
                )}
            </div>

            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                {/* state tabs */}
                <div className="flex items-center gap-4 px-4 py-3 border-b border-gray-200 dark:border-gray-800">
                    <button
                        onClick={() => setState("open")}
                        className={cn(
                            "flex items-center gap-1.5 text-sm font-medium",
                            state === "open"
                                ? "text-gray-900 dark:text-white"
                                : "text-gray-500 hover:text-gray-700"
                        )}
                    >
                        <CircleDot className="w-4 h-4 text-green-500" />
                        Open <Badge size="sm">{repo.open_issue_count}</Badge>
                    </button>
                    <button
                        onClick={() => setState("closed")}
                        className={cn(
                            "flex items-center gap-1.5 text-sm font-medium",
                            state === "closed"
                                ? "text-gray-900 dark:text-white"
                                : "text-gray-500 hover:text-gray-700"
                        )}
                    >
                        <CheckCircle2 className="w-4 h-4 text-purple-500" />
                        Closed
                    </button>
                </div>

                {isLoading ? (
                    <div className="py-10"><PageSpinner /></div>
                ) : issues.length === 0 ? (
                    <div className="py-16 text-center">
                        {state === "open"
                            ? <CircleDot className="w-10 h-10 mx-auto text-gray-200 mb-3" />
                            : <CheckCircle2 className="w-10 h-10 mx-auto text-gray-200 mb-3" />}
                        <p className="text-gray-500 text-sm">
                            {state === "open" ? "No open issues" : "No closed issues"}
                        </p>
                    </div>
                ) : (
                    <div className="divide-y divide-gray-100 dark:divide-gray-800">
                        {issues.map((issue) => (
                            <IssueRow key={issue.id} issue={issue} owner={owner} repoName={repo.name} />
                        ))}
                    </div>
                )}
            </div>

            {showNew && (
                <NewIssueModal
                    owner={owner}
                    repoName={repo.name}
                    onClose={() => setShowNew(false)}
                    onCreated={() => {
                        setShowNew(false);
                        queryClient.invalidateQueries({ queryKey: ["issues", owner, repo.name] });
                    }}
                />
            )}
        </div>
    );
}

function IssueRow({ issue, owner, repoName }: { issue: Issue; owner: string; repoName: string }) {
    return (

        href={`/${owner}/${repoName}/issues/${issue.number}`}
    className="flex items-start gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-800"
        >
        {issue.state === "Open"
                ? <CircleDot className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                : <CheckCircle2 className="w-4 h-4 text-purple-500 mt-0.5 flex-shrink-0" />}

    <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-sm font-medium text-gray-900 dark:text-white hover:text-blue-600">
            {issue.title}
          </span>
            {issue.is_pinned && <Badge size="sm" variant="info">Pinned</Badge>}
            {issue.locked   && <Badge size="sm" variant="warning">Locked</Badge>}
        </div>
        <p className="text-xs text-gray-500 mt-0.5">
            #{issue.number} opened {relativeTime(issue.created_at)}
        </p>
    </div>

    {issue.comment_count > 0 && (
        <span className="flex items-center gap-1 text-xs text-gray-500 flex-shrink-0">
          <MessageSquare className="w-3.5 h-3.5" />
            {issue.comment_count}
        </span>
    )}
</a>
);
}

function NewIssueModal({ owner, repoName, onClose, onCreated }: {
    owner:     string;
    repoName:  string;
    onClose:   () => void;
    onCreated: () => void;
}) {
    const [title, setTitle] = useState("");
    const [body,  setBody]  = useState("");
    const [error, setError] = useState("");

    const mutation = useMutation({
        mutationFn: () => issuesApi.create(owner, repoName, { title, body }),
        onSuccess:  onCreated,
        onError:    (e: any) => setError(e.response?.data?.error ?? "Failed to create issue"),
    });

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50">
            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-700 w-full max-w-2xl shadow-2xl">
                <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-800">
                    <h2 className="text-lg font-semibold text-gray-900 dark:text-white">New issue</h2>
                </div>
                <div className="p-6 space-y-4">
                    {error && <p className="text-sm text-red-500">{error}</p>}
                    <input
                        value={title}
                        onChange={(e) => setTitle(e.target.value)}
                        placeholder="Title"
                        className="w-full px-3 py-2 rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <textarea
                        value={body}
                        onChange={(e) => setBody(e.target.value)}
                        placeholder="Leave a comment"
                        rows={6}
                        className="w-full px-3 py-2 rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono"
                    />
                </div>
                <div className="px-6 py-4 border-t border-gray-200 dark:border-gray-800 flex justify-end gap-2">
                    <Button variant="ghost" size="sm" onClick={onClose}>Cancel</Button>
                    <Button
                        size="sm"
                        loading={mutation.isPending}
                        disabled={!title.trim()}
                        onClick={() => mutation.mutate()}
                    >
                        Submit issue
                    </Button>
                </div>
            </div>
        </div>
    );
}