import { useParams, NavLink, Routes, Route, Navigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
    BookOpen, GitBranch, CircleDot, GitPullRequest,
    Package, Play, Settings, Star, GitFork, Eye,
    Lock, Archive
} from "lucide-react";
import { Header } from "../../components/layout/Header";
import { Avatar } from "../../components/ui/Avatar";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { PageSpinner } from "../../components/ui/Spinner";
import { reposApi } from "../../api/repos";
import { usersApi } from "../../api/users";
import { useAuthStore } from "../../stores/auth";
import { cn } from "../../lib/utils";
import RepoHomePage     from "./RepoHomePage";
import RepoIssuesPage   from "./RepoIssuesPage";
import RepoPullsPage    from "./RepoPullsPage";
import RepoReleasesPage from "./RepoReleasesPage";
import RepoActionsPage  from "./RepoActionsPage";
import RepoSettingsPage from "./RepoSettingsPage";

const tabs = [
    { label: "Code",     icon: <BookOpen className="w-4 h-4" />,        path: "",         end: true },
    { label: "Issues",   icon: <CircleDot className="w-4 h-4" />,       path: "issues" },
    { label: "Pull requests", icon: <GitPullRequest className="w-4 h-4" />, path: "pulls" },
    { label: "Actions",  icon: <Play className="w-4 h-4" />,            path: "actions" },
    { label: "Releases", icon: <Package className="w-4 h-4" />,         path: "releases" },
    { label: "Settings", icon: <Settings className="w-4 h-4" />,        path: "settings" },
];

export default function RepoLayout() {
    const { owner, repo: repoName } = useParams<{ owner: string; repo: string }>();
    const currentUser = useAuthStore((s) => s.user);

    const { data: repo, isLoading } = useQuery({
        queryKey: ["repo", owner, repoName],
        queryFn:  () => reposApi.get(owner!, repoName!).then((r) => r.data),
        enabled:  !!owner && !!repoName,
    });

    const { data: ownerData } = useQuery({
        queryKey: ["user", owner],
        queryFn:  () => usersApi.get(owner!).then((r) => r.data),
        enabled:  !!owner,
    });

    if (isLoading) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
                <Header />
                <PageSpinner />
            </div>
        );
    }

    if (!repo) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
                <Header />
                <div className="max-w-7xl mx-auto px-4 py-16 text-center text-gray-500">
                    Repository not found
                </div>
            </div>
        );
    }

    const isOwner = currentUser?.id === repo.owner_id;
    const base    = `/${owner}/${repoName}`;

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            <Header />

            {/* repo header */}
            <div className="border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950">
                <div className="max-w-7xl mx-auto px-4 pt-4 pb-0">

                    {/* breadcrumb + name */}
                    <div className="flex items-center gap-2 mb-1 flex-wrap">
                        <BookOpen className="w-4 h-4 text-gray-400" />
                        <a href={`/${owner}`}
                           className="text-blue-600 dark:text-blue-400 hover:underline font-medium">
                            {owner}
                        </a>
                        <span className="text-gray-400">/</span>
                        <a href={base}
                           className="text-blue-600 dark:text-blue-400 hover:underline font-bold text-lg">
                            {repoName}
                        </a>
                        <Badge variant={repo.visibility === "Public" ? "success" : "default"} size="sm">
                            {repo.visibility === "Public" ? (
                                <><Eye className="w-3 h-3" /> Public</>
                            ) : (
                                <><Lock className="w-3 h-3" /> Private</>
                            )}
                        </Badge>
                        {repo.is_fork    && <Badge size="sm"><GitFork className="w-3 h-3" /> Fork</Badge>}
                        {repo.is_archived && <Badge variant="warning" size="sm"><Archive className="w-3 h-3" /> Archived</Badge>}
                    </div>

                    {repo.description && (
                        <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                            {repo.description}
                        </p>
                    )}

                    {/* action buttons */}
                    <div className="flex items-center gap-2 mb-4 flex-wrap">
                        <Button variant="outline" size="sm" icon={<Eye className="w-3.5 h-3.5" />}>
                            Watch <Badge size="sm" className="ml-1">{repo.watcher_count}</Badge>
                        </Button>
                        <Button variant="outline" size="sm" icon={<GitFork className="w-3.5 h-3.5" />}>
                            Fork <Badge size="sm" className="ml-1">{repo.fork_count}</Badge>
                        </Button>
                        <Button variant="outline" size="sm" icon={<Star className="w-3.5 h-3.5" />}>
                            Star <Badge size="sm" className="ml-1">{repo.star_count}</Badge>
                        </Button>
                    </div>

                    {/* tabs */}
                    <nav className="flex gap-1 -mb-px overflow-x-auto">
                        {tabs.map((tab) => (
                            <NavLink
                                key={tab.path}
                                to={tab.path ? `${base}/${tab.path}` : base}
                                end={tab.end}
                                className={({ isActive }) => cn(
                                    "flex items-center gap-1.5 px-3 py-2.5 text-sm font-medium border-b-2 whitespace-nowrap transition-colors",
                                    isActive
                                        ? "border-orange-500 text-gray-900 dark:text-white"
                                        : "border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white hover:border-gray-300"
                                )}
                            >
                                {tab.icon}
                                {tab.label}
                                {tab.label === "Issues" && repo.open_issue_count > 0 && (
                                    <Badge size="sm">{repo.open_issue_count}</Badge>
                                )}
                            </NavLink>
                        ))}
                    </nav>
                </div>
            </div>

            {/* page content */}
            <div className="max-w-7xl mx-auto px-4 py-6">
                <Routes>
                    <Route index          element={<RepoHomePage repo={repo} owner={owner!} />} />
                    <Route path="issues"  element={<RepoIssuesPage repo={repo} owner={owner!} />} />
                    <Route path="pulls"   element={<RepoPullsPage repo={repo} owner={owner!} />} />
                    <Route path="actions" element={<RepoActionsPage repo={repo} owner={owner!} />} />
                    <Route path="releases" element={<RepoReleasesPage repo={repo} owner={owner!} />} />
                    <Route path="settings" element={<RepoSettingsPage repo={repo} owner={owner!} />} />
                    <Route path="*"       element={<Navigate to={base} replace />} />
                </Routes>
            </div>
        </div>
    );
}