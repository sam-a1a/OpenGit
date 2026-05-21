import { useQuery } from "@tanstack/react-query";
import { Play, CheckCircle2, XCircle, Clock, RotateCw } from "lucide-react";
import { Badge } from "../../components/ui/Badge";
import { PageSpinner } from "../../components/ui/Spinner";
import { apiClient } from "../../api/client";
import { relativeTime, cn } from "../../lib/utils";
import type { Repo } from "../../types/repo";

interface Props { repo: Repo; owner: string; }

export default function RepoActionsPage({ repo, owner }: Props) {
    const { data, isLoading } = useQuery({
        queryKey: ["runs", owner, repo.name],
        queryFn:  () => apiClient.get(`/repos/${owner}/${repo.name}/actions/runs`)
            .then((r) => r.data),
    });

    const runs = data?.workflow_runs ?? [];

    const statusIcon = (status: string, conclusion: string | null) => {
        if (status === "completed") {
            if (conclusion === "success")   return <CheckCircle2 className="w-5 h-5 text-green-500" />;
            if (conclusion === "failure")   return <XCircle className="w-5 h-5 text-red-500" />;
            if (conclusion === "cancelled") return <XCircle className="w-5 h-5 text-gray-400" />;
        }
        if (status === "in_progress") return (
            <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
        );
        return <Clock className="w-5 h-5 text-yellow-500" />;
    };

    const statusBadge = (status: string, conclusion: string | null) => {
        if (status === "in_progress") return <Badge variant="info" dot>In progress</Badge>;
        if (status === "queued")      return <Badge variant="warning" dot>Queued</Badge>;
        if (conclusion === "success") return <Badge variant="success" dot>Success</Badge>;
        if (conclusion === "failure") return <Badge variant="danger" dot>Failed</Badge>;
        return <Badge dot>{conclusion ?? status}</Badge>;
    };

    if (isLoading) return <PageSpinner />;

    return (
        <div className="space-y-4">
            <div className="flex items-center justify-between">
                <h2 className="text-sm font-semibold text-gray-900 dark:text-white">
                    Workflow runs
                </h2>
            </div>

            {runs.length === 0 ? (
                <div className="text-center py-16 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800">
                    <Play className="w-12 h-12 mx-auto text-gray-200 mb-4" />
                    <p className="text-gray-500">No workflow runs yet</p>
                </div>
            ) : (
                <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 divide-y divide-gray-100 dark:divide-gray-800">
                    {runs.map((run: any) => (
                        <div key={run.id} className="flex items-center gap-4 px-4 py-3">
                            <div className="flex-shrink-0">
                                {statusIcon(run.status, run.conclusion)}
                            </div>
                            <div className="min-w-0 flex-1">
                                <div className="flex items-center gap-2 flex-wrap">
                  <span className="text-sm font-medium text-gray-900 dark:text-white truncate">
                    {run.event} — {run.head_branch ?? "unknown branch"}
                  </span>
                                    {statusBadge(run.status, run.conclusion)}
                                </div>
                                <p className="text-xs text-gray-500 mt-0.5">
                                    Run #{run.run_number} · {relativeTime(run.created_at)}
                                    {run.head_sha && ` · ${run.head_sha.slice(0, 7)}`}
                                </p>
                            </div>
                            <button className="p-1.5 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-400 flex-shrink-0">
                                <RotateCw className="w-4 h-4" />
                            </button>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}