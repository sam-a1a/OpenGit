export interface Repo {
    id:                     string;
    owner_id:               string;
    org_id:                 string | null;
    name:                   string;
    description:            string | null;
    visibility:             "Public" | "Private" | "Internal";
    default_branch:         string;
    is_fork:                boolean;
    forked_from_id:         string | null;
    is_template:            boolean;
    is_archived:            boolean;
    is_empty:               boolean;
    has_issues:             boolean;
    has_wiki:               boolean;
    has_projects:           boolean;
    has_discussions:        boolean;
    allow_merge_commit:     boolean;
    allow_squash_merge:     boolean;
    allow_rebase_merge:     boolean;
    delete_branch_on_merge: boolean;
    star_count:             number;
    fork_count:             number;
    watcher_count:          number;
    open_issue_count:       number;
    git_path:               string;
    created_at:             string;
    updated_at:             string;
    pushed_at:              string | null;
}

export interface TreeEntry {
    mode:       string;
    entry_type: "blob" | "tree";
    sha:        string;
    name:       string;
    path:       string;
}

export interface CommitInfo {
    sha:             string;
    message:         string;
    author_name:     string;
    author_email:    string;
    authored_at:     string;
    committer_name:  string;
    committer_email: string;
    committed_at:    string;
    parents:         string[];
}

export interface BlobContent {
    path:      string;
    sha:       string;
    content:   string;
    size:      number;
    is_binary: boolean;
    encoding:  string;
}

export interface RefInfo {
    name:     string;
    sha:      string;
    ref_type: "branch" | "tag";
}