import { apiClient } from "./client";

export const orgsApi = {
    list:   () => apiClient.get("/user/orgs"),

    get:    (org: string) => apiClient.get(`/orgs/${org}`),

    create: (data: object) => apiClient.post("/orgs", data),

    update: (org: string, data: object) => apiClient.patch(`/orgs/${org}`, data),

    delete: (org: string) => apiClient.delete(`/orgs/${org}`),

    repos:  (org: string, page = 1) =>
        apiClient.get(`/orgs/${org}/repos`, { params: { page } }),

    members: {
        list:   (org: string) => apiClient.get(`/orgs/${org}/members`),
        get:    (org: string, username: string) =>
            apiClient.get(`/orgs/${org}/members/${username}`),
        remove: (org: string, username: string) =>
            apiClient.delete(`/orgs/${org}/members/${username}`),
        updateRole: (org: string, username: string, role: string) =>
            apiClient.patch(`/orgs/${org}/members/${username}/role`, { role }),
    },

    invitations: {
        list:   (org: string) => apiClient.get(`/orgs/${org}/invitations`),
        create: (org: string, email: string, role = "member") =>
            apiClient.post(`/orgs/${org}/invitations`, { email, role }),
        cancel: (org: string, id: string) =>
            apiClient.delete(`/orgs/${org}/invitations/${id}`),
        accept:  (token: string) => apiClient.post(`/invitations/${token}/accept`),
        decline: (token: string) => apiClient.post(`/invitations/${token}/decline`),
    },

    teams: {
        list:   (org: string) => apiClient.get(`/orgs/${org}/teams`),
        get:    (org: string, slug: string) => apiClient.get(`/orgs/${org}/teams/${slug}`),
        create: (org: string, data: object)  => apiClient.post(`/orgs/${org}/teams`, data),
        update: (org: string, slug: string, data: object) =>
            apiClient.patch(`/orgs/${org}/teams/${slug}`, data),
        delete: (org: string, slug: string) =>
            apiClient.delete(`/orgs/${org}/teams/${slug}`),
        members: {
            list:   (org: string, slug: string) =>
                apiClient.get(`/orgs/${org}/teams/${slug}/members`),
            add:    (org: string, slug: string, username: string) =>
                apiClient.put(`/orgs/${org}/teams/${slug}/members/${username}`),
            remove: (org: string, slug: string, username: string) =>
                apiClient.delete(`/orgs/${org}/teams/${slug}/members/${username}`),
        },
    },
};