import axios from "axios";
import { API_BASE } from "../lib/constants";

export const apiClient = axios.create({
    baseURL: `${API_BASE}/api/v1`,
    headers: { "Content-Type": "application/json" },
});

apiClient.interceptors.request.use((config) => {
    const token = localStorage.getItem("access_token");
    if (token) config.headers.Authorization = `Bearer ${token}`;
    return config;
});

apiClient.interceptors.response.use(
    (res) => res,
    async (error) => {
        const original = error.config;
        if (error.response?.status === 401 && !original._retry) {
            original._retry = true;
            const refresh = localStorage.getItem("refresh_token");
            if (refresh) {
                try {
                    const { data } = await axios.post(`${API_BASE}/api/v1/auth/refresh`, {
                        refresh_token: refresh,
                    });
                    localStorage.setItem("access_token",  data.access_token);
                    localStorage.setItem("refresh_token", data.refresh_token);
                    original.headers.Authorization = `Bearer ${data.access_token}`;
                    return apiClient(original);
                } catch {
                    localStorage.removeItem("access_token");
                    localStorage.removeItem("refresh_token");
                    window.location.href = "/login";
                }
            }
        }
        return Promise.reject(error);
    }
);