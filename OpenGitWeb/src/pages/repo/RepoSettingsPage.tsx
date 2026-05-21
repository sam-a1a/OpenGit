import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Settings, Trash2, AlertTriangle } from "lucide-react";
import { Button } from "../../components/ui/Button";
import { Input } from "../../components/ui/Input";
import { Alert } from "../../components/ui/Alert";
import { reposApi } from "../../api/repos";
import { useAuthStore } from "../../stores/auth";
import { useNavigate } from "react-router-dom";
import type { Repo } from "../../types/repo";

interface Props { repo: Repo; owner: string; }

export default function RepoSettingsPage({ repo, owner }: Props) {
    const currentUser  = useAuthStore((s) => s.user);
    const navigate     = useNavigate();
    const queryClient  = useQueryClient();
    const isOwner      = currentUser?.id === repo.owner_id;

    const [name,        setName]        = useState(repo.name);
    const [description, setDescription] = useState(repo.description ?? "");
    const [deleteConf,  setDeleteConf]  = useState("");
    const [error,       setError]       = useState("");
    const [success,     setSuccess]     = useState("");

    const updateMutation = useMutation({
        mutationFn: () => reposApi.update(owner, repo.name, { name, description }),
        onSuccess:  () => {
            setSuccess("Settings saved");
            queryClient.invalidateQueries({ queryKey: ["repo", owner, repo.name] });
            if (name !== repo.name) navigate(`/${owner}/${name}/settings`);
        },
        onError: (e: any) => setError(e.response?.data?.error ?? "Failed to save"),
    });

    const deleteMutation = useMutation({
        mutationFn: () => reposApi.delete(owner, repo.name),
        onSuccess:  () => navigate(`/${owner}`),
        onError: (e: any) => setError(e.response?.data?.error ?? "Failed to delete"),
    });

    if (!isOwner) {
        return (
            <Alert type="error">
                You do not have permission to access settings for this repository.
            </Alert>
        );
    }

    return (
        <div className="max-w-2xl space-y-8">
            {error   && <Alert type="error"   onClose={() => setError("")}>{error}</Alert>}
            {success && <Alert type="success" >{success}</Alert>}

            {/* general */}
            <section className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6">
                <h2 className="text-base font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                    <Settings className="w-5 h-5" /> General
                </h2>
                <div className="space-y-4">
                    <Input
                        label="Repository name"
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                    />
                    <Input
                        label="Description"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        hint="Optional. Describe your repository"
                    />
                    <div className="space-y-2">
                        <p className="text-sm font-medium text-gray-700 dark:text-gray-300">Features</p>
                        {[
                            { key: "has_issues",      label: "Issues" },
                            { key: "has_wiki",        label: "Wikis" },
                            { key: "has_projects",    label: "Projects" },
                            { key: "has_discussions", label: "Discussions" },
                        ].map(({ key, label }) => (
                            <label key={key} className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400 cursor-pointer">
                                <input type="checkbox" defaultChecked={(repo as any)[key]}
                                       className="rounded border-gray-300" />
                                {label}
                            </label>
                        ))}
                    </div>
                    <Button
                        loading={updateMutation.isPending}
                        onClick={() => updateMutation.mutate()}
                    >
                        Save changes
                    </Button>
                </div>
            </section>

            {/* danger zone */}
            <section className="bg-white dark:bg-gray-900 rounded-xl border border-red-200 dark:border-red-900 p-6">
                <h2 className="text-base font-semibold text-red-600 dark:text-red-400 mb-4 flex items-center gap-2">
                    <AlertTriangle className="w-5 h-5" /> Danger Zone
                </h2>
                <div className="space-y-4">
                    <div>
                        <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Delete this repository
                        </p>
                        <p className="text-xs text-gray-500 mb-3">
                            Once deleted, there is no going back. This will permanently delete the
                            repository, wiki, issues, pull requests, and all associated data.
                        </p>
                        <Input
                            placeholder={`Type "${owner}/${repo.name}" to confirm`}
                            value={deleteConf}
                            onChange={(e) => setDeleteConf(e.target.value)}
                        />
                        <Button
                            variant="danger"
                            size="sm"
                            className="mt-2"
                            loading={deleteMutation.isPending}
                            disabled={deleteConf !== `${owner}/${repo.name}`}
                            icon={<Trash2 className="w-4 h-4" />}
                            onClick={() => deleteMutation.mutate()}
                        >
                            Delete this repository
                        </Button>
                    </div>
                </div>
            </section>
        </div>
    );
}