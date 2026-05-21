import { Link, useNavigate } from "react-router-dom";
import { useAuthStore } from "../../stores/auth";
import { useUiStore } from "../../stores/ui";
import { Avatar } from "../ui/Avatar";
import { Button } from "../ui/Button";
import {
    Bell, Search, Sun, Moon, Plus, GitBranch,
    Settings, LogOut, User, BookOpen, ChevronDown
} from "lucide-react";
import { useState, useRef, useEffect } from "react";
import { useDebounce } from "../../hooks/useDebounce";
import { searchApi } from "../../api/search";
import { APP_NAME } from "../../lib/constants";

export function Header() {
    const user        = useAuthStore((s) => s.user);
    const { theme, toggleTheme } = useUiStore();
    const navigate    = useNavigate();
    const clearAuth   = useAuthStore((s) => s.clearAuth);

    const [query,      setQuery]      = useState("");
    const [results,    setResults]    = useState<any>(null);
    const [showSearch, setShowSearch] = useState(false);
    const [showUser,   setShowUser]   = useState(false);
    const [showNew,    setShowNew]    = useState(false);

    const debounced = useDebounce(query, 300);
    const searchRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!debounced) { setResults(null); return; }
        searchApi.all(debounced).then((r) => setResults(r.data.results));
    }, [debounced]);

    useEffect(() => {
        const handler = (e: MouseEvent) => {
            if (searchRef.current && !searchRef.current.contains(e.target as Node)) {
                setShowSearch(false);
            }
        };
        document.addEventListener("mousedown", handler);
        return () => document.removeEventListener("mousedown", handler);
    }, []);

    const logout = () => {
        localStorage.removeItem("access_token");
        localStorage.removeItem("refresh_token");
        clearAuth();
        navigate("/login");
    };

    return (
        <header className="sticky top-0 z-50 h-14 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 flex items-center px-4 gap-3">

            {/* logo */}
            <Link to="/" className="flex items-center gap-2 flex-shrink-0">
                <GitBranch className="w-6 h-6 text-blue-600" />
                <span className="font-bold text-gray-900 dark:text-white hidden sm:block">
          {APP_NAME}
        </span>
            </Link>

            {/* search */}
            <div ref={searchRef} className="flex-1 max-w-lg relative">
                <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
                    <input
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        onFocus={() => setShowSearch(true)}
                        placeholder="Search or jump to..."
                        className="w-full pl-9 pr-3 py-1.5 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    />
                </div>

                {showSearch && results && (
                    <div className="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl overflow-hidden">
                        {results.repositories?.length > 0 && (
                            <div>
                                <div className="px-3 py-1.5 text-xs font-semibold text-gray-500 bg-gray-50 dark:bg-gray-800">
                                    Repositories
                                </div>
                                {results.repositories.slice(0, 3).map((r: any) => (
                                    <button
                                        key={r.id}
                                        onClick={() => { navigate(`/${r.owner}/${r.name}`); setShowSearch(false); setQuery(""); }}
                                        className="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-gray-50 dark:hover:bg-gray-800 text-left"
                                    >
                                        <BookOpen className="w-4 h-4 text-gray-400 flex-shrink-0" />
                                        <span className="text-gray-900 dark:text-gray-100">
                      {r.owner}/{r.name}
                    </span>
                                    </button>
                                ))}
                            </div>
                        )}
                        {results.users?.length > 0 && (
                            <div>
                                <div className="px-3 py-1.5 text-xs font-semibold text-gray-500 bg-gray-50 dark:bg-gray-800">
                                    Users
                                </div>
                                {results.users.slice(0, 3).map((u: any) => (
                                    <button
                                        key={u.id}
                                        onClick={() => { navigate(`/${u.username}`); setShowSearch(false); setQuery(""); }}
                                        className="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-gray-50 dark:hover:bg-gray-800 text-left"
                                    >
                                        <Avatar username={u.username} size="xs" />
                                        <span className="text-gray-900 dark:text-gray-100">{u.username}</span>
                                    </button>
                                ))}
                            </div>
                        )}
                    </div>
                )}
            </div>

            <div className="flex items-center gap-1 ml-auto">
                {/* theme toggle */}
                <button
                    onClick={toggleTheme}
                    className="p-2 rounded-md text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800 dark:text-gray-400"
                >
                    {theme === "dark" ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
                </button>

                {user ? (
                    <>
                        {/* new */}
                        <div className="relative">
                            <button
                                onClick={() => setShowNew(!showNew)}
                                className="p-2 rounded-md text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800 dark:text-gray-400 flex items-center gap-0.5"
                            >
                                <Plus className="w-4 h-4" />
                                <ChevronDown className="w-3 h-3" />
                            </button>
                            {showNew && (
                                <div className="absolute right-0 top-full mt-1 w-44 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl py-1 text-sm">
                                    <button onClick={() => { navigate("/new"); setShowNew(false); }}
                                            className="w-full px-4 py-2 text-left hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300">
                                        New repository
                                    </button>
                                    <button onClick={() => { navigate("/organizations/new"); setShowNew(false); }}
                                            className="w-full px-4 py-2 text-left hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300">
                                        New organization
                                    </button>
                                </div>
                            )}
                        </div>

                        {/* notifications */}
                        <Link to="/notifications"
                              className="p-2 rounded-md text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800 dark:text-gray-400">
                            <Bell className="w-4 h-4" />
                        </Link>

                        {/* user menu */}
                        <div className="relative">
                            <button
                                onClick={() => setShowUser(!showUser)}
                                className="flex items-center gap-1 p-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800"
                            >
                                <Avatar src={user.avatar_url} username={user.username} size="sm" />
                                <ChevronDown className="w-3 h-3 text-gray-400" />
                            </button>
                            {showUser && (
                                <div className="absolute right-0 top-full mt-1 w-52 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl py-1 text-sm">
                                    <div className="px-4 py-2 border-b border-gray-100 dark:border-gray-800">
                                        <p className="font-semibold text-gray-900 dark:text-white">{user.username}</p>
                                        {user.display_name && (
                                            <p className="text-xs text-gray-500">{user.display_name}</p>
                                        )}
                                    </div>
                                    <button onClick={() => { navigate(`/${user.username}`); setShowUser(false); }}
                                            className="w-full flex items-center gap-2 px-4 py-2 text-left hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300">
                                        <User className="w-4 h-4" /> Your profile
                                    </button>
                                    <button onClick={() => { navigate("/settings"); setShowUser(false); }}
                                            className="w-full flex items-center gap-2 px-4 py-2 text-left hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300">
                                        <Settings className="w-4 h-4" /> Settings
                                    </button>
                                    <div className="border-t border-gray-100 dark:border-gray-800 mt-1 pt-1">
                                        <button onClick={logout}
                                                className="w-full flex items-center gap-2 px-4 py-2 text-left hover:bg-gray-50 dark:hover:bg-gray-800 text-red-600 dark:text-red-400">
                                            <LogOut className="w-4 h-4" /> Sign out
                                        </button>
                                    </div>
                                </div>
                            )}
                        </div>
                    </>
                ) : (
                    <div className="flex items-center gap-2">
                        <Button variant="ghost" size="sm" onClick={() => navigate("/login")}>
                            Sign in
                        </Button>
                        <Button size="sm" onClick={() => navigate("/register")}>
                            Sign up
                        </Button>
                    </div>
                )}
            </div>
        </header>
    );
}