import { useState } from "react";
import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { Search, Star, GitFork, Eye, TrendingUp } from "lucide-react";
import { PageLayout } from "../../components/layout/PageLayout";
import { Badge } from "../../components/ui/Badge";
import { Spinner } from "../../components/ui/Spinner";
import { reposApi } from "../../api/repos";
import { searchApi } from "../../api/search";
import { useDebounce } from "../../hooks/useDebounce";
import { relativeTime } from "../../lib/utils";
import type { Repo } from "../../types/repo";

export default function ExplorePage() {
    const [query, setQuery] = useState("");
    const debounced         = useDebounce(query, 400);

    const { data: trending, isLoading: trendingLoading } = useQuery({
        queryKey: ["repos", "trending"],
        queryFn:  () => reposApi.search("", 1).then((r) => r.data.repositories ?? []),
    });

    const { data: searchResults, isLoading: searching } = useQuery({
        queryKey: ["search", debounced],
        queryFn:  () => searchApi.repos(debounced).then((r) => r.data.repositories ?? []),
        enabled:  !!debounced,
    });

    const repos: Repo[] = debounced
        ? (searchResults ?? [])
        : (trending ?? []);

    const loading = debounced ? searching : trendingLoading;

    return (
        <PageLayout>
            <div className="max-w-4xl mx-auto">

                {/* hero */}
                <div className="text-center mb-10">
                    <div className="inline-flex items-center gap-2 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 px-3 py-1 rounded-full text-sm font-medium mb-4">
                        <TrendingUp className="w-4 h-4" />
                        Explore
                    </div>
                    <h1 className="text-4xl font-bold text-gray-900 dark:text-white mb-3">
                        Discover repositories
                    </h1>
                    <p className="text-gray-500 dark:text-gray-400 text-lg">
                        Find open source projects to use and contribute to
                    </p>
                </div>

                {/* search */}
                <div className="relative mb-8">
                    <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
                    <input
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder="Search repositories..."
                        className="w-full pl-12 pr-4 py-3 text-base rounded-xl border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 shadow-sm"
                    />
                </div>

                {/* results */}
                {loading ? (
                    <div className="flex justify-center py-16">
                        <Spinner className="w-8 h-8" />
                    </div>
                ) : repos.length === 0 ? (
                    <div className="text-center py-16 text-gray-500">
                        {debounced ? `No results for "${debounced}"` : "No repositories yet"}
                    </div>
                ) : (
                    <div className="space-y-3">
                        {!debounced && (
                            <h2 className="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-4">
                                Trending repositories
                            </h2>
                        )}
                        {repos.map((repo) => (
                            <RepoCard key={repo.id} repo={repo} />
                        ))}
                    </div>
                )}
            </div>
        </PageLayout>
    );
}

function RepoCard({ repo }: { repo: Repo }) {
    return (
        <Link
            to={`/${repo.owner_id}/${repo.name}`}
            className="block bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl p-5 hover:border-blue-300 dark:hover:border-blue-700 hover:shadow-sm transition-all"
        >
            <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                    <div className="flex items-center gap-2 mb-1 flex-wrap">
            <span className="font-semibold text-blue-600 dark:text-blue-400 hover:underline truncate">
              {repo.name}
            </span>
                        <Badge variant={repo.visibility === "Public" ? "success" : "default"} size="sm">
                            {repo.visibility}
                        </Badge>
                        {repo.is_fork && <Badge size="sm">Fork</Badge>}
                        {repo.is_archived && <Badge variant="warning" size="sm">Archived</Badge>}
                    </div>
                    {repo.description && (
                        <p className="text-sm text-gray-600 dark:text-gray-400 mb-3 line-clamp-2">
                            {repo.description}
                        </p>
                    )}
                    <div className="flex items-center gap-4 text-xs text-gray-500">
            <span className="flex items-center gap-1">
              <Star className="w-3.5 h-3.5" />
                {repo.star_count.toLocaleString()}
            </span>
                        <span className="flex items-center gap-1">
              <GitFork className="w-3.5 h-3.5" />
                            {repo.fork_count.toLocaleString()}
            </span>
                        <span className="flex items-center gap-1">
              <Eye className="w-3.5 h-3.5" />
                            {repo.watcher_count.toLocaleString()}
            </span>
                        {repo.pushed_at && (
                            <span>Updated {relativeTime(repo.pushed_at)}</span>
                        )}
                    </div>
                </div>
            </div>
        </Link>
    );
}