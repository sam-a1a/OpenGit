import { apiClient } from "./client";

export const notificationsApi = {
    list:       (params = {}) => apiClient.get("/notifications", { params }),
    count:      () => apiClient.get("/notifications/count"),
    markAllRead: (data = {}) => apiClient.put("/notifications", data),
    deleteRead: () => apiClient.delete("/notifications"),

    get:    (id: string) => apiClient.get(`/notifications/${id}`),
    read:   (id: string) => apiClient.patch(`/notifications/${id}/read`),
    save:   (id: string) => apiClient.put(`/notifications/${id}/save`),
    unsave: (id: string) => apiClient.delete(`/notifications/${id}/save`),
    delete: (id: string) => apiClient.delete(`/notifications/${id}`),

    repo: {
        list:      (owner: string, repo: string) =>
            apiClient.get(`/repos/${owner}/${repo}/notifications`),
        markRead:  (owner: string, repo: string) =>
            apiClient.put(`/repos/${owner}/${repo}/notifications`),
        subscribe: (owner: string, repo: string, data: object) =>
            apiClient.put(`/repos/${owner}/${repo}/subscription`, data),
        unsubscribe: (owner: string, repo: string) =>
            apiClient.delete(`/repos/${owner}/${repo}/subscription`),
    },
};