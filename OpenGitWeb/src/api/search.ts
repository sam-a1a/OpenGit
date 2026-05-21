import { apiClient } from "./client";

export const searchApi = {
    all:      (q: string) => apiClient.get("/search", { params: { q } }),
    repos:    (q: string, page = 1) => apiClient.get("/search/repositories", { params: { q, page } }),
    issues:   (q: string, page = 1) => apiClient.get("/search/issues", { params: { q, page } }),
    users:    (q: string, page = 1) => apiClient.get("/search/users", { params: { q, page } }),
    pulls:    (q: string, page = 1) => apiClient.get("/search/pulls", { params: { q, page } }),
};