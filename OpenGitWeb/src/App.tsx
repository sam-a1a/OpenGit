import { Routes, Route, Navigate } from "react-router-dom";
import { useAuthStore } from "./stores/auth";
import { useUiStore } from "./stores/ui";
import { useEffect } from "react";
import AdminPage from "./pages/admin/AdminPage";

import LandingPage         from "./pages/LandingPage";
import LoginPage           from "./pages/auth/LoginPage";
import RegisterPage        from "./pages/auth/RegisterPage";
import ExplorePage         from "./pages/explore/ExplorePage";
import ProfilePage         from "./pages/user/ProfilePage";
import RepoLayout          from "./pages/repo/RepoLayout";
import IssuePage           from "./pages/repo/IssuePage";
import PullRequestPage     from "./pages/repo/PullRequestPage";
import FileViewerPage      from "./pages/repo/FileViewerPage";
import CommitHistoryPage   from "./pages/repo/CommitHistoryPage";
import SettingsPage        from "./pages/settings/SettingsPage";
import NotificationsPage   from "./pages/notifications/NotificationsPage";
import NewRepoPage         from "./pages/repo/NewRepoPage";
import SearchPage          from "./pages/search/SearchPage";
import OrgPage             from "./pages/org/OrgPage";

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
            {/* landing */}
            <Route path="/" element={<LandingPage />} />

            {/* auth */}
            <Route path="/login"    element={<LoginPage />} />
            <Route path="/register" element={<RegisterPage />} />

            {/* public */}
            <Route path="/explore" element={<ExplorePage />} />
            <Route path="/search"  element={<SearchPage />} />

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

            {/* org */}
            <Route path="/orgs/:org/*" element={<OrgPage />} />

            {/* repo — specific pages first */}
            <Route path="/:owner/:repo/issues/:number"
                   element={<IssuePage />} />
            <Route path="/:owner/:repo/pulls/:number"
                   element={<PullRequestPage />} />
            <Route path="/:owner/:repo/blob/:ref/*"
                   element={<FileViewerPage />} />
            <Route path="/:owner/:repo/commits/*"
                   element={<CommitHistoryPage />} />

            {/* repo — layout with tabs */}
            <Route path="/:owner/:repo/*" element={<RepoLayout />} />

            <Route path="/admin/*" element={
                <PrivateRoute><AdminPage /></PrivateRoute>
            } />

            {/* user profile — last, catches /:username */}
            <Route path="/:username" element={<ProfilePage />} />

            <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
    );
}