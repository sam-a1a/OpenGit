import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { User } from "../types/user";

interface AuthStore {
    user:          User | null;
    access_token:  string | null;
    refresh_token: string | null;
    setAuth:       (user: User, access: string, refresh: string) => void;
    clearAuth:     () => void;
    updateUser:    (user: Partial<User>) => void;
}

export const useAuthStore = create<AuthStore>()(
    persist(
        (set) => ({
            user:          null,
            access_token:  null,
            refresh_token: null,

            setAuth: (user, access_token, refresh_token) =>
                set({ user, access_token, refresh_token }),

            clearAuth: () =>
                set({ user: null, access_token: null, refresh_token: null }),

            updateUser: (partial) =>
                set((state) => ({
                    user: state.user ? { ...state.user, ...partial } : null,
                })),
        }),
        {
            name: "opengit-auth",
            onRehydrateStorage: () => (state) => {
                if (state?.access_token) {
                    localStorage.setItem("access_token", state.access_token);
                }
                if (state?.refresh_token) {
                    localStorage.setItem("refresh_token", state.refresh_token);
                }
            },
        }
    )
);