import { apiClient } from "./client";

export const reposApi = {
    list:    (page = 1, perPage = 30) =>
        apiClient.get("/user/repos", { params: { page, per_page: perPage } }),

    get:     (owner: string, repo: string) =>
        apiClient.get(`/repos/${owner}/${repo}`),

    create:  (data: object) =>
        apiClient.post("/repos", data),

    update:  (owner: string, repo: string, data: object) =>
        apiClient.patch(`/repos/${owner}/${repo}`, data),

    delete:  (owner: string, repo: string) =>
        apiClient.delete(`/repos/${owner}/${repo}`),

    fork:    (owner: string, repo: string) =>
        apiClient.post(`/repos/${owner}/${repo}/forks`),

    star:    (owner: string, repo: string) =>
        apiClient.put(`/repos/${owner}/${repo}/star`),

    unstar:  (owner: string, repo: string) =>
        apiClient.delete(`/repos/${owner}/${repo}/star`),

    // git browser
    refs:    (owner: string, repo: string) =>
        apiClient.get(`/repos/${owner}/${repo}/git/refs`),

    tree:    (owner: string, repo: string, ref: string, path = "") =>
        apiClient.get(`/repos/${owner}/${repo}/git/tree/${ref}`, { params: { path } }),

    blob:    (owner: string, repo: string, ref: string, path: string) =>
        apiClient.get(`/repos/${owner}/${repo}/git/blob/${ref}`, { params: { path } }),

    commits: (owner: string, repo: string, ref: string, page = 1) =>
        apiClient.get(`/repos/${owner}/${repo}/git/commits/${ref}`, { params: { page } }),

    commit:  (owner: string, repo: string, sha: string) =>
        apiClient.get(`/repos/${owner}/${repo}/git/commits/${sha}/single`),

    diff:    (owner: string, repo: string, base: string, head: string) =>
        apiClient.get(`/repos/${owner}/${repo}/git/diff`, { params: { base, head } }),

    blame:   (owner: string, repo: string, ref: string, path: string) =>
        apiClient.get(`/repos/${owner}/${repo}/git/blame/${ref}`, { params: { path } }),

    stats:   (owner: string, repo: string) =>
        apiClient.get(`/repos/${owner}/${repo}/git/stats`),

    search:  (q: string, page = 1) =>
        apiClient.get("/repos/search", { params: { q, page } }),
};