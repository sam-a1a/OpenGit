import { apiClient } from "./client";

export const usersApi = {
    me:       () => apiClient.get("/user"),

    update:   (data: object) => apiClient.patch("/user", data),

    get:      (username: string) => apiClient.get(`/users/${username}`),

    repos:    (username: string, page = 1) =>
        apiClient.get(`/users/${username}/repos`, { params: { page } }),

    followers: (username: string) =>
        apiClient.get(`/users/${username}/followers`),

    following: (username: string) =>
        apiClient.get(`/users/${username}/following`),

    follow:   (username: string) =>
        apiClient.put(`/users/${username}/follow`),

    unfollow: (username: string) =>
        apiClient.delete(`/users/${username}/follow`),

    search:   (q: string, page = 1) =>
        apiClient.get("/users/search", { params: { q, page } }),

    sshKeys: {
        list:   () => apiClient.get("/user/keys"),
        add:    (title: string, key: string) =>
            apiClient.post("/user/keys", { title, key }),
        delete: (id: string) => apiClient.delete(`/user/keys/${id}`),
    },
};