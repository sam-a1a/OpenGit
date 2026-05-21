export interface PullRequest {
    id:              string;
    repo_id:         string;
    author_id:       string | null;
    number:          number;
    title:           string;
    body:            string | null;
    state:           "Open" | "Closed" | "Merged";
    is_draft:        boolean;
    locked:          boolean;
    head_branch:     string;
    base_branch:     string;
    head_sha:        string | null;
    merge_commit_sha: string | null;
    merged_at:       string | null;
    closed_at:       string | null;
    comment_count:   number;
    additions:       number;
    deletions:       number;
    changed_files:   number;
    created_at:      string;
    updated_at:      string;
}