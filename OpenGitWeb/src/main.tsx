import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { ToastProvider } from "./components/ui/Toast";
import { ErrorBoundary } from "./components/ErrorBoundary";
import App from "./App";
import "./index.css";

const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            staleTime:            1000 * 60 * 5,
            retry:                1,
            refetchOnWindowFocus: false,
        },
    },
});

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <ErrorBoundary>
            <QueryClientProvider client={queryClient}>
                <BrowserRouter>
                    <ToastProvider>
                        <App />
                    </ToastProvider>
                </BrowserRouter>
                <ReactQueryDevtools initialIsOpen={false} />
            </QueryClientProvider>
        </ErrorBoundary>
    </React.StrictMode>
);