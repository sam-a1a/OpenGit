import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useMutation } from "@tanstack/react-query";
import { Lock, Eye, BookOpen } from "lucide-react";
import { PageLayout } from "../../components/layout/PageLayout";
import { Button } from "../../components/ui/Button";
import { Input } from "../../components/ui/Input";
import { Alert } from "../../components/ui/Alert";
import { useAuthStore } from "../../stores/auth";
import { reposApi } from "../../api/repos";
import { cn } from "../../lib/utils";

export default function NewRepoPage() {
    const user     = useAuthStore((s) => s.user);
    const navigate = useNavigate();

    const [name,        setName]        = useState("");
    const [description, setDescription] = useState("");
    const [visibility,  setVisibility]  = useState<"public" | "private">("public");
    const [autoInit,    setAutoInit]    = useState(true);
    const [error,       setError]       = useState("");

    const mutation = useMutation({
        mutationFn: () => reposApi.create({
            name, description, visibility, auto_init: autoInit,
        }),
        onSuccess: (r) => navigate(`/${user?.username}/${r.data.name}`),
        onError:   (e: any) => setError(e.response?.data?.error ?? "Failed to create repository"),
    });

    const isValid = name.trim().length >= 1 &&
        /^[a-zA-Z0-9-_.]+$/.test(name);

    return (
        <PageLayout narrow>
            <div className="max-w-2xl">
                <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">
                    Create a new repository
                </h1>

                {error && <Alert type="error" onClose={() => setError("")} className="mb-4">{error}</Alert>}

                <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6 space-y-5">

                    {/* owner/name */}
                    <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Repository name <span className="text-red-500">*</span>
                        </label>
                        <div className="flex items-center gap-2">
                            <div className="flex items-center gap-2 px-3 py-2 border border-gray-300 dark:border-gray-700 rounded-md bg-gray-50 dark:bg-gray-800 text-sm text-gray-600 dark:text-gray-400">
                                <BookOpen className="w-4 h-4" />
                                {user?.username}
                            </div>
                            <span className="text-gray-400">/</span>
                            <input
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                                placeholder="my-repository"
                                className="flex-1 px-3 py-2 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                        </div>
                        {name && !isValid && (
                            <p className="mt-1 text-xs text-red-500">
                                Only letters, numbers, hyphens, underscores and dots allowed
                            </p>
                        )}
                    </div>

                    <Input
                        label="Description"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        placeholder="Optional description"
                    />

                    {/* visibility */}
                    <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            Visibility
                        </label>
                        <div className="space-y-2">
                            {([
                                { key: "public",  label: "Public",  desc: "Anyone on the internet can see this repository.", Icon: Eye  },
                                { key: "private", label: "Private", desc: "Only you and people you share with can see this.", Icon: Lock },
                            ] as const).map(({ key, label, desc, Icon }) => (
                                <label
                                    key={key}
                                    className={cn(
                                        "flex items-start gap-3 p-4 rounded-lg border cursor-pointer transition-colors",
                                        visibility === key
                                            ? "border-blue-500 bg-blue-50/50 dark:bg-blue-900/10"
                                            : "border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800"
                                    )}
                                >
                                    <input
                                        type="radio"
                                        name="visibility"
                                        value={key}
                                        checked={visibility === key}
                                        onChange={() => setVisibility(key)}
                                        className="mt-0.5"
                                    />
                                    <div>
                                        <div className="flex items-center gap-1.5 text-sm font-medium text-gray-900 dark:text-white">
                                            <Icon className="w-4 h-4" />
                                            {label}
                                        </div>
                                        <p className="text-xs text-gray-500 mt-0.5">{desc}</p>
                                    </div>
                                </label>
                            ))}
                        </div>
                    </div>

                    {/* init */}
                    <div className="border-t border-gray-100 dark:border-gray-800 pt-4">
                        <label className="flex items-start gap-3 cursor-pointer">
                            <input
                                type="checkbox"
                                checked={autoInit}
                                onChange={(e) => setAutoInit(e.target.checked)}
                                className="mt-0.5 rounded border-gray-300"
                            />
                            <div>
                                <p className="text-sm font-medium text-gray-900 dark:text-white">
                                    Add a README file
                                </p>
                                <p className="text-xs text-gray-500 mt-0.5">
                                    This will let you immediately clone the repository.
                                </p>
                            </div>
                        </label>
                    </div>

                    <div className="border-t border-gray-100 dark:border-gray-800 pt-4">
                        <Button
                            className="w-full"
                            loading={mutation.isPending}
                            disabled={!isValid}
                            onClick={() => mutation.mutate()}
                        >
                            Create repository
                        </Button>
                    </div>
                </div>
            </div>
        </PageLayout>
    );
}