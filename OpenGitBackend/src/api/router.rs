use crate::{
    api::routes::{
        auth::{login_handler, logout_handler, me_handler, refresh_handler, register_handler},
        git_browser::{
            get_blame, get_blob, get_commit, get_diff, get_raw, get_stats,
            get_tree, list_commits, list_refs,
        },
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
        pull_requests::{
            add_pr_assignees, add_pr_labels, close_pr, create_pr, create_review,
            create_review_comment, create_status, delete_review_comment, dismiss_review,
            get_pr, is_merged, list_prs, list_review_comments, list_reviews,
            list_statuses, merge_pr, remove_reviewers, reopen_pr, request_reviewers,
            resolve_review_comment, update_pr, update_review_comment,
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
        webhooks::{
            create_webhook, delete_webhook, get_delivery, get_webhook,
            list_deliveries, list_webhooks, ping_webhook, redeliver, update_webhook,
        },
        releases::{
            create_release, delete_asset, delete_release, download_asset,
            get_latest_release, get_release, get_release_by_tag, list_assets,
            list_releases, update_release, upload_asset,
        },
        notifications::{
            delete_all_read, delete_notification, delete_repo_subscription,
            get_notification, get_repo_subscription, list_notifications,
            list_repo_notifications, mark_all_read, mark_read, mark_repo_read,
            save_notification, set_repo_subscription, unread_count, unsave_notification,
        },
        organizations::{
            accept_invitation, add_team_member, add_team_repo, cancel_invitation,
            create_invitation, create_org, create_team, decline_invitation, delete_org,
            delete_team, get_member, get_org, get_team, list_invitations, list_members,
            list_my_orgs, list_org_repos, list_team_members, list_team_repos, list_teams,
            list_user_orgs, remove_member, remove_team_member, remove_team_repo,
            update_member_role, update_org, update_team,
        },
        search::{
            reindex_all, search_comments_meili, search_issues_meili,
            search_prs_meili, search_repos_meili, search_users_meili, unified_search,
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
        .route("/api/v1/users/search",               get(search_users))
        .route("/api/v1/users/{username}",            get(get_user))
        .route("/api/v1/users/{username}/repos",      get(get_user_repos))
        .route("/api/v1/users/{username}/followers",  get(get_followers))
        .route("/api/v1/users/{username}/following",  get(get_following))
        .route("/api/v1/users/{username}/follow",     put(follow_user))
        .route("/api/v1/users/{username}/follow",     delete(unfollow_user))
        .route("/api/v1/users/{username}/block",      put(block_user))
        .route("/api/v1/users/{username}/block",      delete(unblock_user))

        // repos
        .route("/api/v1/repos/search",                                  get(search_repos))
        .route("/api/v1/repos",                                         post(create_repo))
        .route("/api/v1/repos/{owner}/{repo}",                          get(get_repo))
        .route("/api/v1/repos/{owner}/{repo}",                          patch(update_repo))
        .route("/api/v1/repos/{owner}/{repo}",                          delete(delete_repo))
        .route("/api/v1/repos/{owner}/{repo}/forks",                    get(list_forks))
        .route("/api/v1/repos/{owner}/{repo}/forks",                    post(fork_repo))
        .route("/api/v1/repos/{owner}/{repo}/star",                     put(star_repo))
        .route("/api/v1/repos/{owner}/{repo}/star",                     delete(unstar_repo))
        .route("/api/v1/repos/{owner}/{repo}/watch",                    put(watch_repo))
        .route("/api/v1/repos/{owner}/{repo}/watch",                    delete(unwatch_repo))
        .route("/api/v1/repos/{owner}/{repo}/stargazers",               get(list_stargazers))
        .route("/api/v1/repos/{owner}/{repo}/topics",                   get(get_topics))
        .route("/api/v1/repos/{owner}/{repo}/topics",                   put(update_topics))
        .route("/api/v1/repos/{owner}/{repo}/archive",                  put(archive_repo))
        .route("/api/v1/repos/{owner}/{repo}/archive",                  delete(unarchive_repo))
        .route("/api/v1/repos/{owner}/{repo}/collaborators",            get(list_collaborators))
        .route("/api/v1/repos/{owner}/{repo}/collaborators/{username}", put(add_collaborator))
        .route("/api/v1/repos/{owner}/{repo}/collaborators/{username}", delete(remove_collaborator))
        .route("/api/v1/repos/{owner}/{repo}/branch-protections",       get(list_branch_protections))
        .route("/api/v1/repos/{owner}/{repo}/branch-protections",       post(create_branch_protection))

        // webhooks
        .route("/api/v1/repos/{owner}/{repo}/hooks",                                         get(list_webhooks))
        .route("/api/v1/repos/{owner}/{repo}/hooks",                                         post(create_webhook))
        .route("/api/v1/repos/{owner}/{repo}/hooks/{hook_id}",                               get(get_webhook))
        .route("/api/v1/repos/{owner}/{repo}/hooks/{hook_id}",                               patch(update_webhook))
        .route("/api/v1/repos/{owner}/{repo}/hooks/{hook_id}",                               delete(delete_webhook))
        .route("/api/v1/repos/{owner}/{repo}/hooks/{hook_id}/pings",                         post(ping_webhook))
        .route("/api/v1/repos/{owner}/{repo}/hooks/{hook_id}/deliveries",                    get(list_deliveries))
        .route("/api/v1/repos/{owner}/{repo}/hooks/{hook_id}/deliveries/{delivery_id}",      get(get_delivery))
        .route("/api/v1/repos/{owner}/{repo}/hooks/{hook_id}/deliveries/{delivery_id}/attempts", post(redeliver))

        // notifications
        .route("/api/v1/notifications",                                     get(list_notifications))
        .route("/api/v1/notifications",                                     put(mark_all_read))
        .route("/api/v1/notifications",                                     delete(delete_all_read))
        .route("/api/v1/notifications/count",                               get(unread_count))
        .route("/api/v1/notifications/{id}",                                get(get_notification))
        .route("/api/v1/notifications/{id}/read",                           patch(mark_read))
        .route("/api/v1/notifications/{id}/save",                           put(save_notification))
        .route("/api/v1/notifications/{id}/save",                           delete(unsave_notification))
        .route("/api/v1/notifications/{id}",                                delete(delete_notification))
        .route("/api/v1/repos/{owner}/{repo}/notifications",                get(list_repo_notifications))
        .route("/api/v1/repos/{owner}/{repo}/notifications",                put(mark_repo_read))
        .route("/api/v1/repos/{owner}/{repo}/subscription",                 get(get_repo_subscription))
        .route("/api/v1/repos/{owner}/{repo}/subscription",                 put(set_repo_subscription))
        .route("/api/v1/repos/{owner}/{repo}/subscription",                 delete(delete_repo_subscription))

        // organizations
        .route("/api/v1/user/orgs",                                                         get(list_my_orgs))
        .route("/api/v1/orgs",                                                              post(create_org))
        .route("/api/v1/orgs/{org}",                                                        get(get_org))
        .route("/api/v1/orgs/{org}",                                                        patch(update_org))
        .route("/api/v1/orgs/{org}",                                                        delete(delete_org))
        .route("/api/v1/orgs/{org}/repos",                                                  get(list_org_repos))
        .route("/api/v1/users/{username}/orgs",                                             get(list_user_orgs))

        // org members
        .route("/api/v1/orgs/{org}/members",                                                get(list_members))
        .route("/api/v1/orgs/{org}/members/{username}",                                     get(get_member))
        .route("/api/v1/orgs/{org}/members/{username}",                                     delete(remove_member))
        .route("/api/v1/orgs/{org}/members/{username}/role",                                patch(update_member_role))

        // invitations
        .route("/api/v1/orgs/{org}/invitations",                                            get(list_invitations))
        .route("/api/v1/orgs/{org}/invitations",                                            post(create_invitation))
        .route("/api/v1/orgs/{org}/invitations/{invitation_id}",                            delete(cancel_invitation))
        .route("/api/v1/invitations/{token}/accept",                                        post(accept_invitation))
        .route("/api/v1/invitations/{token}/decline",                                       post(decline_invitation))

        // teams
        .route("/api/v1/orgs/{org}/teams",                                                  get(list_teams))
        .route("/api/v1/orgs/{org}/teams",                                                  post(create_team))
        .route("/api/v1/orgs/{org}/teams/{team_slug}",                                      get(get_team))
        .route("/api/v1/orgs/{org}/teams/{team_slug}",                                      patch(update_team))
        .route("/api/v1/orgs/{org}/teams/{team_slug}",                                      delete(delete_team))
        .route("/api/v1/orgs/{org}/teams/{team_slug}/members",                              get(list_team_members))
        .route("/api/v1/orgs/{org}/teams/{team_slug}/members/{username}",                   put(add_team_member))
        .route("/api/v1/orgs/{org}/teams/{team_slug}/members/{username}",                   delete(remove_team_member))
        .route("/api/v1/orgs/{org}/teams/{team_slug}/repos",                                get(list_team_repos))
        .route("/api/v1/orgs/{org}/teams/{team_slug}/repos/{owner}/{repo}",                 put(add_team_repo))
        .route("/api/v1/orgs/{org}/teams/{team_slug}/repos/{owner}/{repo}",                 delete(remove_team_repo))

        // search
        .route("/api/v1/search",              get(unified_search))
        .route("/api/v1/search/repositories", get(search_repos_meili))
        .route("/api/v1/search/issues",       get(search_issues_meili))
        .route("/api/v1/search/pulls",        get(search_prs_meili))
        .route("/api/v1/search/users",        get(search_users_meili))
        .route("/api/v1/search/comments",     get(search_comments_meili))
        .route("/api/v1/admin/reindex",       post(reindex_all))

        // git browser
        .route("/api/v1/repos/{owner}/{repo}/git/refs",                 get(list_refs))
        .route("/api/v1/repos/{owner}/{repo}/git/commits/{ref}",        get(list_commits))
        .route("/api/v1/repos/{owner}/{repo}/git/commits/{sha}/single", get(get_commit))
        .route("/api/v1/repos/{owner}/{repo}/git/tree/{ref}",           get(get_tree))
        .route("/api/v1/repos/{owner}/{repo}/git/blob/{ref}",           get(get_blob))
        .route("/api/v1/repos/{owner}/{repo}/git/blame/{ref}",          get(get_blame))
        .route("/api/v1/repos/{owner}/{repo}/git/diff",                 get(get_diff))
        .route("/api/v1/repos/{owner}/{repo}/git/raw/{ref}",            get(get_raw))
        .route("/api/v1/repos/{owner}/{repo}/git/stats",                get(get_stats))

        // issues
        .route("/api/v1/repos/{owner}/{repo}/issues",                            get(list_issues))
        .route("/api/v1/repos/{owner}/{repo}/issues",                            post(create_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}",                   get(get_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}",                   patch(update_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/lock",              put(lock_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/lock",              delete(unlock_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/pin",               put(pin_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/pin",               delete(unpin_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/assignees",         post(add_assignees))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/assignees",         delete(remove_assignees))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/labels",            post(add_issue_labels))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/labels/{label_id}", delete(remove_issue_label))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/comments",          get(list_comments))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/comments",          post(create_comment))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/reactions",         post(add_issue_reaction))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/reactions/{id}",    delete(remove_issue_reaction))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/subscription",      put(subscribe_issue))
        .route("/api/v1/repos/{owner}/{repo}/issues/{number}/subscription",      delete(unsubscribe_issue))
        .route("/api/v1/repos/{owner}/{repo}/comments/{comment_id}",             patch(update_comment))
        .route("/api/v1/repos/{owner}/{repo}/comments/{comment_id}",             delete(delete_comment))
        .route("/api/v1/repos/{owner}/{repo}/comments/{comment_id}/reactions",   post(add_comment_reaction))

        // labels
        .route("/api/v1/repos/{owner}/{repo}/labels",            get(list_labels))
        .route("/api/v1/repos/{owner}/{repo}/labels",            post(create_label))
        .route("/api/v1/repos/{owner}/{repo}/labels/{label_id}", patch(update_label))
        .route("/api/v1/repos/{owner}/{repo}/labels/{label_id}", delete(delete_label))

        // milestones
        .route("/api/v1/repos/{owner}/{repo}/milestones",                 get(list_milestones))
        .route("/api/v1/repos/{owner}/{repo}/milestones",                 post(create_milestone))
        .route("/api/v1/repos/{owner}/{repo}/milestones/{milestone_id}",  patch(update_milestone))
        .route("/api/v1/repos/{owner}/{repo}/milestones/{milestone_id}",  delete(delete_milestone))

        // pull requests
        .route("/api/v1/repos/{owner}/{repo}/pulls",                                          get(list_prs))
        .route("/api/v1/repos/{owner}/{repo}/pulls",                                          post(create_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}",                                 get(get_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}",                                 patch(update_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/close",                           put(close_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/reopen",                          put(reopen_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/merge",                           put(merge_pr))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/merged",                          get(is_merged))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/reviews",                         get(list_reviews))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/reviews",                         post(create_review))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/reviews/{review_id}/dismissals",  put(dismiss_review))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/comments",                        get(list_review_comments))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/comments",                        post(create_review_comment))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/requested_reviewers",             post(request_reviewers))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/requested_reviewers",             delete(remove_reviewers))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/labels",                          post(add_pr_labels))
        .route("/api/v1/repos/{owner}/{repo}/pulls/{number}/assignees",                       post(add_pr_assignees))
        .route("/api/v1/repos/{owner}/{repo}/pulls/comments/{comment_id}",                    patch(update_review_comment))
        .route("/api/v1/repos/{owner}/{repo}/pulls/comments/{comment_id}",                    delete(delete_review_comment))
        .route("/api/v1/repos/{owner}/{repo}/pulls/comments/{comment_id}/resolve",            put(resolve_review_comment))
        .route("/api/v1/repos/{owner}/{repo}/statuses/{sha}",                                 get(list_statuses))
        .route("/api/v1/repos/{owner}/{repo}/statuses/{sha}",                                 post(create_status))

        // releases
        .route("/api/v1/repos/{owner}/{repo}/releases",                         get(list_releases))
        .route("/api/v1/repos/{owner}/{repo}/releases",                         post(create_release))
        .route("/api/v1/repos/{owner}/{repo}/releases/latest",                  get(get_latest_release))
        .route("/api/v1/repos/{owner}/{repo}/releases/tags/{tag}",              get(get_release_by_tag))
        .route("/api/v1/repos/{owner}/{repo}/releases/{release_id}",            get(get_release))
        .route("/api/v1/repos/{owner}/{repo}/releases/{release_id}",            patch(update_release))
        .route("/api/v1/repos/{owner}/{repo}/releases/{release_id}",            delete(delete_release))
        .route("/api/v1/repos/{owner}/{repo}/releases/{release_id}/assets",     get(list_assets))
        .route("/api/v1/repos/{owner}/{repo}/releases/{release_id}/assets",     post(upload_asset))
        .route("/api/v1/repos/{owner}/{repo}/releases/{release_id}/assets/{asset_id}", get(download_asset))
        .route("/api/v1/repos/{owner}/{repo}/releases/{release_id}/assets/{asset_id}", delete(delete_asset))

        // git smart http
        .route("/{owner}/{repo}/info/refs",        get(info_refs))
        .route("/{owner}/{repo}/git-upload-pack",  post(git_upload_pack))
        .route("/{owner}/{repo}/git-receive-pack", post(git_receive_pack))

        .with_state(state)
}