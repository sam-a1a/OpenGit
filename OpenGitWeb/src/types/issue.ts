export interface Issue {
    id:            string;
    repo_id:       string;
    author_id:     string | null;
    milestone_id:  string | null;
    number:        number;
    title:         string;
    body:          string | null;
    state:         "Open" | "Closed";
    state_reason:  string | null;
    locked:        boolean;
    is_pinned:     boolean;
    comment_count: number;
    closed_at:     string | null;
    created_at:    string;
    updated_at:    string;
}

export interface Label {
    id:          string;
    repo_id:     string;
    name:        string;
    color:       string;
    description: string | null;
}

export interface Milestone {
    id:          string;
    repo_id:     string;
    title:       string;
    description: string | null;
    state:       "Open" | "Closed";
    due_on:      string | null;
}

export interface IssueComment {
    id:         string;
    issue_id:   string;
    author_id:  string | null;
    body:       string;
    is_edited:  boolean;
    created_at: string;
    updated_at: string;
}