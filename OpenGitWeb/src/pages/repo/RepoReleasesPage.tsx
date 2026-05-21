import { useQuery } from "@tanstack/react-query";
import { Package, Download, Tag } from "lucide-react";
import { Badge } from "../../components/ui/Badge";
import { PageSpinner } from "../../components/ui/Spinner";
import { apiClient } from "../../api/client";
import { formatDate } from "../../lib/utils";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { Repo } from "../../types/repo";

interface Props { repo: Repo; owner: string; }

export default function RepoReleasesPage({ repo, owner }: Props) {
    const { data, isLoading } = useQuery({
        queryKey: ["releases", owner, repo.name],
        queryFn:  () => apiClient.get(`/repos/${owner}/${repo.name}/releases`)
            .then((r) => r.data),
    });

    const releases = data ?? [];

    if (isLoading) return <PageSpinner />;

    if (releases.length === 0) {
        return (
            <div className="text-center py-16 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800">
                <Package className="w-12 h-12 mx-auto text-gray-200 mb-4" />
                <p className="text-gray-500">No releases yet</p>
            </div>
        );
    }

    return (
        <div className="space-y-6">
            {releases.map((release: any) => (
                <div key={release.id}
                     className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6">
                    <div className="flex items-start justify-between gap-4 mb-3">
                        <div>
                            <div className="flex items-center gap-2 flex-wrap mb-1">
                                <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                                    {release.name ?? release.tag_name}
                                </h3>
                                {release.is_latest    && <Badge variant="success">Latest</Badge>}
                                {release.is_prerelease && <Badge variant="warning">Pre-release</Badge>}
                                {release.is_draft     && <Badge>Draft</Badge>}
                            </div>
                            <span className="flex items-center gap-1 text-sm text-gray-500">
                <Tag className="w-3.5 h-3.5" />
                                {release.tag_name} · {formatDate(release.published_at ?? release.created_at)}
              </span>
                        </div>
                    </div>

                    {release.body && (
                        <div className="prose dark:prose-invert prose-sm max-w-none mb-4 border-t border-gray-100 dark:border-gray-800 pt-4">
                            <ReactMarkdown remarkPlugins={[remarkGfm]}>
                                {release.body}
                            </ReactMarkdown>
                        </div>
                    )}

                    {release.assets?.length > 0 && (
                        <div className="border-t border-gray-100 dark:border-gray-800 pt-4">
                            <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                                Assets ({release.assets.length})
                            </h4>
                            <div className="space-y-1">
                                {release.assets.map((asset: any) => (

                                    key={asset.id}
                                    href={`/api/v1/repos/${owner}/${repo.name}/releases/${release.id}/assets/${asset.id}`}
                                    className="flex items-center gap-2 text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                    >
                                    <Download className="w-3.5 h-3.5" />
                                {asset.name}
                                    <span className="text-gray-400 text-xs">
                                    ({(asset.size_bytes / 1024 / 1024).toFixed(1)} MB)
                            </span>
                        </a>
                        ))}
                </div>
                </div>
                )}
        </div>
    ))}
</div>
);
}