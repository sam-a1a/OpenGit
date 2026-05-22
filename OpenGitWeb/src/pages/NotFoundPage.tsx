import React from "react";
import { Link, useNavigate } from "react-router-dom";
import { GitBranch, ArrowLeft, Search } from "lucide-react";
import { Button } from "../components/ui/Button";
import { APP_NAME } from "../lib/constants";

export default function NotFoundPage() {
    const navigate = useNavigate();

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950 flex flex-col">
            <nav className="px-6 py-4 border-b border-gray-100 dark:border-gray-900">
                <Link to="/" className="flex items-center gap-2 w-fit">
                    <div className="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center">
                        <GitBranch className="w-5 h-5 text-white" />
                    </div>
                    <span className="font-bold text-gray-900 dark:text-white">{APP_NAME}</span>
                </Link>
            </nav>

            <div className="flex-1 flex items-center justify-center px-4">
                <div className="text-center max-w-lg">
                    <div className="mb-8 relative">
                        <div className="text-[10rem] font-black text-gray-100 dark:text-gray-800 leading-none select-none">
                            404
                        </div>
                        <div className="absolute inset-0 flex items-center justify-center">
                            <div className="w-20 h-20 bg-blue-600 rounded-2xl flex items-center justify-center shadow-2xl">
                                <GitBranch className="w-10 h-10 text-white" />
                            </div>
                        </div>
                    </div>

                    <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-3">
                        This page doesn't exist
                    </h1>
                    <p className="text-gray-500 dark:text-gray-400 mb-8 text-lg">
                        The page you're looking for has been moved, deleted, or never existed.
                    </p>

                    <div className="flex items-center justify-center gap-3 flex-wrap">
                        <Button
                            variant="ghost"
                            icon={<ArrowLeft className="w-4 h-4" />}
                            onClick={() => navigate(-1)}
                        >
                            Go back
                        </Button>
                        <Link to="/">
                            <Button>Go home</Button>
                        </Link>
                        <Link to="/explore">
                            <Button variant="outline" icon={<Search className="w-4 h-4" />}>
                                Explore
                            </Button>
                        </Link>
                    </div>
                </div>
            </div>
        </div>
    );
}