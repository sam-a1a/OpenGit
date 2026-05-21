import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { NavLink, Routes, Route, Navigate } from "react-router-dom";
import {
    User, Key, Shield, Bell, Palette,
    Trash2, Plus, Eye, EyeOff, Check
} from "lucide-react";
import { Header } from "../../components/layout/Header";
import { Button } from "../../components/ui/Button";
import { Input } from "../../components/ui/Input";
import { Alert } from "../../components/ui/Alert";
import { Avatar } from "../../components/ui/Avatar";
import { Badge } from "../../components/ui/Badge";
import { usersApi } from "../../api/users";
import { useAuthStore } from "../../stores/auth";
import { apiClient } from "../../api/client";
import { cn, formatDate } from "../../lib/utils";
import { Copy } from "lucide-react";
import { PageSpinner } from "../../components/ui/Spinner";
import { useUiStore } from "../../stores/ui";

const navItems = [
    { path: "profile",    label: "Profile",       icon: <User className="w-4 h-4" /> },
    { path: "security",   label: "Password & Auth", icon: <Shield className="w-4 h-4" /> },
    { path: "keys",       label: "SSH Keys",       icon: <Key className="w-4 h-4" /> },
    { path: "tokens",     label: "Access Tokens",  icon: <Key className="w-4 h-4" /> },
    { path: "notifications", label: "Notifications", icon: <Bell className="w-4 h-4" /> },
    { path: "appearance", label: "Appearance",      icon: <Palette className="w-4 h-4" /> },
];

export default function SettingsPage() {
    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            <Header />
            <div className="max-w-5xl mx-auto px-4 py-8">
                <h1 className="text-xl font-bold text-gray-900 dark:text-white mb-6">Settings</h1>
                <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
                    <nav className="space-y-1">
                        {navItems.map((item) => (
                            <NavLink
                                key={item.path}
                                to={`/settings/${item.path}`}
                                className={({ isActive }) => cn(
                                    "flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm font-medium transition-colors",
                                    isActive
                                        ? "bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-400"
                                        : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-white"
                                )}
                            >
                                {item.icon}
                                {item.label}
                            </NavLink>
                        ))}
                    </nav>

                    <div className="md:col-span-3">
                        <Routes>
                            <Route index element={<Navigate to="profile" replace />} />
                            <Route path="profile"       element={<ProfileSettings />} />
                            <Route path="security"      element={<SecuritySettings />} />
                            <Route path="keys"          element={<SshKeysSettings />} />
                            <Route path="tokens"        element={<TokensSettings />} />
                            <Route path="notifications" element={<NotificationSettings />} />
                            <Route path="appearance"    element={<AppearanceSettings />} />
                        </Routes>
                    </div>
                </div>
            </div>
        </div>
    );
}

// Profile settings

function ProfileSettings() {
    const user        = useAuthStore((s) => s.user);
    const updateUser  = useAuthStore((s) => s.updateUser);
    const queryClient = useQueryClient();
    const [success, setSuccess] = useState("");
    const [error,   setError]   = useState("");

    const [form, setForm] = useState({
        display_name:     user?.display_name ?? "",
        bio:              user?.bio ?? "",
        website:          user?.website ?? "",
        location:         user?.location ?? "",
        company:          user?.company ?? "",
        twitter_username: user?.twitter_username ?? "",
        pronouns:         user?.pronouns ?? "",
    });

    const mutation = useMutation({
        mutationFn: () => usersApi.update(form),
        onSuccess:  (r) => {
            updateUser(r.data);
            setSuccess("Profile updated successfully");
            setTimeout(() => setSuccess(""), 3000);
        },
        onError: (e: any) => setError(e.response?.data?.error ?? "Failed to update profile"),
    });

    return (
        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6 space-y-5">
            <h2 className="text-base font-semibold text-gray-900 dark:text-white">Public profile</h2>

            {success && <Alert type="success">{success}</Alert>}
            {error   && <Alert type="error" onClose={() => setError("")}>{error}</Alert>}

            <div className="flex items-center gap-4">
                <Avatar username={user?.username ?? ""} src={user?.avatar_url} size="xl" />
                <div>
                    <p className="text-sm font-medium text-gray-700 dark:text-gray-300">{user?.username}</p>
                    <Button variant="outline" size="sm" className="mt-1">Change avatar</Button>
                </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <Input label="Display name"
                       value={form.display_name}
                       onChange={(e) => setForm({ ...form, display_name: e.target.value })} />
                <Input label="Pronouns"
                       value={form.pronouns}
                       onChange={(e) => setForm({ ...form, pronouns: e.target.value })}
                       hint="e.g. they/them" />
            </div>

            <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Bio</label>
                <textarea
                    value={form.bio}
                    onChange={(e) => setForm({ ...form, bio: e.target.value })}
                    rows={3}
                    placeholder="Tell us about yourself"
                    maxLength={160}
                    className="w-full px-3 py-2 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                />
                <p className="text-xs text-gray-400 mt-1">{form.bio.length}/160</p>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <Input label="Company"
                       value={form.company}
                       onChange={(e) => setForm({ ...form, company: e.target.value })} />
                <Input label="Location"
                       value={form.location}
                       onChange={(e) => setForm({ ...form, location: e.target.value })} />
                <Input label="Website"
                       type="url"
                       value={form.website}
                       onChange={(e) => setForm({ ...form, website: e.target.value })} />
                <Input label="Twitter username"
                       value={form.twitter_username}
                       onChange={(e) => setForm({ ...form, twitter_username: e.target.value })}
                       hint="Without the @" />
            </div>

            <Button loading={mutation.isPending} onClick={() => mutation.mutate()}>
                Save profile
            </Button>
        </div>
    );
}

// Security settings

function SecuritySettings() {
    const [showCurrent, setShowCurrent] = useState(false);
    const [showNew,     setShowNew]     = useState(false);
    const [form, setForm] = useState({ current: "", newPw: "", confirm: "" });
    const [error,   setError]   = useState("");
    const [success, setSuccess] = useState("");

    const { data: twoFaData } = useQuery({
        queryKey: ["2fa-status"],
        queryFn:  () => apiClient.get("/user/2fa").then((r) => r.data),
    });

    const changePassword = useMutation({
        mutationFn: () => apiClient.post("/user/password", {
            current_password: form.current,
            new_password:     form.newPw,
        }),
        onSuccess: () => {
            setSuccess("Password changed successfully");
            setForm({ current: "", newPw: "", confirm: "" });
        },
        onError: (e: any) => setError(e.response?.data?.error ?? "Failed to change password"),
    });

    const handleSubmit = () => {
        setError("");
        if (form.newPw !== form.confirm) {
            setError("Passwords do not match");
            return;
        }
        if (form.newPw.length < 8) {
            setError("Password must be at least 8 characters");
            return;
        }
        changePassword.mutate();
    };

    return (
        <div className="space-y-4">
            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6 space-y-4">
                <h2 className="text-base font-semibold text-gray-900 dark:text-white">Change password</h2>
                {error   && <Alert type="error"   onClose={() => setError("")}>{error}</Alert>}
                {success && <Alert type="success">{success}</Alert>}

                <div className="relative">
                    <Input
                        label="Current password"
                        type={showCurrent ? "text" : "password"}
                        value={form.current}
                        onChange={(e) => setForm({ ...form, current: e.target.value })}
                    />
                    <button onClick={() => setShowCurrent(!showCurrent)}
                            className="absolute right-3 top-8 text-gray-400">
                        {showCurrent ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </button>
                </div>
                <div className="relative">
                    <Input
                        label="New password"
                        type={showNew ? "text" : "password"}
                        value={form.newPw}
                        onChange={(e) => setForm({ ...form, newPw: e.target.value })}
                        hint="At least 8 characters"
                    />
                    <button onClick={() => setShowNew(!showNew)}
                            className="absolute right-3 top-8 text-gray-400">
                        {showNew ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </button>
                </div>
                <Input
                    label="Confirm new password"
                    type="password"
                    value={form.confirm}
                    onChange={(e) => setForm({ ...form, confirm: e.target.value })}
                />
                <Button loading={changePassword.isPending} onClick={handleSubmit}>
                    Update password
                </Button>
            </div>

            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6">
                <div className="flex items-start justify-between">
                    <div>
                        <h2 className="text-base font-semibold text-gray-900 dark:text-white mb-1">
                            Two-factor authentication
                        </h2>
                        <p className="text-sm text-gray-500">
                            Add an extra layer of security to your account.
                        </p>
                    </div>
                    <Badge variant={twoFaData?.enabled ? "success" : "default"}>
                        {twoFaData?.enabled ? "Enabled" : "Disabled"}
                    </Badge>
                </div>
                <div className="mt-4">
                    {twoFaData?.enabled ? (
                        <div className="space-y-2">
                            <p className="text-sm text-gray-600 dark:text-gray-400">
                                {twoFaData.backup_codes_remaining} backup codes remaining
                            </p>
                            <div className="flex gap-2">
                                <Button variant="outline" size="sm">Regenerate backup codes</Button>
                                <Button variant="danger"  size="sm">Disable 2FA</Button>
                            </div>
                        </div>
                    ) : (
                        <Button size="sm" onClick={() => window.location.href = "/settings/2fa/setup"}>
                            Enable 2FA
                        </Button>
                    )}
                </div>
            </div>
        </div>
    );
}

// SSH keys

function SshKeysSettings() {
    const queryClient = useQueryClient();
    const [title,   setTitle]   = useState("");
    const [key,     setKey]     = useState("");
    const [error,   setError]   = useState("");
    const [success, setSuccess] = useState("");

    const { data: keys, isLoading } = useQuery({
        queryKey: ["ssh-keys"],
        queryFn:  () => usersApi.sshKeys.list().then((r) => r.data),
    });

    const addKey = useMutation({
        mutationFn: () => usersApi.sshKeys.add(title, key),
        onSuccess:  () => {
            setTitle(""); setKey("");
            setSuccess("SSH key added");
            queryClient.invalidateQueries({ queryKey: ["ssh-keys"] });
        },
        onError: (e: any) => setError(e.response?.data?.error ?? "Failed to add key"),
    });

    const deleteKey = useMutation({
        mutationFn: (id: string) => usersApi.sshKeys.delete(id),
        onSuccess:  () => queryClient.invalidateQueries({ queryKey: ["ssh-keys"] }),
    });

    return (
        <div className="space-y-4">
            {error   && <Alert type="error"   onClose={() => setError("")}>{error}</Alert>}
            {success && <Alert type="success" onClose={() => setSuccess("")}>{success}</Alert>}

            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6 space-y-4">
                <h2 className="text-base font-semibold text-gray-900 dark:text-white">Add SSH key</h2>
                <Input label="Title" value={title}
                       onChange={(e) => setTitle(e.target.value)}
                       placeholder="My MacBook Pro" />
                <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Key
                    </label>
                    <textarea
                        value={key}
                        onChange={(e) => setKey(e.target.value)}
                        rows={4}
                        placeholder="Begins with 'ssh-rsa', 'ssh-ed25519', etc."
                        className="w-full px-3 py-2 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono"
                    />
                </div>
                <Button
                    icon={<Plus className="w-4 h-4" />}
                    loading={addKey.isPending}
                    disabled={!title.trim() || !key.trim()}
                    onClick={() => addKey.mutate()}
                >
                    Add SSH key
                </Button>
            </div>

            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
                <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-800">
                    <h2 className="text-base font-semibold text-gray-900 dark:text-white">Your SSH keys</h2>
                </div>
                {isLoading ? (
                    <div className="py-8"><PageSpinner /></div>
                ) : !keys?.length ? (
                    <div className="py-10 text-center text-gray-500 text-sm">
                        No SSH keys added yet
                    </div>
                ) : (
                    <div className="divide-y divide-gray-100 dark:divide-gray-800">
                        {keys.map((k: any) => (
                            <div key={k.id} className="flex items-start justify-between px-6 py-4">
                                <div>
                                    <p className="text-sm font-medium text-gray-900 dark:text-white">{k.title}</p>
                                    <p className="text-xs font-mono text-gray-500 mt-1 truncate max-w-xs">
                                        {k.fingerprint}
                                    </p>
                                    <p className="text-xs text-gray-400 mt-0.5">
                                        Added {formatDate(k.created_at)}
                                        {k.last_used_at && ` · Last used ${formatDate(k.last_used_at)}`}
                                    </p>
                                </div>
                                <button
                                    onClick={() => deleteKey.mutate(k.id)}
                                    className="p-1.5 rounded text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"
                                >
                                    <Trash2 className="w-4 h-4" />
                                </button>
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

// Access tokens

function TokensSettings() {
    const queryClient = useQueryClient();
    const [name,    setName]    = useState("");
    const [newToken, setNewToken] = useState<string | null>(null);
    const [copied,  setCopied]  = useState(false);

    const { data: tokens, isLoading } = useQuery({
        queryKey: ["pat-tokens"],
        queryFn:  () => apiClient.get("/user/tokens").then((r) => r.data).catch(() => []),
    });

    const createToken = useMutation({
        mutationFn: () => apiClient.post("/user/tokens", { name, scopes: ["repo", "user"] }),
        onSuccess:  (r) => {
            setNewToken(r.data.token);
            setName("");
            queryClient.invalidateQueries({ queryKey: ["pat-tokens"] });
        },
    });

    const copyToken = () => {
        if (newToken) {
            navigator.clipboard.writeText(newToken);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    return (
        <div className="space-y-4">
            {newToken && (
                <Alert type="success" title="Token created">
                    <p className="mb-2">Make sure to copy your token now — it won't be shown again.</p>
                    <div className="flex gap-2 mt-2">
                        <code className="flex-1 text-xs bg-green-100 dark:bg-green-900/40 px-2 py-1.5 rounded font-mono break-all">
                            {newToken}
                        </code>
                        <button onClick={copyToken}
                                className="p-1.5 rounded border border-green-300 hover:bg-green-100 text-green-700">
                            {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                        </button>
                    </div>
                </Alert>
            )}

            <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6 space-y-4">
                <h2 className="text-base font-semibold text-gray-900 dark:text-white">New token</h2>
                <Input label="Token name" value={name}
                       onChange={(e) => setName(e.target.value)}
                       placeholder="My CI token" />
                <Button
                    loading={createToken.isPending}
                    disabled={!name.trim()}
                    onClick={() => createToken.mutate()}
                >
                    Generate token
                </Button>
            </div>
        </div>
    );
}

// Notification settings

function NotificationSettings() {
    return (
        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6">
            <h2 className="text-base font-semibold text-gray-900 dark:text-white mb-4">Notifications</h2>
            <p className="text-sm text-gray-500">Notification preferences coming soon.</p>
        </div>
    );
}

// Appearance

function AppearanceSettings() {
    const { theme, setTheme } = useUiStore();

    return (
        <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6">
            <h2 className="text-base font-semibold text-gray-900 dark:text-white mb-4">Appearance</h2>
            <div className="space-y-2">
                {(["light", "dark"] as const).map((t) => (
                    <button
                        key={t}
                        onClick={() => setTheme(t)}
                        className={cn(
                            "w-full flex items-center gap-3 px-4 py-3 rounded-lg border text-sm font-medium transition-colors",
                            theme === t
                                ? "border-blue-500 bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-400"
                                : "border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800"
                        )}
                    >
                        {t === "light" ? "☀️ Light" : "🌙 Dark"}
                        {theme === t && <Check className="w-4 h-4 ml-auto" />}
                    </button>
                ))}
            </div>
        </div>
    );
}
