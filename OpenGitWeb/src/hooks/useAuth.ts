import { useAuthStore } from "../stores/auth";
import { apiClient } from "../api/client";
import { useNavigate } from "react-router-dom";
import { useCallback } from "react";

export function useAuth() {
    const { user, access_token, setAuth, clearAuth } = useAuthStore();
    const navigate = useNavigate();

    const login = useCallback(
        async (email: string, password: string) => {
            const { data } = await apiClient.post("/auth/login", { email, password });

            if (data.two_factor_required) {
                return { twoFactorRequired: true, pendingToken: data.pending_token };
            }

            localStorage.setItem("access_token",  data.access_token);
            localStorage.setItem("refresh_token", data.refresh_token);
            setAuth(data.user, data.access_token, data.refresh_token);
            return { twoFactorRequired: false };
        },
        [setAuth]
    );

    const register = useCallback(
        async (username: string, email: string, password: string) => {
            const { data } = await apiClient.post("/auth/register", {
                username, email, password,
            });
            localStorage.setItem("access_token",  data.access_token);
            localStorage.setItem("refresh_token", data.refresh_token);
            setAuth(data.user, data.access_token, data.refresh_token);
        },
        [setAuth]
    );

    const logout = useCallback(async () => {
        try { await apiClient.post("/auth/logout"); } catch {}
        localStorage.removeItem("access_token");
        localStorage.removeItem("refresh_token");
        clearAuth();
        navigate("/login");
    }, [clearAuth, navigate]);

    return {
        user,
        isAuthenticated: !!access_token,
        login,
        register,
        logout,
    };
}