import { Routes, Route, Navigate } from "react-router-dom";
import { useAuthStore } from "./stores/auth";
import { useUiStore } from "./stores/ui";
import { useEffect } from "react";

import LoginPage           from "./pages/auth/LoginPage";
import RegisterPage        from "./pages/auth/RegisterPage";
import ExplorePage         from "./pages/explore/ExplorePage";
import ProfilePage         from "./pages/user/ProfilePage";
import RepoLayout          from "./pages/repo/RepoLayout";
import IssuePage           from "./pages/repo/IssuePage";
import FileViewerPage      from "./pages/repo/FileViewerPage";
import SettingsPage        from "./pages/settings/SettingsPage";
import NotificationsPage   from "./pages/notifications/NotificationsPage";
import NewRepoPage         from "./pages/repo/NewRepoPage";

function PrivateRoute({ children }: { children: React.ReactNode }) {
    const token = useAuthStore((s) => s.access_token);
    return token ? <>{children}</> : <Navigate to="/login" replace />;
}

export default function App() {
    const theme = useUiStore((s) => s.theme);

    useEffect(() => {
        document.documentElement.classList.toggle("dark", theme === "dark");
    }, [theme]);

    return (
        <Routes>
            {/* public */}
            <Route path="/login"    element={<LoginPage />} />
            <Route path="/register" element={<RegisterPage />} />
            <Route path="/explore"  element={<ExplorePage />} />

            {/* protected */}
            <Route path="/new" element={
                <PrivateRoute><NewRepoPage /></PrivateRoute>
            } />
            <Route path="/notifications" element={
                <PrivateRoute><NotificationsPage /></PrivateRoute>
            } />
            <Route path="/settings/*" element={
                <PrivateRoute><SettingsPage /></PrivateRoute>
            } />

            {/* repo — issue detail */}
            <Route path="/:owner/:repo/issues/:number"
                   element={<IssuePage />} />

            {/* repo — file viewer */}
            <Route path="/:owner/:repo/blob/:ref/*"
                   element={<FileViewerPage />} />

            {/* repo — all other tabs */}
            <Route path="/:owner/:repo/*" element={<RepoLayout />} />

            {/* user profile */}
            <Route path="/:username" element={<ProfilePage />} />

            {/* default */}
            <Route path="/" element={
                <PrivateRoute>
                    <Navigate to="/explore" replace />
                </PrivateRoute>
            } />

            <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
    );
}