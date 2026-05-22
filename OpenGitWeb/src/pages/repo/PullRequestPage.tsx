import { useState } from "react";
import { useParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
    GitPullRequest, GitMerge, XCircle, MessageSquare,
    GitCommit, FileDiff, CheckCircle2, ChevronDown
} from "lucide-react";
import { Header } from "../../components/layout/Header";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { Alert } from "../../components/ui/Alert";
import { PageSpinner } from "../../components/ui/Spinner";
import { Tabs } from "../../components/ui/Tabs";
import { apiClient } from "../../api/client";
import { useAuthStore } from "../../stores/auth";
import { relativeTime, cn } from "../../lib/utils";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { useUiStore } from "../../stores/ui";

export default function PullRequestPage() {
    const { owner, repo: repoName, number } = useParams<{
        owner: string; repo: string; number: string;
    }>();
    const currentUser = useAuthStore((s) => s.user);
    const theme       = useUiStore((s) => s.theme);
    const queryClient = useQueryClient();
    const [tab,     setTab]     = useState("conversation");
    const [comment, setComment] = useState("");
    const [mergeMethod, setMergeMethod] = useState<"merge" | "squash" | "rebase">("merge");
    const [showMerge, setShowMerge] = useState(false);
    const [mergeError, setMergeError] = useState("");

    const { data: pr, isLoading } = useQuery({
        queryKey: ["pr", owner, repoName, number],
        queryFn:  () => apiClient.get(`/repos/${owner}/${repoName}/pulls/${number}`)
            .then((r) => r.data),
    });

    const { data: reviewComments } = useQuery({
        queryKey: ["pr-comments", owner, repoName, number],
        queryFn:  () => apiClient.get(`/repos/${owner}/${repoName}/pulls/${number}/comments`)
            .then((r) => r.data),
        enabled: !!pr,
    });

    const { data: reviews } = useQuery({
        queryKey: ["pr-reviews", owner, repoName, number],
        queryFn:  () => apiClient.get(`/repos/${owner}/${repoName}/pulls/${number}/reviews`)
            .then((r) => r.data),
        enabled: !!pr,
    });

    const { data: diffData } = useQuery({
        queryKey: ["pr-diff", owner, repoName, pr?.base_branch, pr?.head_branch],
        queryFn:  () => apiClient.get(`/repos/${owner}/${repoName}/git/diff`, {
            params: { base: pr.base_branch, head: pr.head_branch }
        }).then((r) => r.data),
        enabled: !!pr && tab === "files",
    });

    const { data: commits } = useQuery({
        queryKey: ["pr-commits", owner, repoName, pr?.head_branch],
        queryFn:  () => apiClient.get(`/repos/${owner}/${repoName}/git/commits/${pr.head_branch}`)
            .then((r) => r.data),
        enabled: !!pr && tab === "commits",
    });

    const addComment = useMutation({
        mutationFn: () => apiClient.post(
            `/repos/${owner}/${repoName}/pulls/${number}/comments`,
            { body: comment }
        ),
        onSuccess: () => {
            setComment("");
            queryClient.invalidateQueries({ queryKey: ["pr-comments"] });
        },
    });

    const mergePr = useMutation({
        mutationFn: () => apiClient.put(
            `/repos/${owner}/${repoName}/pulls/${number}/merge`,
            { merge_method: mergeMethod }
        ),
        onSuccess: () => {
            setShowMerge(false);
            queryClient.invalidateQueries({ queryKey: ["pr", owner, repoName, number] });
        },
        onError: (e: any) => setMergeError(e.response?.data?.error ?? "Merge failed"),
    });

    const closePr = useMutation({
        mutationFn: () => apiClient.put(`/repos/${owner}/${repoName}/pulls/${number}/close`),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ["pr"] }),
    });

    const reopenPr = useMutation({
        mutationFn: () => apiClient.put(`/repos/${owner}/${repoName}/pulls/${number}/reopen`),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ["pr"] }),
    });

    if (isLoading) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
                <Header />
                <PageSpinner />
            </div>
        );
    }

    if (!pr) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
                <Header />
                <div className="max-w-4xl mx-auto px-4 py-16 text-center text-gray-500">
                    Pull request not found
                </div>
            </div>
        );
    }

    const isOpen   = pr.state === "Open";
    const isMerged = pr.state === "Merged";
    const isOwner  = currentUser?.id === pr.author_id;

    const StateIcon = isMerged
        ? GitMerge
        : isOpen
            ? GitPullRequest
            : XCircle;

    const stateVariant = isMerged ? "purple" : isOpen ? "success" : "danger";
    const stateLabel   = isMerged ? "Merged"  : isOpen ? "Open"   : "Closed";

    const tabs = [
        { key: "conversation", label: "Conversation",  count: (reviewComments ?? []).length },
        { key: "commits",      label: "Commits",        count: commits?.commits?.length },
        { key: "files",        label: "Files changed",  count: diffData?.files_changed },
    ];

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            <Header />
            <div className="max-w-5xl mx-auto px-4 py-6">

                {/* header */}
                <div className="mb-6">
                    <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2 leading-tight">
                        {pr.title}
                        <span className="text-gray-400 font-normal ml-2">#{pr.number}</span>
                    </h1>
                    <div className="flex items-center gap-3 flex-wrap">
                        <Badge variant={stateVariant} size="md" dot>
                            <StateIcon className="w-3.5 h-3.5" />
                            {stateLabel}
                        </Badge>
                        {pr.is_draft && <Badge>Draft</Badge>}
                        <span className="text-sm text-gray-500">
              {pr.head_branch} → {pr.base_branch}
            </span>
                        <span className="text-sm text-gray-500">
              {relativeTime(pr.created_at)}
            </span>
                    </div>
                </div>

                <Tabs tabs={tabs} active={tab} onChange={setTab} className="mb-6" />

                <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
                    <div className="lg:col-span-3 space-y-4">

                        {/* conversation tab */}
                        {tab === "conversation" && (
                            <>
                                {/* PR body */}
                                <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                                    <div className="flex items-center gap-2 px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 text-sm">
                    <span className="font-medium text-gray-900 dark:text-white">
                      {pr.author_id ? "Author" : "Unknown"}
                    </span>
                                        <span className="text-gray-500">opened this pull request {relativeTime(pr.created_at)}</span>
                                    </div>
                                    <div className="p-5">
                                        {pr.body ? (
                                            <div className="prose dark:prose-invert prose-sm max-w-none">
                                                <ReactMarkdown remarkPlugins={[remarkGfm]}>{pr.body}</ReactMarkdown>
                                            </div>
                                        ) : (
                                            <p className="text-gray-400 italic text-sm">No description provided.</p>
                                        )}
                                    </div>
                                </div>

                                {/* review comments */}
                                {(reviewComments ?? []).map((c: any) => (
                                    <div key={c.id} className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                                        <div className="flex items-center gap-2 px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 text-sm">
                      <span className="font-medium text-gray-900 dark:text-white">
                        Reviewer
                      </span>
                                            <span className="text-gray-500">{relativeTime(c.created_at)}</span>
                                            {c.path && (
                                                <Badge size="sm" className="ml-auto font-mono">{c.path}:{c.line}</Badge>
                                            )}
                                        </div>
                                        <div className="p-4 text-sm">
                                            {c.path && (
                                                <div className="mb-3 bg-gray-50 dark:bg-gray-800 rounded-lg p-3 font-mono text-xs text-gray-600 dark:text-gray-400 border border-gray-200 dark:border-gray-700">
                                                    {c.path}
                                                </div>
                                            )}
                                            <div className="prose dark:prose-invert prose-sm max-w-none">
                                                <ReactMarkdown remarkPlugins={[remarkGfm]}>{c.body}</ReactMarkdown>
                                            </div>
                                        </div>
                                    </div>
                                ))}

                                {/* merge section */}
                                {isOpen && currentUser && (
                                    <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-5">
                                        <div className="flex items-center gap-2 mb-3">
                                            <CheckCircle2 className="w-5 h-5 text-green-500" />
                                            <span className="text-sm font-medium text-gray-900 dark:text-white">
                        Ready to merge
                      </span>
                                        </div>

                                        {mergeError && (
                                            <Alert type="error" onClose={() => setMergeError("")} className="mb-3">
                                                {mergeError}
                                            </Alert>
                                        )}

                                        <div className="flex items-stretch gap-px">
                                            <Button
                                                className="rounded-r-none flex-1"
                                                loading={mergePr.isPending}
                                                onClick={() => mergePr.mutate()}
                                            >
                                                <GitMerge className="w-4 h-4" />
                                                {mergeMethod === "merge"  ? "Merge pull request" :
                                                    mergeMethod === "squash" ? "Squash and merge" : "Rebase and merge"}
                                            </Button>
                                            <div className="relative">
                                                <button
                                                    onClick={() => setShowMerge(!showMerge)}
                                                    className="h-full px-2.5 bg-blue-600 hover:bg-blue-700 text-white rounded-r-md border-l border-blue-500 flex items-center"
                                                >
                                                    <ChevronDown className="w-4 h-4" />
                                                </button>
                                                {showMerge && (
                                                    <div className="absolute right-0 bottom-full mb-1 w-56 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-xl shadow-xl overflow-hidden text-sm z-10">
                                                        {(["merge", "squash", "rebase"] as const).map((m) => (
                                                            <button
                                                                key={m}
                                                                onClick={() => { setMergeMethod(m); setShowMerge(false); }}
                                                                className={cn(
                                                                    "w-full px-4 py-3 text-left hover:bg-gray-50 dark:hover:bg-gray-800",
                                                                    mergeMethod === m
                                                                        ? "text-blue-600 dark:text-blue-400 font-medium"
                                                                        : "text-gray-700 dark:text-gray-300"
                                                                )}
                                                            >
                                                                <p className="font-medium capitalize">{m} and merge</p>
                                                                <p className="text-xs text-gray-500 mt-0.5">
                                                                    {m === "merge"  && "Keep all commits in history"}
                                                                    {m === "squash" && "Combine into one commit"}
                                                                    {m === "rebase" && "Rebase commits onto base"}
                                                                </p>
                                                            </button>
                                                        ))}
                                                    </div>
                                                )}
                                            </div>
                                        </div>
                                    </div>
                                )}

                                {/* add comment */}
                                {currentUser && (
                                    <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4">
                    <textarea
                        value={comment}
                        onChange={(e) => setComment(e.target.value)}
                        placeholder="Leave a comment..."
                        rows={4}
                        className="w-full px-3 py-2 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono"
                    />
                                        <div className="flex items-center justify-between mt-3">
                                            {isOpen ? (
                                                <Button variant="secondary" size="sm"
                                                        loading={closePr.isPending}
                                                        onClick={() => closePr.mutate()}>
                                                    <XCircle className="w-4 h-4" /> Close pull request
                                                </Button>
                                            ) : !isMerged ? (
                                                <Button variant="secondary" size="sm"
                                                        loading={reopenPr.isPending}
                                                        onClick={() => reopenPr.mutate()}>
                                                    <GitPullRequest className="w-4 h-4" /> Reopen
                                                </Button>
                                            ) : <div />}
                                            <Button
                                                size="sm"
                                                disabled={!comment.trim()}
                                                loading={addComment.isPending}
                                                onClick={() => addComment.mutate()}
                                            >
                                                <MessageSquare className="w-4 h-4" /> Comment
                                            </Button>
                                        </div>
                                    </div>
                                )}
                            </>
                        )}

                        {/* commits tab */}
                        {tab === "commits" && (
                            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                                {!commits?.commits?.length ? (
                                    <div className="py-10 text-center text-gray-500 text-sm">No commits</div>
                                ) : (
                                    <div className="divide-y divide-gray-100 dark:divide-gray-800">
                                        {commits.commits.map((c: any) => (
                                            <div key={c.sha} className="flex items-start gap-3 px-4 py-3">
                                                <GitCommit className="w-4 h-4 text-gray-400 mt-0.5 flex-shrink-0" />
                                                <div className="min-w-0 flex-1">
                                                    <p className="text-sm text-gray-900 dark:text-white font-medium truncate">
                                                        {c.message}
                                                    </p>
                                                    <p className="text-xs text-gray-500 mt-0.5">
                                                        {c.author_name} · {relativeTime(c.authored_at)}
                                                    </p>
                                                </div>
                                                <code className="text-xs font-mono bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 px-2 py-0.5 rounded flex-shrink-0">
                                                    {c.sha.slice(0, 7)}
                                                </code>
                                            </div>
                                        ))}
                                    </div>
                                )}
                            </div>
                        )}

                        {/* files changed tab */}
                        {tab === "files" && (
                            <div className="space-y-3">
                                {!diffData ? (
                                    <PageSpinner />
                                ) : !diffData.files?.length ? (
                                    <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 py-10 text-center text-gray-500 text-sm">
                                        No file changes
                                    </div>
                                ) : (
                                    <>
                                        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-3 flex items-center gap-3 text-sm">
                                            <FileDiff className="w-4 h-4 text-gray-400" />
                                            <span className="text-gray-700 dark:text-gray-300">
                        <strong>{diffData.files_changed}</strong> file{diffData.files_changed !== 1 ? "s" : ""} changed
                      </span>
                                            <span className="text-green-600 dark:text-green-400">
                        +{diffData.additions}
                      </span>
                                            <span className="text-red-600 dark:text-red-400">
                        -{diffData.deletions}
                      </span>
                                        </div>

                                        {diffData.files.map((file: any) => (
                                            <DiffFile key={file.path} file={file} theme={theme} patch={diffData.patch} />
                                        ))}
                                    </>
                                )}
                            </div>
                        )}
                    </div>

                    {/* sidebar */}
                    <aside className="space-y-4 text-sm">
                        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4 space-y-3">
                            <div>
                                <h3 className="font-semibold text-gray-700 dark:text-gray-300 mb-1">Reviewers</h3>
                                <p className="text-gray-400 text-xs">No reviewers</p>
                            </div>
                            <div>
                                <h3 className="font-semibold text-gray-700 dark:text-gray-300 mb-1">Assignees</h3>
                                <p className="text-gray-400 text-xs">No assignees</p>
                            </div>
                            <div>
                                <h3 className="font-semibold text-gray-700 dark:text-gray-300 mb-1">Labels</h3>
                                <p className="text-gray-400 text-xs">None yet</p>
                            </div>
                            <div>
                                <h3 className="font-semibold text-gray-700 dark:text-gray-300 mb-2">Stats</h3>
                                <div className="space-y-1 text-xs text-gray-500">
                                    <div className="flex justify-between">
                                        <span>Additions</span>
                                        <span className="text-green-600">+{pr.additions}</span>
                                    </div>
                                    <div className="flex justify-between">
                                        <span>Deletions</span>
                                        <span className="text-red-600">-{pr.deletions}</span>
                                    </div>
                                    <div className="flex justify-between">
                                        <span>Files</span>
                                        <span>{pr.changed_files}</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </aside>
                </div>
            </div>
        </div>
    );
}

function DiffFile({ file, theme, patch }: { file: any; theme: string; patch: string }) {
    const [collapsed, setCollapsed] = useState(false);

    const filePatch = patch
        .split("diff --git")
        .find((p) => p.includes(file.path)) ?? "";

    const lines = filePatch.split("\n").slice(1);

    return (
        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
            <button
                onClick={() => setCollapsed(!collapsed)}
                className="w-full flex items-center gap-2 px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 text-left hover:bg-gray-100 dark:hover:bg-gray-750"
            >
                <FileDiff className="w-4 h-4 text-gray-400 flex-shrink-0" />
                <span className="text-sm font-mono text-gray-700 dark:text-gray-300 flex-1 truncate">
          {file.path}
        </span>
                <span className="text-xs text-green-600 dark:text-green-400">+{file.additions}</span>
                <span className="text-xs text-red-600 dark:text-red-400 ml-1">-{file.deletions}</span>
            </button>

            {!collapsed && (
                <div className="overflow-x-auto text-xs font-mono">
                    {lines.map((line, i) => {
                        const isAdd = line.startsWith("+") && !line.startsWith("+++");
                        const isDel = line.startsWith("-") && !line.startsWith("---");
                        const isHunk = line.startsWith("@@");

                        return (
                            <div key={i} className={cn(
                                "px-4 py-0.5 whitespace-pre",
                                isAdd  && "bg-green-50 dark:bg-green-900/20 text-green-800 dark:text-green-300",
                                isDel  && "bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-300",
                                isHunk && "bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 py-1",
                                !isAdd && !isDel && !isHunk && "text-gray-700 dark:text-gray-300"
                            )}>
                                {line}
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
}