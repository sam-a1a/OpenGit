import React, { useState } from "react";
import { useParams, useNavigate, Link } from "react-router-dom";
import { useQuery, useMutation } from "@tanstack/react-query";
import { GitPullRequest, ArrowLeft, ArrowRight, Info } from "lucide-react";
import { Header } from "../../components/layout/Header";
import { Button } from "../../components/ui/Button";
import { Input } from "../../components/ui/Input";
import { Alert } from "../../components/ui/Alert";
import { Badge } from "../../components/ui/Badge";
import { PageSpinner } from "../../components/ui/Spinner";
import { apiClient } from "../../api/client";
import { reposApi } from "../../api/repos";
import { useToast } from "../../components/ui/Toast";
import { cn } from "../../lib/utils";

export default function CreatePrPage() {
    const { owner, repo: repoName } = useParams<{ owner: string; repo: string }>();
    const navigate = useNavigate();
    const { success, error: toastError } = useToast();

    const [title,      setTitle]      = useState("");
    const [body,       setBody]       = useState("");
    const [headBranch, setHeadBranch] = useState("");
    const [baseBranch, setBaseBranch] = useState("");
    const [isDraft,    setIsDraft]    = useState(false);
    const [error,      setError]      = useState("");

    const { data: refsData, isLoading: refsLoading } = useQuery({
        queryKey: ["refs", owner, repoName],
        queryFn:  () => reposApi.refs(owner!, repoName!).then((r) => r.data),
        enabled:  !!owner && !!repoName,
    });

    const { data: repo } = useQuery({
        queryKey: ["repo", owner, repoName],
        queryFn:  () => reposApi.get(owner!, repoName!).then((r) => r.data),
        enabled:  !!owner && !!repoName,
    });

    const { data: diffData } = useQuery({
        queryKey: ["pr-preview-diff", owner, repoName, baseBranch, headBranch],
        queryFn:  () => reposApi.diff(owner!, repoName!, baseBranch, headBranch)
            .then((r) => r.data),
        enabled: !!baseBranch && !!headBranch && baseBranch !== headBranch,
    });

    const branches = refsData?.branches ?? [];

    const createPr = useMutation({
        mutationFn: () => apiClient.post(`/repos/${owner}/${repoName}/pulls`, {
            title,
            body,
            head:  headBranch,
            base:  baseBranch,
            draft: isDraft,
        }),
        onSuccess: (r) => {
            success("Pull request created");
            navigate(`/${owner}/${repoName}/pulls/${r.data.number}`);
        },
        onError: (e: any) => setError(e.response?.data?.error ?? "Failed to create pull request"),
    });

    const isValid = title.trim() && headBranch && baseBranch && headBranch !== baseBranch;

    if (refsLoading) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
                <Header />
                <PageSpinner />
            </div>
        );
    }

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
                    <Link to={`/${owner}/${repoName}/pulls`}
                          className="text-blue-600 dark:text-blue-400 hover:underline">
                        Pull requests
                    </Link>
                    <span className="text-gray-400">/</span>
                    <span className="text-gray-900 dark:text-white font-medium">New</span>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <div className="lg:col-span-2 space-y-4">
                        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6">
                            <h1 className="text-lg font-bold text-gray-900 dark:text-white mb-5 flex items-center gap-2">
                                <GitPullRequest className="w-5 h-5 text-green-500" />
                                Open a pull request
                            </h1>

                            {error && <Alert type="error" onClose={() => setError("")} className="mb-4">{error}</Alert>}

                            {/* branch selector */}
                            <div className="flex items-center gap-3 mb-6 flex-wrap">
                                <div className="flex-1 min-w-32">
                                    <label className="block text-xs font-medium text-gray-500 mb-1">Base branch</label>
                                    <select
                                        value={baseBranch}
                                        onChange={(e) => setBaseBranch(e.target.value)}
                                        className="w-full text-sm border border-gray-300 dark:border-gray-700 rounded-md px-3 py-2 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    >
                                        <option value="">Select base</option>
                                        {branches.map((b: any) => (
                                            <option key={b.name} value={b.name}>{b.name}</option>
                                        ))}
                                    </select>
                                </div>

                                <ArrowLeft className="w-4 h-4 text-gray-400 mt-5 flex-shrink-0" />

                                <div className="flex-1 min-w-32">
                                    <label className="block text-xs font-medium text-gray-500 mb-1">Compare branch</label>
                                    <select
                                        value={headBranch}
                                        onChange={(e) => setHeadBranch(e.target.value)}
                                        className="w-full text-sm border border-gray-300 dark:border-gray-700 rounded-md px-3 py-2 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    >
                                        <option value="">Select branch</option>
                                        {branches.map((b: any) => (
                                            <option key={b.name} value={b.name}>{b.name}</option>
                                        ))}
                                    </select>
                                </div>
                            </div>

                            {/* diff preview */}
                            {baseBranch && headBranch && baseBranch === headBranch && (
                                <Alert type="warning" className="mb-4">
                                    Head and base branches are the same. Choose different branches.
                                </Alert>
                            )}

                            {diffData && baseBranch !== headBranch && (
                                <div className="flex items-center gap-4 text-sm mb-5 p-3 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
                  <span className="flex items-center gap-1.5 text-green-700 dark:text-green-400">
                    <Info className="w-4 h-4" />
                    Able to merge
                  </span>
                                    <span className="text-gray-600 dark:text-gray-400">
                    {diffData.files_changed} file{diffData.files_changed !== 1 ? "s" : ""} changed
                  </span>
                                    <span className="text-green-600 dark:text-green-400">+{diffData.additions}</span>
                                    <span className="text-red-600 dark:text-red-400">-{diffData.deletions}</span>
                                </div>
                            )}

                            {/* form */}
                            <div className="space-y-4">
                                <Input
                                    label="Title"
                                    value={title}
                                    onChange={(e) => setTitle(e.target.value)}
                                    placeholder="PR title"
                                />

                                <div>
                                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                        Description
                                    </label>
                                    <textarea
                                        value={body}
                                        onChange={(e) => setBody(e.target.value)}
                                        rows={6}
                                        placeholder="Describe your changes..."
                                        className="w-full px-3 py-2 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono"
                                    />
                                </div>

                                <label className="flex items-center gap-2 cursor-pointer">
                                    <input
                                        type="checkbox"
                                        checked={isDraft}
                                        onChange={(e) => setIsDraft(e.target.checked)}
                                        className="rounded border-gray-300"
                                    />
                                    <div>
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                      Draft pull request
                    </span>
                                        <p className="text-xs text-gray-500">
                                            Mark as draft — cannot be merged until marked ready for review
                                        </p>
                                    </div>
                                </label>
                            </div>
                        </div>

                        <div className="flex items-center justify-between">
                            <Button
                                variant="ghost"
                                icon={<ArrowLeft className="w-4 h-4" />}
                                onClick={() => navigate(-1)}
                            >
                                Cancel
                            </Button>
                            <Button
                                loading={createPr.isPending}
                                disabled={!isValid}
                                icon={<GitPullRequest className="w-4 h-4" />}
                                onClick={() => createPr.mutate()}
                            >
                                {isDraft ? "Create draft pull request" : "Create pull request"}
                            </Button>
                        </div>
                    </div>

                    {/* sidebar tips */}
                    <aside className="space-y-4 text-sm">
                        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4">
                            <h3 className="font-semibold text-gray-700 dark:text-gray-300 mb-3">
                                Tips
                            </h3>
                            <ul className="space-y-2 text-gray-500 dark:text-gray-400 text-xs">
                                <li className="flex items-start gap-2">
                                    <span className="w-4 h-4 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded flex items-center justify-center text-xs flex-shrink-0 mt-0.5">1</span>
                                    Choose the base branch you want to merge into
                                </li>
                                <li className="flex items-start gap-2">
                                    <span className="w-4 h-4 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded flex items-center justify-center text-xs flex-shrink-0 mt-0.5">2</span>
                                    Select the branch with your changes to compare
                                </li>
                                <li className="flex items-start gap-2">
                                    <span className="w-4 h-4 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded flex items-center justify-center text-xs flex-shrink-0 mt-0.5">3</span>
                                    Write a clear title and describe your changes
                                </li>
                                <li className="flex items-start gap-2">
                                    <span className="w-4 h-4 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded flex items-center justify-center text-xs flex-shrink-0 mt-0.5">4</span>
                                    Use draft if the work is not ready for review yet
                                </li>
                            </ul>
                        </div>

                        {repo && (
                            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4">
                                <h3 className="font-semibold text-gray-700 dark:text-gray-300 mb-2">Merge methods</h3>
                                <div className="space-y-1.5 text-xs text-gray-500">
                                    {repo.allow_merge_commit  && <p>✓ Merge commit</p>}
                                    {repo.allow_squash_merge  && <p>✓ Squash and merge</p>}
                                    {repo.allow_rebase_merge  && <p>✓ Rebase and merge</p>}
                                </div>
                            </div>
                        )}
                    </aside>
                </div>
            </div>
        </div>
    );
}