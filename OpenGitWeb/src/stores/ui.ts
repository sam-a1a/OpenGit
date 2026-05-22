import { create } from "zustand";
import { persist } from "zustand/middleware";

interface UiStore {
    theme:         "light" | "dark";
    sidebarOpen:   boolean;
    toggleTheme:   () => void;
    setTheme:      (t: "light" | "dark") => void;
    toggleSidebar: () => void;
}

export const useUiStore = create<UiStore>()(
    persist(
        (set) => ({
            theme:       "light",
            sidebarOpen: true,

            toggleTheme: () =>
                set((s) => {
                    const next = s.theme === "light" ? "dark" : "light";
                    document.documentElement.classList.toggle("dark", next === "dark");
                    return { theme: next };
                }),

            setTheme: (theme) => {
                document.documentElement.classList.toggle("dark", theme === "dark");
                set({ theme });
            },

            toggleSidebar: () =>
                set((s) => ({ sidebarOpen: !s.sidebarOpen })),
        }),
        { name: "opengit-ui" }
    )
);