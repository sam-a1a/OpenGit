import { apiClient } from "./client";

export const issuesApi = {
    list:   (owner: string, repo: string, params = {}) =>
        apiClient.get(`/repos/${owner}/${repo}/issues`, { params }),

    get:    (owner: string, repo: string, number: number) =>
        apiClient.get(`/repos/${owner}/${repo}/issues/${number}`),

    create: (owner: string, repo: string, data: object) =>
        apiClient.post(`/repos/${owner}/${repo}/issues`, data),

    update: (owner: string, repo: string, number: number, data: object) =>
        apiClient.patch(`/repos/${owner}/${repo}/issues/${number}`, data),

    close:  (owner: string, repo: string, number: number) =>
        apiClient.patch(`/repos/${owner}/${repo}/issues/${number}`, { state: "closed" }),

    reopen: (owner: string, repo: string, number: number) =>
        apiClient.patch(`/repos/${owner}/${repo}/issues/${number}`, { state: "open" }),

    comments: {
        list:   (owner: string, repo: string, number: number) =>
            apiClient.get(`/repos/${owner}/${repo}/issues/${number}/comments`),
        create: (owner: string, repo: string, number: number, body: string) =>
            apiClient.post(`/repos/${owner}/${repo}/issues/${number}/comments`, { body }),
        update: (owner: string, repo: string, id: string, body: string) =>
            apiClient.patch(`/repos/${owner}/${repo}/comments/${id}`, { body }),
        delete: (owner: string, repo: string, id: string) =>
            apiClient.delete(`/repos/${owner}/${repo}/comments/${id}`),
    },

    labels: {
        list:   (owner: string, repo: string) =>
            apiClient.get(`/repos/${owner}/${repo}/labels`),
        create: (owner: string, repo: string, data: object) =>
            apiClient.post(`/repos/${owner}/${repo}/labels`, data),
    },

    milestones: {
        list:   (owner: string, repo: string) =>
            apiClient.get(`/repos/${owner}/${repo}/milestones`),
        create: (owner: string, repo: string, data: object) =>
            apiClient.post(`/repos/${owner}/${repo}/milestones`, data),
    },
};