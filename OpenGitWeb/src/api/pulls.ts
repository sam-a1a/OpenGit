import { apiClient } from "./client";

export const pullsApi = {
    list:   (owner: string, repo: string, params = {}) =>
        apiClient.get(`/repos/${owner}/${repo}/pulls`, { params }),

    get:    (owner: string, repo: string, number: number) =>
        apiClient.get(`/repos/${owner}/${repo}/pulls/${number}`),

    create: (owner: string, repo: string, data: object) =>
        apiClient.post(`/repos/${owner}/${repo}/pulls`, data),

    update: (owner: string, repo: string, number: number, data: object) =>
        apiClient.patch(`/repos/${owner}/${repo}/pulls/${number}`, data),

    close:  (owner: string, repo: string, number: number) =>
        apiClient.put(`/repos/${owner}/${repo}/pulls/${number}/close`),

    reopen: (owner: string, repo: string, number: number) =>
        apiClient.put(`/repos/${owner}/${repo}/pulls/${number}/reopen`),

    merge:  (owner: string, repo: string, number: number, method = "merge") =>
        apiClient.put(`/repos/${owner}/${repo}/pulls/${number}/merge`, {
            merge_method: method
        }),

    reviews: {
        list:   (owner: string, repo: string, number: number) =>
            apiClient.get(`/repos/${owner}/${repo}/pulls/${number}/reviews`),
        create: (owner: string, repo: string, number: number, data: object) =>
            apiClient.post(`/repos/${owner}/${repo}/pulls/${number}/reviews`, data),
        dismiss: (owner: string, repo: string, number: number, reviewId: string) =>
            apiClient.put(`/repos/${owner}/${repo}/pulls/${number}/reviews/${reviewId}/dismissals`),
    },

    comments: {
        list:   (owner: string, repo: string, number: number) =>
            apiClient.get(`/repos/${owner}/${repo}/pulls/${number}/comments`),
        create: (owner: string, repo: string, number: number, data: object) =>
            apiClient.post(`/repos/${owner}/${repo}/pulls/${number}/comments`, data),
        update: (owner: string, repo: string, commentId: string, body: string) =>
            apiClient.patch(`/repos/${owner}/${repo}/pulls/comments/${commentId}`, { body }),
        delete: (owner: string, repo: string, commentId: string) =>
            apiClient.delete(`/repos/${owner}/${repo}/pulls/comments/${commentId}`),
        resolve: (owner: string, repo: string, commentId: string) =>
            apiClient.put(`/repos/${owner}/${repo}/pulls/comments/${commentId}/resolve`),
    },

    requestReviewers: (owner: string, repo: string, number: number, reviewers: string[]) =>
        apiClient.post(`/repos/${owner}/${repo}/pulls/${number}/requested_reviewers`, { reviewers }),

    statuses: {
        list:   (owner: string, repo: string, sha: string) =>
            apiClient.get(`/repos/${owner}/${repo}/statuses/${sha}`),
        create: (owner: string, repo: string, sha: string, data: object) =>
            apiClient.post(`/repos/${owner}/${repo}/statuses/${sha}`, data),
    },
};