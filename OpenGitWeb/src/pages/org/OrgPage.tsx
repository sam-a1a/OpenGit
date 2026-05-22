import { useParams, NavLink, Routes, Route } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { Users, BookOpen } from "lucide-react";
import { Header } from "../../components/layout/Header";
import { Avatar } from "../../components/ui/Avatar";
import { Badge } from "../../components/ui/Badge";
import { PageSpinner } from "../../components/ui/Spinner";
import { apiClient } from "../../api/client";
import { cn, relativeTime } from "../../lib/utils";

export default function OrgPage() {
    const { org: orgName } = useParams<{ org: string }>();

    const { data: org, isLoading } = useQuery({
        queryKey: ["org", orgName],
        queryFn:  () => apiClient.get(`/orgs/${orgName}`).then((r) => r.data),
        enabled:  !!orgName,
    });

    const { data: repos } = useQuery({
        queryKey: ["org-repos", orgName],
        queryFn:  () => apiClient.get(`/orgs/${orgName}/repos`).then((r) => r.data),
        enabled:  !!orgName,
    });

    const { data: members } = useQuery({
        queryKey: ["org-members", orgName],
        queryFn:  () => apiClient.get(`/orgs/${orgName}/members`).then((r) => r.data),
        enabled:  !!orgName,
    });

    if (isLoading) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
                <Header />
                <PageSpinner />
            </div>
        );
    }

    if (!org) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
                <Header />
                <div className="max-w-4xl mx-auto px-4 py-16 text-center text-gray-500">
                    Organization not found
                </div>
            </div>
        );
    }

    const tabs = [
        { path: "",         label: "Repositories", icon: <BookOpen className="w-4 h-4" /> },
        { path: "people",   label: "People",        icon: <Users className="w-4 h-4" /> },
    ];

    const base = `/orgs/${orgName}`;

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            <Header />

            {/* org header */}
            <div className="border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950">
                <div className="max-w-7xl mx-auto px-4 py-6">
                    <div className="flex items-start gap-5">
                        <Avatar username={org.name} src={org.avatar_url} size="xl"
                                className="ring-4 ring-white dark:ring-gray-900 flex-shrink-0" />
                        <div>
                            <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
                                {org.display_name ?? org.name}
                            </h1>
                            <p className="text-gray-500">{org.name}</p>
                            {org.description && (
                                <p className="text-sm text-gray-600 dark:text-gray-400 mt-2 max-w-xl">
                                    {org.description}
                                </p>
                            )}
                            <div className="flex items-center gap-4 mt-2 text-sm text-gray-500 flex-wrap">
                                {org.location && <span>📍 {org.location}</span>}
                                {org.website  && (
                                    <a href={org.website} target="_blank" rel="noopener noreferrer"
                                       className="text-blue-600 dark:text-blue-400 hover:underline">
                                        🔗 {org.website.replace(/^https?:\/\//, "")}
                                    </a>
                                )}
                            </div>
                        </div>
                    </div>

                    <nav className="flex gap-1 mt-4 -mb-px">
                        {tabs.map((tab) => (
                            <NavLink
                                key={tab.path}
                                to={tab.path ? `${base}/${tab.path}` : base}
                                end={!tab.path}
                                className={({ isActive }) => cn(
                                    "flex items-center gap-1.5 px-3 py-2.5 text-sm font-medium border-b-2 whitespace-nowrap transition-colors",
                                    isActive
                                        ? "border-orange-500 text-gray-900 dark:text-white"
                                        : "border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                                )}
                            >
                                {tab.icon}
                                {tab.label}
                                {tab.label === "Repositories" && repos?.length > 0 && (
                                    <Badge size="sm">{repos.length}</Badge>
                                )}
                                {tab.label === "People" && members?.length > 0 && (
                                    <Badge size="sm">{members.length}</Badge>
                                )}
                            </NavLink>
                        ))}
                    </nav>
                </div>
            </div>

            {/* content */}
            <div className="max-w-7xl mx-auto px-4 py-6">
                <Routes>
                    <Route index element={<OrgRepos repos={repos ?? []} />} />
                    <Route path="people" element={<OrgMembers members={members ?? []} />} />
                </Routes>
            </div>
        </div>
    );
}

function OrgRepos({ repos }: { repos: any[] }) {
    if (!repos.length) {
        return (
            <div className="text-center py-16 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 text-gray-500">
                <BookOpen className="w-10 h-10 mx-auto text-gray-200 mb-3" />
                No public repositories
            </div>
        );
    }

    return (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {repos.map((repo: any) => (
                <a key={repo.id} href={`/${repo.owner_id}/${repo.name}`}
                   className="block bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl p-4 hover:border-blue-300 dark:hover:border-blue-700 transition-colors">
                    <div className="flex items-start gap-2 mb-2">
                        <BookOpen className="w-4 h-4 text-gray-400 mt-0.5 flex-shrink-0" />
                        <p className="text-sm font-semibold text-blue-600 dark:text-blue-400">{repo.name}</p>
                        <Badge size="sm" variant={repo.visibility === "Public" ? "success" : "default"} className="ml-auto">
                            {repo.visibility}
                        </Badge>
                    </div>
                    {repo.description && (
                        <p className="text-xs text-gray-600 dark:text-gray-400 mb-3 line-clamp-2">
                            {repo.description}
                        </p>
                    )}
                    <div className="flex items-center gap-3 text-xs text-gray-500">
                        <span>⭐ {repo.star_count}</span>
                        {repo.pushed_at && <span>Updated {relativeTime(repo.pushed_at)}</span>}
                    </div>
                </a>
            ))}
        </div>
    );
}

function OrgMembers({ members }: { members: any[] }) {
    if (!members.length) {
        return (
            <div className="text-center py-16 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 text-gray-500">
                <Users className="w-10 h-10 mx-auto text-gray-200 mb-3" />
                No public members
            </div>
        );
    }

    return (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
            {members.map((user: any) => (
                <a key={user.id} href={`/${user.username}`}
                   className="flex flex-col items-center gap-2 p-3 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 hover:border-blue-300 transition-colors text-center">
                    <Avatar username={user.username} src={user.avatar_url} size="lg" />
                    <p className="text-xs font-medium text-gray-900 dark:text-white truncate w-full text-center">
                        {user.username}
                    </p>
                </a>
            ))}
        </div>
    );
}