import { apiClient } from "./client";

export const authApi = {
    register: (username: string, email: string, password: string) =>
        apiClient.post("/auth/register", { username, email, password }),

    login: (email: string, password: string) =>
        apiClient.post("/auth/login", { email, password }),

    logout: () =>
        apiClient.post("/auth/logout"),

    refresh: (refresh_token: string) =>
        apiClient.post("/auth/refresh", { refresh_token }),

    me: () =>
        apiClient.get("/auth/me"),

    verify2fa: (pending_token: string, code: string) =>
        apiClient.post("/auth/2fa/verify", { pending_token, code }),
};