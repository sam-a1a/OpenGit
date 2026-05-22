import React, { useState } from "react";
import { useParams, Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
    GitCommit, Copy, Check, ChevronRight,
    FileDiff, Plus, Minus
} from "lucide-react";
import { Header } from "../../components/layout/Header";
import { Avatar } from "../../components/ui/Avatar";
import { Badge } from "../../components/ui/Badge";
import { PageSpinner } from "../../components/ui/Spinner";
import { apiClient } from "../../api/client";
import { formatDateTime, cn } from "../../lib/utils";

export default function CommitDetailPage() {
    const { owner, repo: repoName, sha } = useParams<{
        owner: string; repo: string; sha: string;
    }>();

    const [copiedSha, setCopiedSha] = useState(false);

    const { data, isLoading } = useQuery({
        queryKey: ["commit", owner, repoName, sha],
        queryFn:  () => apiClient.get(
            `/repos/${owner}/${repoName}/git/commits/${sha}/single`
        ).then((r) => r.data),
        enabled: !!owner && !!repoName && !!sha,
    });

    const { data: diffData } = useQuery({
        queryKey: ["commit-diff", owner, repoName, sha],
        queryFn:  () => {
            const parents = data?.commit?.parents ?? [];
            if (!parents.length) return null;
            return apiClient.get(`/repos/${owner}/${repoName}/git/diff`, {
                params: { base: parents[0], head: sha }
            }).then((r) => r.data);
        },
        enabled: !!data?.commit,
    });

    const copySha = () => {
        navigator.clipboard.writeText(sha ?? "");
        setCopiedSha(true);
        setTimeout(() => setCopiedSha(false), 2000);
    };

    if (isLoading) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
                <Header />
                <PageSpinner />
            </div>
        );
    }

    const commit = data?.commit;

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            <Header />
            <div className="max-w-5xl mx-auto px-4 py-6">

                {/* breadcrumb */}
                <div className="flex items-center gap-1 text-sm mb-6 flex-wrap">
                    <Link to={`/${owner}/${repoName}`}
                          className="text-blue-600 dark:text-blue-400 hover:underline font-semibold">
                        {repoName}
                    </Link>
                    <ChevronRight className="w-4 h-4 text-gray-400" />
                    <Link to={`/${owner}/${repoName}/commits/main`}
                          className="text-blue-600 dark:text-blue-400 hover:underline">
                        Commits
                    </Link>
                    <ChevronRight className="w-4 h-4 text-gray-400" />
                    <span className="font-mono text-gray-600 dark:text-gray-400">
            {sha?.slice(0, 7)}
          </span>
                </div>

                {commit && (
                    <div className="space-y-4">
                        {/* commit header */}
                        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6">
                            <h1 className="text-xl font-bold text-gray-900 dark:text-white mb-3 leading-snug">
                                {commit.message}
                            </h1>

                            <div className="flex items-center gap-3 flex-wrap text-sm">
                                <div className="flex items-center gap-2">
                                    <Avatar username={commit.author_name} size="sm" />
                                    <span className="font-medium text-gray-900 dark:text-white">
                    {commit.author_name}
                  </span>
                                </div>
                                <span className="text-gray-500">
                  committed {formatDateTime(commit.authored_at)}
                </span>
                                {commit.committer_name !== commit.author_name && (
                                    <span className="text-gray-500">
                    · committed by {commit.committer_name}
                  </span>
                                )}
                            </div>

                            <div className="flex items-center gap-3 mt-4 flex-wrap">
                                <div className="flex items-center gap-2 bg-gray-50 dark:bg-gray-800 rounded-lg px-3 py-1.5">
                                    <GitCommit className="w-4 h-4 text-gray-400" />
                                    <code className="text-sm font-mono text-gray-700 dark:text-gray-300">
                                        {sha?.slice(0, 40)}
                                    </code>
                                    <button
                                        onClick={copySha}
                                        className="text-gray-400 hover:text-gray-600 ml-1"
                                    >
                                        {copiedSha
                                            ? <Check className="w-3.5 h-3.5 text-green-500" />
                                            : <Copy className="w-3.5 h-3.5" />}
                                    </button>
                                </div>

                                {commit.parents?.length > 0 && (
                                    <div className="flex items-center gap-1 text-sm text-gray-500">
                                        <span>Parent:</span>
                                        {commit.parents.map((p: string) => (
                                            <Link
                                                key={p}
                                                to={`/${owner}/${repoName}/commit/${p}`}
                                                className="font-mono text-blue-600 dark:text-blue-400 hover:underline"
                                            >
                                                {p.slice(0, 7)}
                                            </Link>
                                        ))}
                                    </div>
                                )}
                            </div>
                        </div>

                        {/* diff stats */}
                        {diffData && (
                            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 px-4 py-3 flex items-center gap-4 text-sm flex-wrap">
                                <div className="flex items-center gap-1.5 text-gray-600 dark:text-gray-400">
                                    <FileDiff className="w-4 h-4" />
                                    <span>
                    <strong>{diffData.files_changed}</strong> file{diffData.files_changed !== 1 ? "s" : ""} changed
                  </span>
                                </div>
                                <div className="flex items-center gap-1 text-green-600 dark:text-green-400">
                                    <Plus className="w-4 h-4" />
                                    <strong>{diffData.additions}</strong> additions
                                </div>
                                <div className="flex items-center gap-1 text-red-600 dark:text-red-400">
                                    <Minus className="w-4 h-4" />
                                    <strong>{diffData.deletions}</strong> deletions
                                </div>

                                {/* visual bar */}
                                <div className="flex-1 hidden sm:flex items-center gap-1 ml-2">
                                    {Array.from({ length: Math.min(diffData.files_changed, 10) }).map((_, i) => (
                                        <div key={i}
                                             className={cn(
                                                 "h-2.5 flex-1 rounded-sm",
                                                 i < (diffData.files_changed * 0.6)
                                                     ? "bg-green-400"
                                                     : "bg-red-400"
                                             )}
                                        />
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* file diffs */}
                        {diffData?.files?.map((file: any) => (
                            <CommitFileDiff key={file.path} file={file} patch={diffData.patch} />
                        ))}

                        {!diffData && (
                            <div className="text-center py-8 text-gray-500 text-sm">
                                {commit.parents?.length === 0
                                    ? "Initial commit — no diff available"
                                    : "Loading diff..."}
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}

function CommitFileDiff({ file, patch }: { file: any; patch: string }) {
    const [collapsed, setCollapsed] = useState(false);

    const filePatch = patch
        .split("diff --git")
        .find((p: string) => p.includes(file.path)) ?? "";

    const lines = filePatch.split("\n").slice(1);

    return (
        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
            <button
                onClick={() => setCollapsed(!collapsed)}
                className="w-full flex items-center gap-3 px-4 py-3 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 text-left hover:bg-gray-100 dark:hover:bg-gray-750 transition-colors"
            >
                <FileDiff className="w-4 h-4 text-gray-400 flex-shrink-0" />
                <span className="text-sm font-mono text-gray-700 dark:text-gray-300 flex-1 truncate">
          {file.path}
        </span>
                <div className="flex items-center gap-2 text-xs flex-shrink-0">
          <span className="text-green-600 dark:text-green-400 font-medium">
            +{file.additions}
          </span>
                    <span className="text-red-600 dark:text-red-400 font-medium">
            -{file.deletions}
          </span>
                </div>
            </button>

            {!collapsed && lines.length > 0 && (
                <div className="overflow-x-auto font-mono text-xs">
                    {lines.map((line: string, i: number) => {
                        const isAdd  = line.startsWith("+") && !line.startsWith("+++");
                        const isDel  = line.startsWith("-") && !line.startsWith("---");
                        const isHunk = line.startsWith("@@");

                        return (
                            <div key={i} className={cn(
                                "px-4 py-0.5 whitespace-pre leading-5",
                                isAdd  && "bg-green-50  dark:bg-green-900/20  text-green-800  dark:text-green-300",
                                isDel  && "bg-red-50    dark:bg-red-900/20    text-red-800    dark:text-red-300",
                                isHunk && "bg-blue-50   dark:bg-blue-900/20   text-blue-600   dark:text-blue-400 py-1",
                                !isAdd && !isDel && !isHunk && "text-gray-700 dark:text-gray-300"
                            )}>
                                {line || " "}
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
}