import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface User {
    id:                  string;
    username:            string;
    display_name:        string | null;
    bio:                 string | null;
    avatar_url:          string | null;
    website:             string | null;
    location:            string | null;
    pronouns:            string | null;
    company:             string | null;
    twitter_username:    string | null;
    role:                string;
    status_emoji:        string | null;
    status_message:      string | null;
    status_availability: string;
    is_active:           boolean;
    is_verified:         boolean;
    two_factor_enabled:  boolean;
    profile_private:     boolean;
    created_at:          string;
    updated_at:          string;
}

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
                if (state?.access_token)
                    localStorage.setItem("access_token", state.access_token);
                if (state?.refresh_token)
                    localStorage.setItem("refresh_token", state.refresh_token);
            },
        }
    )
);