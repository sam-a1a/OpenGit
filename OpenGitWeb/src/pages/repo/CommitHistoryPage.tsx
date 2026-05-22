import { useState } from "react";
import { useParams, Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { GitCommit, Copy, Check, ChevronLeft, ChevronRight } from "lucide-react";
import { Header } from "../../components/layout/Header";
import { Button } from "../../components/ui/Button";
import { PageSpinner } from "../../components/ui/Spinner";
import { Avatar } from "../../components/ui/Avatar";
import { reposApi } from "../../api/repos";
import { relativeTime } from "../../lib/utils";

export default function CommitHistoryPage() {
    const { owner, repo: repoName, "*": ref } = useParams<{
        owner: string; repo: string; "*": string;
    }>();
    const [page, setPage] = useState(1);
    const [copiedSha, setCopiedSha] = useState<string | null>(null);

    const branch = ref || "main";

    const { data, isLoading } = useQuery({
        queryKey: ["commits", owner, repoName, branch, page],
        queryFn:  () => reposApi.commits(owner!, repoName!, branch, page)
            .then((r) => r.data),
    });

    const commits = data?.commits ?? [];
    const perPage = data?.per_page ?? 30;

    const copySha = (sha: string) => {
        navigator.clipboard.writeText(sha);
        setCopiedSha(sha);
        setTimeout(() => setCopiedSha(null), 2000);
    };

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            <Header />
            <div className="max-w-4xl mx-auto px-4 py-6">

                {/* breadcrumb */}
                <div className="flex items-center gap-2 text-sm mb-6">
                    <Link to={`/${owner}/${repoName}`}
                          className="text-blue-600 dark:text-blue-400 hover:underline font-semibold">
                        {repoName}
                    </Link>
                    <span className="text-gray-400">/</span>
                    <span className="flex items-center gap-1.5 text-gray-900 dark:text-white font-medium">
            <GitCommit className="w-4 h-4" />
            Commits on {branch}
          </span>
                </div>

                {isLoading ? (
                    <PageSpinner />
                ) : commits.length === 0 ? (
                    <div className="text-center py-16 text-gray-500">No commits found</div>
                ) : (
                    <>
                        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                            <div className="divide-y divide-gray-100 dark:divide-gray-800">
                                {commits.map((commit: any) => (
                                    <div key={commit.sha} className="flex items-start gap-3 px-5 py-4 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
                                        <GitCommit className="w-4 h-4 text-gray-400 mt-1 flex-shrink-0" />

                                        <div className="min-w-0 flex-1">
                                            <Link
                                                to={`/${owner}/${repoName}/commit/${commit.sha}`}
                                                className="text-sm font-medium text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400 line-clamp-2 block"
                                            >
                                                {commit.message}
                                            </Link>
                                            <div className="flex items-center gap-2 mt-1.5 flex-wrap">
                                                <Avatar username={commit.author_name} size="xs" />
                                                <span className="text-xs text-gray-600 dark:text-gray-400 font-medium">
                          {commit.author_name}
                        </span>
                                                <span className="text-xs text-gray-400">
                          committed {relativeTime(commit.authored_at)}
                        </span>
                                                {commit.parents?.length > 1 && (
                                                    <span className="text-xs bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400 px-1.5 py-0.5 rounded">
                            Merge
                          </span>
                                                )}
                                            </div>
                                        </div>

                                        <div className="flex items-center gap-2 flex-shrink-0">
                                            <Link
                                                to={`/${owner}/${repoName}/commit/${commit.sha}`}
                                                className="text-xs font-mono bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 px-2 py-1 rounded"
                                            >
                                                {commit.sha.slice(0, 7)}
                                            </Link>
                                            <button
                                                onClick={() => copySha(commit.sha)}
                                                className="p-1.5 rounded text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700"
                                                title="Copy full SHA"
                                            >
                                                {copiedSha === commit.sha
                                                    ? <Check className="w-3.5 h-3.5 text-green-500" />
                                                    : <Copy className="w-3.5 h-3.5" />}
                                            </button>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>

                        {/* pagination */}
                        <div className="flex items-center justify-between mt-4">
                            <Button
                                variant="outline" size="sm"
                                disabled={page === 1}
                                icon={<ChevronLeft className="w-4 h-4" />}
                                onClick={() => setPage((p) => p - 1)}
                            >
                                Newer
                            </Button>
                            <span className="text-sm text-gray-500">Page {page}</span>
                            <Button
                                variant="outline" size="sm"
                                disabled={commits.length < perPage}
                                onClick={() => setPage((p) => p + 1)}
                            >
                                Older <ChevronRight className="w-4 h-4" />
                            </Button>
                        </div>
                    </>
                )}
            </div>
        </div>
    );
}