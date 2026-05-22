import React from "react";
import { Link } from "react-router-dom";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "./ui/Button";

interface Props {
    children: React.ReactNode;
}

interface State {
    hasError: boolean;
    error:    Error | null;
}

export class ErrorBoundary extends React.Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = { hasError: false, error: null };
    }

    static getDerivedStateFromError(error: Error): State {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, info: React.ErrorInfo) {
        console.error("ErrorBoundary caught:", error, info);
    }

    render() {
        if (this.state.hasError) {
            return (
                <div className="min-h-screen bg-gray-50 dark:bg-gray-950 flex items-center justify-center px-4">
                    <div className="text-center max-w-md">
                        <div className="w-16 h-16 bg-red-100 dark:bg-red-900/30 rounded-2xl flex items-center justify-center mx-auto mb-6">
                            <AlertTriangle className="w-8 h-8 text-red-500" />
                        </div>
                        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-3">
                            Something went wrong
                        </h1>
                        <p className="text-gray-500 mb-2 text-sm">
                            An unexpected error occurred. This has been logged.
                        </p>
                        {this.state.error && (
                            <div className="bg-gray-100 dark:bg-gray-800 rounded-lg p-3 mb-6 text-left">
                                <code className="text-xs text-red-500 dark:text-red-400 font-mono break-all">
                                    {this.state.error.message}
                                </code>
                            </div>
                        )}
                        <div className="flex items-center justify-center gap-3">
                            <Button
                                variant="outline"
                                icon={<RefreshCw className="w-4 h-4" />}
                                onClick={() => this.setState({ hasError: false, error: null })}
                            >
                                Try again
                            </Button>
                            <Link to="/">
                                <Button>Go home</Button>
                            </Link>
                        </div>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}