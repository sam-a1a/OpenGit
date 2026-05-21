import { useParams, Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { useSearchParams } from "react-router-dom";
import { FileText, Copy, Check, ChevronRight } from "lucide-react";
import { useState } from "react";
import { Header } from "../../components/layout/Header";
import { PageSpinner } from "../../components/ui/Spinner";
import { Badge } from "../../components/ui/Badge";
import { reposApi } from "../../api/repos";
import { cn } from "../../lib/utils";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark, oneLight } from "react-syntax-highlighter/dist/esm/styles/prism";
import { useUiStore } from "../../stores/ui";

const EXT_MAP: Record<string, string> = {
    rs: "rust", ts: "typescript", tsx: "typescript", js: "javascript",
    jsx: "javascript", py: "python", go: "go", java: "java", cpp: "cpp",
    c: "c", cs: "csharp", rb: "ruby", php: "php", sh: "bash",
    yaml: "yaml", yml: "yaml", toml: "toml", json: "json", md: "markdown",
    html: "html", css: "css", scss: "scss", sql: "sql", dockerfile: "dockerfile",
    kt: "kotlin", swift: "swift", dart: "dart", ex: "elixir",
};

export default function FileViewerPage() {
    const { owner, repo: repoName, "*": filePath } = useParams<{
        owner: string; repo: string; "*": string;
    }>();
    const [searchParams] = useSearchParams();
    const ref     = searchParams.get("ref") ?? "main";
    const theme   = useUiStore((s) => s.theme);
    const [copied, setCopied] = useState(false);

    const { data, isLoading } = useQuery({
        queryKey: ["blob", owner, repoName, ref, filePath],
        queryFn:  () => reposApi.blob(owner!, repoName!, ref, filePath ?? "")
            .then((r) => r.data),
        enabled:  !!owner && !!repoName && !!filePath,
    });

    const ext      = filePath?.split(".").pop()?.toLowerCase() ?? "";
    const language = EXT_MAP[ext] ?? "text";

    const copy = () => {
        if (data?.content) {
            navigator.clipboard.writeText(data.content);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    const pathParts = filePath?.split("/") ?? [];

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            <Header />
            <div className="max-w-7xl mx-auto px-4 py-6">

                {/* breadcrumb */}
                <div className="flex items-center gap-1 text-sm mb-4 flex-wrap">
                    <Link to={`/${owner}/${repoName}`}
                          className="text-blue-600 dark:text-blue-400 hover:underline font-semibold">
                        {repoName}
                    </Link>
                    {pathParts.map((part, i) => (
                        <span key={i} className="flex items-center gap-1">
              <ChevronRight className="w-4 h-4 text-gray-400" />
                            {i === pathParts.length - 1 ? (
                                <span className="text-gray-900 dark:text-white font-medium">{part}</span>
                            ) : (
                                <Link
                                    to={`/${owner}/${repoName}/tree/${ref}/${pathParts.slice(0, i + 1).join("/")}`}
                                    className="text-blue-600 dark:text-blue-400 hover:underline"
                                >
                                    {part}
                                </Link>
                            )}
            </span>
                    ))}
                </div>

                {isLoading ? (
                    <PageSpinner />
                ) : !data ? (
                    <div className="text-center py-16 text-gray-500">File not found</div>
                ) : (
                    <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                        {/* header */}
                        <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-800">
                            <div className="flex items-center gap-2 text-sm">
                                <FileText className="w-4 h-4 text-gray-400" />
                                <span className="font-medium text-gray-700 dark:text-gray-300">
                  {pathParts[pathParts.length - 1]}
                </span>
                                <Badge size="sm">{(data.size / 1024).toFixed(1)} KB</Badge>
                                {data.is_binary && <Badge size="sm" variant="warning">Binary</Badge>}
                            </div>
                            <div className="flex items-center gap-2">
                                <Badge size="sm">{language}</Badge>
                                <button
                                    onClick={copy}
                                    className="p-1.5 rounded-md border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-500"
                                >
                                    {copied ? <Check className="w-4 h-4 text-green-500" /> : <Copy className="w-4 h-4" />}
                                </button>
                            </div>
                        </div>

                        {/* content */}
                        {data.is_binary ? (
                            <div className="p-8 text-center text-gray-500">
                                Binary file — cannot display
                            </div>
                        ) : (
                            <div className="overflow-auto text-sm">
                                <SyntaxHighlighter
                                    language={language}
                                    style={theme === "dark" ? oneDark : oneLight}
                                    showLineNumbers
                                    lineNumberStyle={{ color: "#6b7280", minWidth: "3em", paddingRight: "1em" }}
                                    customStyle={{
                                        margin: 0,
                                        borderRadius: 0,
                                        background: "transparent",
                                        fontSize: "0.8125rem",
                                    }}
                                >
                                    {data.content}
                                </SyntaxHighlighter>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}