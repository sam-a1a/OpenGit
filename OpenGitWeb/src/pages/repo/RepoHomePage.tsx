import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
    GitBranch, Clock, FileText, Folder,
    ChevronRight, Download, Copy, Check, Tag
} from "lucide-react";
import { Button } from "../../components/ui/Button";
import { PageSpinner } from "../../components/ui/Spinner";
import { Badge } from "../../components/ui/Badge";
import { reposApi } from "../../api/repos";
import { relativeTime, cn } from "../../lib/utils";
import type { Repo, TreeEntry, CommitInfo, RefInfo } from "../../types/repo";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

interface Props {
    repo:  Repo;
    owner: string;
}

export default function RepoHomePage({ repo, owner }: Props) {
    const [ref,  setRef]      = useState(repo.default_branch);
    const [path, setPath]     = useState("");
    const [copied, setCopied] = useState(false);
    const [showClone, setShowClone] = useState(false);

    const { data: refs } = useQuery({
        queryKey: ["refs", owner, repo.name],
        queryFn:  () => reposApi.refs(owner, repo.name).then((r) => r.data),
    });

    const { data: treeData, isLoading: treeLoading } = useQuery({
        queryKey: ["tree", owner, repo.name, ref, path],
        queryFn:  () => reposApi.tree(owner, repo.name, ref, path).then((r) => r.data),
        enabled:  !repo.is_empty,
    });

    const { data: commitsData } = useQuery({
        queryKey: ["commits", owner, repo.name, ref],
        queryFn:  () => reposApi.commits(owner, repo.name, ref).then((r) => r.data),
        enabled:  !repo.is_empty,
    });

    const { data: statsData } = useQuery({
        queryKey: ["stats", owner, repo.name],
        queryFn:  () => reposApi.stats(owner, repo.name).then((r) => r.data),
        enabled:  !repo.is_empty,
    });

    const entries: TreeEntry[]   = treeData?.entries ?? [];
    const lastCommit: CommitInfo = commitsData?.commits?.[0];
    const branches: RefInfo[]    = refs?.branches ?? [];
    const tags: RefInfo[]        = refs?.tags ?? [];

    const cloneUrl = `${window.location.origin}/${owner}/${repo.name}.git`;

    const copyClone = () => {
        navigator.clipboard.writeText(cloneUrl);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    const navigatePath = (entry: TreeEntry) => {
        if (entry.entry_type === "tree") {
            setPath(entry.path);
        }
    };

    const pathParts = path ? path.split("/") : [];

    if (repo.is_empty) {
        return (
            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-10 text-center">
                <GitBranch className="w-12 h-12 mx-auto text-gray-300 mb-4" />
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                    Get started
                </h3>
                <p className="text-gray-500 mb-6 text-sm">
                    This repository is empty. Push your first commit.
                </p>
                <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 text-left max-w-md mx-auto">
                    <p className="text-xs font-mono text-gray-600 dark:text-gray-300 space-y-1">
                        <span className="block text-gray-400"># clone via HTTPS</span>
                        <span className="block">git clone {cloneUrl}</span>
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-4">
            <div className="lg:col-span-3 space-y-4">

                {/* branch selector + clone */}
                <div className="flex items-center gap-2 flex-wrap">
                    <select
                        value={ref}
                        onChange={(e) => { setRef(e.target.value); setPath(""); }}
                        className="text-sm border border-gray-300 dark:border-gray-700 rounded-md px-3 py-1.5 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    >
                        {branches.map((b) => (
                            <option key={b.name} value={b.name}>{b.name}</option>
                        ))}
                        {tags.map((t) => (
                            <option key={t.name} value={t.name}>{t.name}</option>
                        ))}
                    </select>

                    <span className="text-sm text-gray-500 flex items-center gap-1">
            <GitBranch className="w-3.5 h-3.5" />
                        {branches.length} branch{branches.length !== 1 ? "es" : ""}
          </span>
                    <span className="text-sm text-gray-500 flex items-center gap-1">
            <Tag className="w-3.5 h-3.5" />
                        {tags.length} tag{tags.length !== 1 ? "s" : ""}
          </span>

                    <div className="ml-auto relative">
                        <Button
                            variant="primary"
                            size="sm"
                            icon={<Download className="w-3.5 h-3.5" />}
                            onClick={() => setShowClone(!showClone)}
                        >
                            Code
                        </Button>
                        {showClone && (
                            <div className="absolute right-0 top-full mt-1 w-80 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-xl shadow-xl p-4 z-10">
                                <p className="text-xs font-semibold text-gray-500 mb-2 uppercase tracking-wide">
                                    Clone with HTTPS
                                </p>
                                <div className="flex gap-2">
                                    <input
                                        readOnly
                                        value={cloneUrl}
                                        className="flex-1 text-xs bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md px-2 py-1.5 font-mono text-gray-700 dark:text-gray-300"
                                    />
                                    <button onClick={copyClone}
                                            className="p-1.5 rounded-md border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-500">
                                        {copied ? <Check className="w-4 h-4 text-green-500" /> : <Copy className="w-4 h-4" />}
                                    </button>
                                </div>
                            </div>
                        )}
                    </div>
                </div>

                {/* breadcrumb */}
                {path && (
                    <div className="flex items-center gap-1 text-sm">
                        <button onClick={() => setPath("")}
                                className="text-blue-600 dark:text-blue-400 hover:underline font-medium">
                            {repo.name}
                        </button>
                        {pathParts.map((part, i) => (
                            <span key={i} className="flex items-center gap-1">
                <ChevronRight className="w-4 h-4 text-gray-400" />
                <button
                    onClick={() => setPath(pathParts.slice(0, i + 1).join("/"))}
                    className="text-blue-600 dark:text-blue-400 hover:underline"
                >
                  {part}
                </button>
              </span>
                        ))}
                    </div>
                )}

                {/* file tree */}
                <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                    {/* last commit bar */}
                    {lastCommit && (
                        <div className="flex items-center gap-2 px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 text-sm">
                            <Clock className="w-3.5 h-3.5 text-gray-400" />
                            <span className="font-medium text-gray-700 dark:text-gray-300 truncate">
                {lastCommit.message}
              </span>
                            <span className="text-gray-400 ml-auto flex-shrink-0">
                {relativeTime(lastCommit.authored_at)}
              </span>
                        </div>
                    )}

                    {treeLoading ? (
                        <div className="flex items-center justify-center py-10">
                            <PageSpinner />
                        </div>
                    ) : (
                        <div className="divide-y divide-gray-100 dark:divide-gray-800">
                            {entries.length === 0 ? (
                                <div className="py-10 text-center text-gray-500 text-sm">
                                    Empty directory
                                </div>
                            ) : (
                                entries
                                    .sort((a, b) => {
                                        if (a.entry_type === "tree" && b.entry_type !== "tree") return -1;
                                        if (a.entry_type !== "tree" && b.entry_type === "tree") return 1;
                                        return a.name.localeCompare(b.name);
                                    })
                                    .map((entry) => (
                                        <FileRow
                                            key={entry.sha}
                                            entry={entry}
                                            onClick={() => navigatePath(entry)}
                                            owner={owner}
                                            repoName={repo.name}
                                            ref_={ref}
                                        />
                                    ))
                            )}
                        </div>
                    )}
                </div>

                {/* readme */}
                <ReadmePanel owner={owner} repoName={repo.name} ref_={ref} />
            </div>

            {/* right sidebar */}
            <aside className="space-y-4">
                {repo.description && (
                    <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4">
                        <h3 className="text-sm font-semibold text-gray-900 dark:text-white mb-2">About</h3>
                        <p className="text-sm text-gray-600 dark:text-gray-400">{repo.description}</p>
                    </div>
                )}

                <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4 space-y-3 text-sm">
                    <div className="flex items-center justify-between text-gray-600 dark:text-gray-400">
            <span className="flex items-center gap-1.5">
              <GitBranch className="w-4 h-4" /> Branches
            </span>
                        <Badge>{branches.length}</Badge>
                    </div>
                    <div className="flex items-center justify-between text-gray-600 dark:text-gray-400">
            <span className="flex items-center gap-1.5">
              <Tag className="w-4 h-4" /> Tags
            </span>
                        <Badge>{tags.length}</Badge>
                    </div>
                    {statsData && (
                        <div className="flex items-center justify-between text-gray-600 dark:text-gray-400">
              <span className="flex items-center gap-1.5">
                <Clock className="w-4 h-4" /> Commits
              </span>
                            <Badge>{statsData.commit_count?.toLocaleString()}</Badge>
                        </div>
                    )}
                </div>
            </aside>
        </div>
    );
}

function FileRow({ entry, onClick, owner, repoName, ref_ }: {
    entry:    TreeEntry;
    onClick:  () => void;
    owner:    string;
    repoName: string;
    ref_:     string;
}) {
    const isDir = entry.entry_type === "tree";

    return (
        <div
            onClick={onClick}
            className="flex items-center gap-3 px-4 py-2 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer"
        >
            {isDir ? (
                <Folder className="w-4 h-4 text-blue-400 flex-shrink-0" />
            ) : (
                <FileText className="w-4 h-4 text-gray-400 flex-shrink-0" />
            )}
            <span className={cn(
                "text-sm",
                isDir
                    ? "text-blue-600 dark:text-blue-400 font-medium"
                    : "text-gray-700 dark:text-gray-300"
            )}>
        {entry.name}
      </span>
        </div>
    );
}

function ReadmePanel({ owner, repoName, ref_ }: {
    owner:    string;
    repoName: string;
    ref_:     string;
}) {
    const { data } = useQuery({
        queryKey: ["readme", owner, repoName, ref_],
        queryFn:  () => reposApi.blob(owner, repoName, ref_, "README.md")
            .then((r) => r.data)
            .catch(() => null),
    });

    if (!data || data.is_binary) return null;

    return (
        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
            <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-800 flex items-center gap-2">
                <FileText className="w-4 h-4 text-gray-400" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">README.md</span>
            </div>
            <div className="p-6 prose dark:prose-invert prose-sm max-w-none">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>
                    {data.content}
                </ReactMarkdown>
            </div>
        </div>
    );
}