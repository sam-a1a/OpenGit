import { useState } from "react";
import { useParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
    CircleDot, CheckCircle2, MessageSquare, Lock,
    Pin, Edit2, Trash2, ThumbsUp, ThumbsDown,
    Smile, Rocket, Eye, Heart
} from "lucide-react";
import { Header } from "../../components/layout/Header";
import { Avatar } from "../../components/ui/Avatar";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { Alert } from "../../components/ui/Alert";
import { PageSpinner } from "../../components/ui/Spinner";
import { issuesApi } from "../../api/issues";
import { usersApi } from "../../api/users";
import { useAuthStore } from "../../stores/auth";
import { relativeTime, formatDateTime } from "../../lib/utils";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

const REACTIONS = [
    { key: "thumbs_up",   emoji: "👍" },
    { key: "thumbs_down", emoji: "👎" },
    { key: "laugh",       emoji: "😄" },
    { key: "hooray",      emoji: "🎉" },
    { key: "confused",    emoji: "😕" },
    { key: "heart",       emoji: "❤️" },
    { key: "rocket",      emoji: "🚀" },
    { key: "eyes",        emoji: "👀" },
];

export default function IssuePage() {
    const { owner, repo: repoName, number } = useParams<{
        owner: string; repo: string; number: string;
    }>();
    const currentUser  = useAuthStore((s) => s.user);
    const queryClient  = useQueryClient();
    const [comment, setComment] = useState("");
    const [editId,  setEditId]  = useState<string | null>(null);
    const [editBody, setEditBody] = useState("");

    const { data: issue, isLoading: issueLoading } = useQuery({
        queryKey: ["issue", owner, repoName, number],
        queryFn:  () => issuesApi.get(owner!, repoName!, Number(number))
            .then((r) => r.data),
    });

    const { data: commentsData, isLoading: commentsLoading } = useQuery({
        queryKey: ["issue-comments", owner, repoName, number],
        queryFn:  () => issuesApi.comments.list(owner!, repoName!, Number(number))
            .then((r) => r.data),
        enabled:  !!issue,
    });

    const addComment = useMutation({
        mutationFn: () => issuesApi.comments.create(owner!, repoName!, Number(number), comment),
        onSuccess:  () => {
            setComment("");
            queryClient.invalidateQueries({ queryKey: ["issue-comments", owner, repoName, number] });
        },
    });

    const updateComment = useMutation({
        mutationFn: () => issuesApi.comments.update(owner!, repoName!, editId!, editBody),
        onSuccess:  () => {
            setEditId(null);
            queryClient.invalidateQueries({ queryKey: ["issue-comments", owner, repoName, number] });
        },
    });

    const deleteComment = useMutation({
        mutationFn: (id: string) => issuesApi.comments.delete(owner!, repoName!, id),
        onSuccess:  () => {
            queryClient.invalidateQueries({ queryKey: ["issue-comments", owner, repoName, number] });
        },
    });

    const toggleState = useMutation({
        mutationFn: () => issue?.state === "Open"
            ? issuesApi.close(owner!, repoName!, Number(number))
            : issuesApi.reopen(owner!, repoName!, Number(number)),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["issue", owner, repoName, number] });
        },
    });

    const addReaction = useMutation({
        mutationFn: (reaction: string) =>
            issuesApi.comments.list(owner!, repoName!, Number(number)), // placeholder
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["issue-comments"] });
        },
    });

    if (issueLoading) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
                <Header />
                <PageSpinner />
            </div>
        );
    }

    if (!issue) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
                <Header />
                <div className="max-w-4xl mx-auto px-4 py-16 text-center text-gray-500">
                    Issue not found
                </div>
            </div>
        );
    }

    const comments = commentsData ?? [];
    const isOpen   = issue.state === "Open";

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            <Header />
            <div className="max-w-4xl mx-auto px-4 py-6">

                {/* title */}
                <div className="mb-6">
                    <div className="flex items-start gap-3 mb-2">
                        <h1 className="text-2xl font-bold text-gray-900 dark:text-white leading-tight">
                            {issue.title}
                            <span className="text-gray-400 font-normal ml-2">#{issue.number}</span>
                        </h1>
                    </div>
                    <div className="flex items-center gap-3 flex-wrap">
                        <Badge variant={isOpen ? "success" : "purple"} size="md" dot>
                            {isOpen
                                ? <><CircleDot className="w-3.5 h-3.5" /> Open</>
                                : <><CheckCircle2 className="w-3.5 h-3.5" /> Closed</>}
                        </Badge>
                        {issue.is_pinned && <Badge variant="info" size="sm"><Pin className="w-3 h-3" /> Pinned</Badge>}
                        {issue.locked   && <Badge variant="warning" size="sm"><Lock className="w-3 h-3" /> Locked</Badge>}
                        <span className="text-sm text-gray-500">
              Opened {relativeTime(issue.created_at)} · {issue.comment_count} comment{issue.comment_count !== 1 ? "s" : ""}
            </span>
                    </div>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
                    <div className="lg:col-span-3 space-y-4">

                        {/* original post */}
                        <CommentCard
                            authorId={issue.author_id}
                            body={issue.body ?? ""}
                            createdAt={issue.created_at}
                            currentUserId={currentUser?.id}
                            isOriginal
                        />

                        {/* comments */}
                        {commentsLoading ? (
                            <PageSpinner />
                        ) : (
                            comments.map((c: any) => (
                                <div key={c.id}>
                                    {editId === c.id ? (
                                        <div className="bg-white dark:bg-gray-900 rounded-xl border border-blue-300 p-4">
                      <textarea
                          value={editBody}
                          onChange={(e) => setEditBody(e.target.value)}
                          rows={4}
                          className="w-full px-3 py-2 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono"
                      />
                                            <div className="flex gap-2 mt-2 justify-end">
                                                <Button variant="ghost" size="sm" onClick={() => setEditId(null)}>Cancel</Button>
                                                <Button size="sm" loading={updateComment.isPending}
                                                        onClick={() => updateComment.mutate()}>
                                                    Update
                                                </Button>
                                            </div>
                                        </div>
                                    ) : (
                                        <CommentCard
                                            authorId={c.author_id}
                                            body={c.body}
                                            createdAt={c.created_at}
                                            isEdited={c.is_edited}
                                            currentUserId={currentUser?.id}
                                            onEdit={() => { setEditId(c.id); setEditBody(c.body); }}
                                            onDelete={() => deleteComment.mutate(c.id)}
                                        />
                                    )}
                                </div>
                            ))
                        )}

                        {/* add comment */}
                        {currentUser && (
                            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                                <div className="px-4 py-3 border-b border-gray-100 dark:border-gray-800 text-sm font-medium text-gray-700 dark:text-gray-300">
                                    Leave a comment
                                </div>
                                <div className="p-4">
                  <textarea
                      value={comment}
                      onChange={(e) => setComment(e.target.value)}
                      rows={5}
                      placeholder="Write your comment..."
                      className="w-full px-3 py-2 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono"
                  />
                                    <div className="flex items-center justify-between mt-3">
                                        <Button
                                            variant={isOpen ? "secondary" : "secondary"}
                                            size="sm"
                                            loading={toggleState.isPending}
                                            onClick={() => toggleState.mutate()}
                                        >
                                            {isOpen ? (
                                                <><CheckCircle2 className="w-4 h-4" /> Close issue</>
                                            ) : (
                                                <><CircleDot className="w-4 h-4" /> Reopen issue</>
                                            )}
                                        </Button>
                                        <Button
                                            size="sm"
                                            disabled={!comment.trim()}
                                            loading={addComment.isPending}
                                            onClick={() => addComment.mutate()}
                                        >
                                            <MessageSquare className="w-4 h-4" />
                                            Comment
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        )}
                    </div>

                    {/* sidebar */}
                    <aside className="space-y-4 text-sm">
                        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4">
                            <h3 className="font-semibold text-gray-700 dark:text-gray-300 mb-3">Labels</h3>
                            <p className="text-gray-400 text-xs">None yet</p>
                        </div>
                        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4">
                            <h3 className="font-semibold text-gray-700 dark:text-gray-300 mb-3">Assignees</h3>
                            <p className="text-gray-400 text-xs">No one assigned</p>
                        </div>
                        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-4">
                            <h3 className="font-semibold text-gray-700 dark:text-gray-300 mb-3">Milestone</h3>
                            <p className="text-gray-400 text-xs">No milestone</p>
                        </div>
                    </aside>
                </div>
            </div>
        </div>
    );
}

function CommentCard({
                         authorId, body, createdAt, isEdited,
                         currentUserId, isOriginal, onEdit, onDelete
                     }: {
    authorId:      string | null;
    body:          string;
    createdAt:     string;
    isEdited?:     boolean;
    currentUserId?: string;
    isOriginal?:   boolean;
    onEdit?:       () => void;
    onDelete?:     () => void;
}) {
    const { data: author } = useQuery({
        queryKey: ["user-id", authorId],
        queryFn:  () => authorId
            ? usersApi.get(authorId).then((r) => r.data).catch(() => null)
            : null,
        enabled: !!authorId,
    });

    const isOwn = currentUserId && authorId === currentUserId;

    return (
        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
            <div className="flex items-center justify-between px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
                <div className="flex items-center gap-2 text-sm">
                    <Avatar username={author?.username ?? "?"} src={author?.avatar_url} size="xs" />
                    <span className="font-medium text-gray-900 dark:text-white">
            {author?.username ?? "unknown"}
          </span>
                    <span className="text-gray-500">
            commented {relativeTime(createdAt)}
          </span>
                    {isEdited && <span className="text-gray-400 text-xs">· edited</span>}
                    {isOriginal && (
                        <Badge size="sm" variant="info">Author</Badge>
                    )}
                </div>
                {isOwn && !isOriginal && (
                    <div className="flex items-center gap-1">
                        {onEdit && (
                            <button onClick={onEdit}
                                    className="p-1 rounded text-gray-400 hover:text-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700">
                                <Edit2 className="w-3.5 h-3.5" />
                            </button>
                        )}
                        {onDelete && (
                            <button onClick={onDelete}
                                    className="p-1 rounded text-gray-400 hover:text-red-500 hover:bg-gray-100 dark:hover:bg-gray-700">
                                <Trash2 className="w-3.5 h-3.5" />
                            </button>
                        )}
                    </div>
                )}
            </div>
            <div className="p-4">
                {body ? (
                    <div className="prose dark:prose-invert prose-sm max-w-none">
                        <ReactMarkdown remarkPlugins={[remarkGfm]}>{body}</ReactMarkdown>
                    </div>
                ) : (
                    <p className="text-gray-400 italic text-sm">No description provided.</p>
                )}
            </div>
            <div className="px-4 py-2 border-t border-gray-100 dark:border-gray-800 flex items-center gap-1">
                {REACTIONS.map((r) => (
                    <button key={r.key}
                            className="text-base hover:scale-125 transition-transform p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-800"
                            title={r.key.replace("_", " ")}>
                        {r.emoji}
                    </button>
                ))}
            </div>
        </div>
    );
}