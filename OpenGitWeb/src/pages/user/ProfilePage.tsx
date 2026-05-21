import { useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
    MapPin, Link2, Twitter, Calendar, Users, Star,
    GitFork, BookOpen, Building
} from "lucide-react";
import { PageLayout } from "../../components/layout/PageLayout";
import { Avatar } from "../../components/ui/Avatar";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { PageSpinner } from "../../components/ui/Spinner";
import { usersApi } from "../../api/users";
import { reposApi } from "../../api/repos";
import { useAuthStore } from "../../stores/auth";
import { formatDate, relativeTime } from "../../lib/utils";
import type { Repo } from "../../types/repo";

export default function ProfilePage() {
    const { username }  = useParams<{ username: string }>();
    const currentUser   = useAuthStore((s) => s.user);
    const isOwnProfile  = currentUser?.username === username;

    const { data: user, isLoading } = useQuery({
        queryKey: ["user", username],
        queryFn:  () => usersApi.get(username!).then((r) => r.data),
        enabled:  !!username,
    });

    const { data: reposData } = useQuery({
        queryKey: ["user-repos", username],
        queryFn:  () => usersApi.repos(username!).then((r) => r.data),
        enabled:  !!username,
    });

    if (isLoading) return <PageLayout><PageSpinner /></PageLayout>;
    if (!user) return <PageLayout><div className="text-center py-16 text-gray-500">User not found</div></PageLayout>;

    const repos: Repo[] = reposData ?? [];

    return (
        <PageLayout>
            <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">

                {/* sidebar */}
                <aside className="lg:col-span-1">
                    <div className="sticky top-20">
                        <Avatar
                            src={user.avatar_url}
                            username={user.username}
                            size="xl"
                            className="mb-4 ring-4 ring-white dark:ring-gray-900"
                        />

                        <h1 className="text-xl font-bold text-gray-900 dark:text-white">
                            {user.display_name ?? user.username}
                        </h1>
                        {user.display_name && (
                            <p className="text-gray-500 text-lg">{user.username}</p>
                        )}

                        {user.status_emoji || user.status_message ? (
                            <div className="mt-2 flex items-center gap-1.5 text-sm text-gray-600 dark:text-gray-400">
                                {user.status_emoji && <span>{user.status_emoji}</span>}
                                {user.status_message && <span>{user.status_message}</span>}
                            </div>
                        ) : null}

                        {isOwnProfile ? (
                            <Button variant="outline" size="sm" className="mt-3 w-full">
                                Edit profile
                            </Button>
                        ) : (
                            <Button size="sm" className="mt-3 w-full">
                                Follow
                            </Button>
                        )}

                        {user.bio && (
                            <p className="mt-4 text-sm text-gray-600 dark:text-gray-400">
                                {user.bio}
                            </p>
                        )}

                        <div className="mt-4 space-y-2 text-sm text-gray-600 dark:text-gray-400">
                            {user.company && (
                                <div className="flex items-center gap-2">
                                    <Building className="w-4 h-4 flex-shrink-0" />
                                    <span>{user.company}</span>
                                </div>
                            )}
                            {user.location && (
                                <div className="flex items-center gap-2">
                                    <MapPin className="w-4 h-4 flex-shrink-0" />
                                    <span>{user.location}</span>
                                </div>
                            )}
                            {user.website && (
                                <div className="flex items-center gap-2">
                                    <Link2 className="w-4 h-4 flex-shrink-0" />
                                    <a href={user.website} target="_blank" rel="noopener noreferrer"
                                       className="text-blue-600 dark:text-blue-400 hover:underline truncate">
                                        {user.website.replace(/^https?:\/\//, "")}
                                    </a>
                                </div>
                            )}
                            {user.twitter_username && (
                                <div className="flex items-center gap-2">
                                    <Twitter className="w-4 h-4 flex-shrink-0" />
                                    <a href={`https://twitter.com/${user.twitter_username}`}
                                       target="_blank" rel="noopener noreferrer"
                                       className="text-blue-600 dark:text-blue-400 hover:underline">
                                        @{user.twitter_username}
                                    </a>
                                </div>
                            )}
                            <div className="flex items-center gap-2">
                                <Calendar className="w-4 h-4 flex-shrink-0" />
                                <span>Joined {formatDate(user.created_at)}</span>
                            </div>
                        </div>
                    </div>
                </aside>

                {/* main content */}
                <main className="lg:col-span-3 space-y-4">
                    <div className="flex items-center justify-between">
                        <h2 className="font-semibold text-gray-900 dark:text-white">
                            Repositories
                        </h2>
                        <Badge>{repos.length}</Badge>
                    </div>

                    {repos.length === 0 ? (
                        <div className="text-center py-12 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 text-gray-500">
                            <BookOpen className="w-10 h-10 mx-auto mb-3 opacity-30" />
                            No public repositories
                        </div>
                    ) : (
                        <div className="space-y-3">
                            {repos.map((repo) => (
                                <div key={repo.id}
                                     className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4">
                                    <div className="flex items-start justify-between gap-3">
                                        <div className="min-w-0">
                                            <div className="flex items-center gap-2 flex-wrap">
                                                <a href={`/${username}/${repo.name}`}
                                                   className="font-semibold text-blue-600 dark:text-blue-400 hover:underline">
                                                    {repo.name}
                                                </a>
                                                <Badge size="sm" variant={repo.visibility === "Public" ? "success" : "default"}>
                                                    {repo.visibility}
                                                </Badge>
                                                {repo.is_fork && <Badge size="sm">Fork</Badge>}
                                            </div>
                                            {repo.description && (
                                                <p className="text-sm text-gray-600 dark:text-gray-400 mt-1 line-clamp-2">
                                                    {repo.description}
                                                </p>
                                            )}
                                        </div>
                                        <Button variant="outline" size="sm" className="flex-shrink-0">
                                            <Star className="w-3.5 h-3.5" />
                                            {repo.star_count}
                                        </Button>
                                    </div>
                                    <div className="mt-3 flex items-center gap-4 text-xs text-gray-500">
                    <span className="flex items-center gap-1">
                      <GitFork className="w-3 h-3" />
                        {repo.fork_count}
                    </span>
                                        {repo.pushed_at && (
                                            <span>Updated {relativeTime(repo.pushed_at)}</span>
                                        )}
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </main>
            </div>
        </PageLayout>
    );
}