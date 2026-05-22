import { useSearchParams, Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { Search, BookOpen, CircleDot, GitPullRequest, User, MessageSquare } from "lucide-react";
import { PageLayout } from "../../components/layout/PageLayout";
import { Badge } from "../../components/ui/Badge";
import { PageSpinner } from "../../components/ui/Spinner";
import { Tabs } from "../../components/ui/Tabs";
import { searchApi } from "../../api/search";
import { relativeTime, cn } from "../../lib/utils";
import { useState } from "react";

export default function SearchPage() {
    const [searchParams, setSearchParams] = useSearchParams();
    const q    = searchParams.get("q") ?? "";
    const type = searchParams.get("type") ?? "repositories";

    const [tab, setTab] = useState(type);

    const changeTab = (t: string) => {
        setTab(t);
        setSearchParams({ q, type: t });
    };

    const { data: repoData, isLoading: repoLoading } = useQuery({
        queryKey: ["search-repos", q],
        queryFn:  () => searchApi.repos(q).then((r) => r.data),
        enabled:  !!q,
    });

    const { data: issueData, isLoading: issueLoading } = useQuery({
        queryKey: ["search-issues", q],
        queryFn:  () => searchApi.issues(q).then((r) => r.data),
        enabled:  !!q,
    });

    const { data: prData, isLoading: prLoading } = useQuery({
        queryKey: ["search-prs", q],
        queryFn:  () => searchApi.pulls(q).then((r) => r.data),
        enabled:  !!q,
    });

    const { data: userData, isLoading: userLoading } = useQuery({
        queryKey: ["search-users", q],
        queryFn:  () => searchApi.users(q).then((r) => r.data),
        enabled:  !!q,
    });

    const tabs = [
        { key: "repositories", label: "Repositories", icon: <BookOpen className="w-4 h-4" />,       count: repoData?.total },
        { key: "issues",       label: "Issues",        icon: <CircleDot className="w-4 h-4" />,       count: issueData?.total },
        { key: "pullrequests", label: "Pull requests", icon: <GitPullRequest className="w-4 h-4" />,  count: prData?.total },
        { key: "users",        label: "Users",          icon: <User className="w-4 h-4" />,            count: userData?.total },
    ];

    if (!q) {
        return (
            <PageLayout>
                <div className="text-center py-20">
                    <Search className="w-12 h-12 mx-auto text-gray-200 mb-4" />
                    <p className="text-gray-500">Search for repositories, issues, pull requests, and users</p>
                </div>
            </PageLayout>
        );
    }

    return (
        <PageLayout>
            <div className="mb-6">
                <h1 className="text-lg font-bold text-gray-900 dark:text-white mb-1">
                    Search results for <span className="text-blue-600 dark:text-blue-400">"{q}"</span>
                </h1>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-5 gap-6">
                {/* left tabs */}
                <aside className="lg:col-span-1">
                    <nav className="space-y-1">
                        {tabs.map((t) => (
                            <button
                                key={t.key}
                                onClick={() => changeTab(t.key)}
                                className={cn(
                                    "w-full flex items-center justify-between px-3 py-2 rounded-lg text-sm font-medium transition-colors",
                                    tab === t.key
                                        ? "bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-400"
                                        : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800"
                                )}
                            >
                <span className="flex items-center gap-2">
                  {t.icon}
                    {t.label}
                </span>
                                {t.count !== undefined && (
                                    <Badge size="sm">{t.count.toLocaleString()}</Badge>
                                )}
                            </button>
                        ))}
                    </nav>
                </aside>

                {/* results */}
                <div className="lg:col-span-4 space-y-3">
                    {tab === "repositories" && (
                        repoLoading ? <PageSpinner /> :
                            (repoData?.repositories ?? []).length === 0 ? (
                                <EmptyResults query={q} type="repositories" />
                            ) : (
                                (repoData?.repositories ?? []).map((r: any) => (
                                    <Link key={r.id} to={`/${r.owner}/${r.name}`}
                                          className="block bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4 hover:border-blue-300 dark:hover:border-blue-700 transition-colors">
                                        <div className="flex items-start gap-2">
                                            <BookOpen className="w-4 h-4 text-gray-400 mt-0.5 flex-shrink-0" />
                                            <div>
                                                <p className="text-sm font-semibold text-blue-600 dark:text-blue-400">
                                                    {r.owner}/{r.name}
                                                </p>
                                                {r.description && (
                                                    <p className="text-xs text-gray-600 dark:text-gray-400 mt-1">{r.description}</p>
                                                )}
                                                <div className="flex items-center gap-3 mt-2 text-xs text-gray-500">
                                                    <span>⭐ {r.stars}</span>
                                                    <span>Updated {relativeTime(r.updated_at)}</span>
                                                </div>
                                            </div>
                                        </div>
                                    </Link>
                                ))
                            )
                    )}

                    {tab === "issues" && (
                        issueLoading ? <PageSpinner /> :
                            (issueData?.issues ?? []).length === 0 ? (
                                <EmptyResults query={q} type="issues" />
                            ) : (
                                (issueData?.issues ?? []).map((i: any) => (
                                    <a key={i.id}
                                       href={`/${i.owner}/${i.repo_name}/issues/${i.number}`}
                                       className="block bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4 hover:border-blue-300 transition-colors">
                                        <div className="flex items-start gap-2">
                                            <CircleDot className={cn(
                                                "w-4 h-4 mt-0.5 flex-shrink-0",
                                                i.state === "open" ? "text-green-500" : "text-purple-500"
                                            )} />
                                            <div>
                                                <p className="text-sm font-semibold text-gray-900 dark:text-white">{i.title}</p>
                                                <p className="text-xs text-gray-500 mt-1">
                                                    {i.owner}/{i.repo_name} · #{i.number} · {relativeTime(i.created_at)}
                                                </p>
                                            </div>
                                        </div>
                                    </a>
                                ))
                            )
                    )}

                    {tab === "pullrequests" && (
                        prLoading ? <PageSpinner /> :
                            (prData?.pull_requests ?? []).length === 0 ? (
                                <EmptyResults query={q} type="pull requests" />
                            ) : (
                                (prData?.pull_requests ?? []).map((p: any) => (
                                    <a key={p.id}
                                       href={`/${p.owner}/${p.repo_name}/pulls/${p.number}`}
                                       className="block bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4 hover:border-blue-300 transition-colors">
                                        <div className="flex items-start gap-2">
                                            <GitPullRequest className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                                            <div>
                                                <p className="text-sm font-semibold text-gray-900 dark:text-white">{p.title}</p>
                                                <p className="text-xs text-gray-500 mt-1">
                                                    {p.owner}/{p.repo_name} · #{p.number} · {relativeTime(p.created_at)}
                                                </p>
                                            </div>
                                        </div>
                                    </a>
                                ))
                            )
                    )}

                    {tab === "users" && (
                        userLoading ? <PageSpinner /> :
                            (userData?.users ?? []).length === 0 ? (
                                <EmptyResults query={q} type="users" />
                            ) : (
                                (userData?.users ?? []).map((u: any) => (
                                    <Link key={u.id} to={`/${u.username}`}
                                          className="flex items-center gap-3 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4 hover:border-blue-300 transition-colors">
                                        <div className="w-10 h-10 rounded-full bg-blue-500 flex items-center justify-center text-white font-bold flex-shrink-0">
                                            {u.username.slice(0, 2).toUpperCase()}
                                        </div>
                                        <div>
                                            <p className="text-sm font-semibold text-blue-600 dark:text-blue-400">
                                                {u.username}
                                            </p>
                                            {u.display_name && (
                                                <p className="text-xs text-gray-500">{u.display_name}</p>
                                            )}
                                            {u.bio && (
                                                <p className="text-xs text-gray-500 line-clamp-1">{u.bio}</p>
                                            )}
                                        </div>
                                    </Link>
                                ))
                            )
                    )}
                </div>
            </div>
        </PageLayout>
    );
}

function EmptyResults({ query, type }: { query: string; type: string }) {
    return (
        <div className="text-center py-16 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800">
            <Search className="w-10 h-10 mx-auto text-gray-200 mb-3" />
            <p className="text-gray-500 text-sm">
                No {type} matching <strong>"{query}"</strong>
            </p>
        </div>
    );
}