use crate::{
    api::routes::{
        auth::{login_handler, logout_handler, me_handler, refresh_handler, register_handler},
        git_http::{git_receive_pack, git_upload_pack, info_refs},
        health::health_check,
        issues::{
            add_assignees, add_comment_reaction, add_issue_labels, add_issue_reaction,
            create_comment, create_issue, create_label, create_milestone, delete_comment,
            delete_label, delete_milestone, get_issue, list_comments, list_issues,
            list_labels, list_milestones, lock_issue, pin_issue, remove_assignees,
            remove_issue_label, remove_issue_reaction, subscribe_issue, unlock_issue,
            unpin_issue, unsubscribe_issue, update_comment, update_issue, update_label,
            update_milestone,
        },
        repos::{
            add_collaborator, archive_repo, create_branch_protection, create_repo,
            delete_repo, fork_repo, get_repo, get_topics, list_branch_protections,
            list_collaborators, list_forks, list_my_repos, list_stargazers,
            remove_collaborator, search_repos, star_repo, unarchive_repo,
            unstar_repo, unwatch_repo, update_repo, update_topics, watch_repo,
        },
        users::{
            add_ssh_key, block_user, delete_ssh_key, follow_user, get_followers,
            get_following, get_me, get_user, get_user_repos, list_ssh_keys,
            search_users, unblock_user, unfollow_user, update_me, update_status,
        },
        pull_requests::{
            add_pr_assignees, add_pr_labels, close_pr, create_pr, create_review,
            create_review_comment, create_status, delete_review_comment, dismiss_review,
            get_pr, is_merged, list_review_comments, list_reviews, list_prs, list_statuses,
            merge_pr, remove_reviewers, reopen_pr, request_reviewers, resolve_review_comment,
            update_pr, update_review_comment,
        }
    },
    state::AppState,
};
use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

pub fn build(state: AppState) -> Router {
    Router::new()
        // health
        .route("/health", get(health_check))

        // auth
        .route("/api/v1/auth/register", post(register_handler))
        .route("/api/v1/auth/login",    post(login_handler))
        .route("/api/v1/auth/refresh",  post(refresh_handler))
        .route("/api/v1/auth/logout",   post(logout_handler))
        .route("/api/v1/auth/me",       get(me_handler))

        // current user
        .route("/api/v1/user",               get(get_me))
        .route("/api/v1/user",               patch(update_me))
        .route("/api/v1/user/status",        patch(update_status))
        .route("/api/v1/user/keys",          get(list_ssh_keys))
        .route("/api/v1/user/keys",          post(add_ssh_key))
        .route("/api/v1/user/keys/{key_id}", delete(delete_ssh_key))
        .route("/api/v1/user/repos",         get(list_my_repos))

        // users
        .route("/api/v1/users/search",                      get(search_users))
        .route("/api/v1/users/{username}",                  get(get_user))
        .route("/api/v1/users/{username}/repos",            get(get_user_repos))
        .route("/api/v1/users/{username}/followers",        get(get_followers))
        .route("/api/v1/users/{username}/following",        get(get_following))
        .route("/api/v1/users/{username}/follow",           put(follow_user))
        .route("/api/v1/users/{username}/follow",           delete(unfollow_user))
        .route("/api/v1/users/{username}/block",            put(block_user))
        .route("/api/v1/users/{username}/block",            delete(unblock_user))

        // repos
        .route("/api/v1/repos/search",                                      get(search_repos))
        .route("/api/v1/repos",                                             post(create_repo))
        .route("/api/v1/repos/{owner}/{repo}",                              get(get_repo))
        .route("/api/v1/repos/{owner}/{repo}",                              patch(update_repo))
        .route("/api/v1/repos/{owner}/{repo}",                              delete(delete_repo))
        .route("/api/v1/repos/{owner}/{repo}/forks",                        get(list_forks))
        .route("/api/v1/repos/{owner}/{repo}/forks",                        post(fork_repo))
        .route("/api/v1/repos/{owner}/{repo}/star",                         put(star_repo))
        .route("/api/v1/repos/{owner}/{repo}/star",                         delete(unstar_repo))
        .route("/api/v1/repos/{owner}/{repo}/watch",                        put(watch_repo))
        .route("/api/v1/repos/{owner}/{repo}/watch",                        delete(unwatch_repo))
        .route("/api/v1/repos/{owner}/{repo}/stargazers",                   get(list_stargazers))
        .route("/api/v1/repos/{owner}/{repo}/topics",                       get(get_topics))
        .route("/api/v1/repos/{owner}/{repo}/topics",                       put(update_topics))
        .route("/api/v1/repos/{owner}/{repo}/archive",                      put(archive_repo))
        .route("/api/v1/repos/{owner}/{repo}/archive",                      delete(unarchive_repo))
        .route("/api/v1/repos/{owner}/{repo}/collaborators",                get(list_collaborators))
        .route("/api/v1/repos/{owner}/{repo}/collaborators/{username}",     put(add_collaborator))
        .route("/api/v1/repos/{owner}/{repo}/collaborators/{username}",     delete(remove_collaborator))
        .route("/api/v1/repos/{owner}/{repo}/branch-protections",           get(list_branch_protections))
        .route("/api/v1/repos/{owner}/{repo}/branch-protections",           post(create_branch_protection))

        // issues
        .route("/api/v1/repos/{owner}/{repo}/issues",                               get(list_issues))
        .route("/api/v1/repos/{owner}/{repo}/issues",                               post(create_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}",                      get(get_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}",                      patch(update_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/lock",                 put(lock_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/lock",                 delete(unlock_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/pin",                  put(pin_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/pin",                  delete(unpin_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/assignees",            post(add_assignees))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/assignees",            delete(remove_assignees))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/labels",               post(add_issue_labels))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/labels/{label_id}",    delete(remove_issue_label))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/comments",             get(list_comments))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/comments",             post(create_comment))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/reactions",            post(add_issue_reaction))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/reactions/{id}",       delete(remove_issue_reaction))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/subscription",         put(subscribe_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/subscription",         delete(unsubscribe_issue))
        .route("/api/v1/repos/{owner}/{repo}/comments/{comment_id}",                patch(update_comment))
        .route("/api/v1/repos/{owner}/{repo}/comments/{comment_id}",                delete(delete_comment))
        .route("/api/v1/repos/{owner}/{repo}/comments/{comment_id}/reactions",      post(add_comment_reaction))

        // pull requests
        .route("/api/v1/repos/{owner}/{repo}/pulls",                                    get(list_prs))
        .route("/api/v1/repos/{owner}/{repo}/pulls",                                    post(create_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}",                           get(get_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}",                           patch(update_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/close",                     put(close_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/reopen",                    put(reopen_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/merge",                     put(merge_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/merged",                    get(is_merged))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/reviews",                   get(list_reviews))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/reviews",                   post(create_review))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/reviews/{review_id}/dismissals", put(dismiss_review))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/comments",                  get(list_review_comments))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/comments",                  post(create_review_comment))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/requested_reviewers",       post(request_reviewers))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/requested_reviewers",       delete(remove_reviewers))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/labels",                    post(add_pr_labels))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/assignees",                 post(add_pr_assignees))
        .route("/api/v1/repos/{owner}/{repo}/pulls/comments/{comment_id}",              patch(update_review_comment))
        .route("/api/v1/repos/{owner}/{repo}/pulls/comments/{comment_id}",              delete(delete_review_comment))
        .route("/api/v1/repos/{owner}/{repo}/pulls/comments/{comment_id}/resolve",      put(resolve_review_comment))
        .route("/api/v1/repos/{owner}/{repo}/statuses/{sha}",                           get(list_statuses))
        .route("/api/v1/repos/{owner}/{repo}/statuses/{sha}",                           post(create_status))

        // labels
        .route("/api/v1/repos/{owner}/{repo}/labels",                get(list_labels))
        .route("/api/v1/repos/{owner}/{repo}/labels",                post(create_label))
        .route("/api/v1/repos/{owner}/{repo}/labels/{label_id}",     patch(update_label))
        .route("/api/v1/repos/{owner}/{repo}/labels/{label_id}",     delete(delete_label))

        // milestones
        .route("/api/v1/repos/{owner}/{repo}/milestones",                    get(list_milestones))
        .route("/api/v1/repos/{owner}/{repo}/milestones",                    post(create_milestone))
        .route("/api/v1/repos/{owner}/{repo}/milestones/{milestone_id}",     patch(update_milestone))
        .route("/api/v1/repos/{owner}/{repo}/milestones/{milestone_id}",     delete(delete_milestone))

        // git smart http
        .route("/{owner}/{repo}/info/refs",        get(info_refs))
        .route("/{owner}/{repo}/git-upload-pack",  post(git_upload_pack))
        .route("/{owner}/{repo}/git-receive-pack", post(git_receive_pack))

        .with_state(state)
}