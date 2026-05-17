use crate::{
    api::routes::{
        auth::{login_handler, logout_handler, me_handler, refresh_handler, register_handler},
        health::health_check,
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
        .route("/api/v1/user",              get(get_me))
        .route("/api/v1/user",              patch(update_me))
        .route("/api/v1/user/status",       patch(update_status))
        .route("/api/v1/user/keys",         get(list_ssh_keys))
        .route("/api/v1/user/keys",         post(add_ssh_key))
        .route("/api/v1/user/keys/:key_id", delete(delete_ssh_key))
        .route("/api/v1/user/repos",        get(list_my_repos))

        // users
        .route("/api/v1/users/search",              get(search_users))
        .route("/api/v1/users/:username",            get(get_user))
        .route("/api/v1/users/:username/repos",      get(get_user_repos))
        .route("/api/v1/users/:username/followers",  get(get_followers))
        .route("/api/v1/users/:username/following",  get(get_following))
        .route("/api/v1/users/:username/follow",     put(follow_user))
        .route("/api/v1/users/:username/follow",     delete(unfollow_user))
        .route("/api/v1/users/:username/block",      put(block_user))
        .route("/api/v1/users/:username/block",      delete(unblock_user))

        // repos
        .route("/api/v1/repos/search",                          get(search_repos))
        .route("/api/v1/repos",                                 post(create_repo))
        .route("/api/v1/repos/:owner/:repo",                    get(get_repo))
        .route("/api/v1/repos/:owner/:repo",                    patch(update_repo))
        .route("/api/v1/repos/:owner/:repo",                    delete(delete_repo))
        .route("/api/v1/repos/:owner/:repo/forks",              get(list_forks))
        .route("/api/v1/repos/:owner/:repo/forks",              post(fork_repo))
        .route("/api/v1/repos/:owner/:repo/star",               put(star_repo))
        .route("/api/v1/repos/:owner/:repo/star",               delete(unstar_repo))
        .route("/api/v1/repos/:owner/:repo/watch",              put(watch_repo))
        .route("/api/v1/repos/:owner/:repo/watch",              delete(unwatch_repo))
        .route("/api/v1/repos/:owner/:repo/stargazers",         get(list_stargazers))
        .route("/api/v1/repos/:owner/:repo/topics",             get(get_topics))
        .route("/api/v1/repos/:owner/:repo/topics",             put(update_topics))
        .route("/api/v1/repos/:owner/:repo/archive",            put(archive_repo))
        .route("/api/v1/repos/:owner/:repo/archive",            delete(unarchive_repo))
        .route("/api/v1/repos/:owner/:repo/collaborators",      get(list_collaborators))
        .route("/api/v1/repos/:owner/:repo/collaborators/:username", put(add_collaborator))
        .route("/api/v1/repos/:owner/:repo/collaborators/:username", delete(remove_collaborator))
        .route("/api/v1/repos/:owner/:repo/branch-protections", get(list_branch_protections))
        .route("/api/v1/repos/:owner/:repo/branch-protections", post(create_branch_protection))

        .with_state(state)
}