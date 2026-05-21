import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { GitBranch, Eye, EyeOff } from "lucide-react";
import { Button } from "../../components/ui/Button";
import { Input } from "../../components/ui/Input";
import { Alert } from "../../components/ui/Alert";
import { useAuth } from "../../hooks/useAuth";
import { APP_NAME } from "../../lib/constants";

const schema = z.object({
    email:    z.string().email("Invalid email address"),
    password: z.string().min(1, "Password is required"),
});

type FormData = z.infer<typeof schema>;

export default function LoginPage() {
    const { login }           = useAuth();
    const navigate            = useNavigate();
    const [error,  setError]  = useState("");
    const [showPw, setShowPw] = useState(false);

    const { register, handleSubmit, formState: { errors, isSubmitting } } = useForm<FormData>({
        resolver: zodResolver(schema),
    });

    const onSubmit = async (data: FormData) => {
        setError("");
        try {
            const result = await login(data.email, data.password);
            if (result.twoFactorRequired) {
                navigate("/2fa", { state: { pendingToken: result.pendingToken } });
            } else {
                navigate("/");
            }
        } catch (e: any) {
            setError(e.response?.data?.error ?? "Invalid email or password");
        }
    };

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950 flex items-center justify-center p-4">
            <div className="w-full max-w-sm">

                {/* logo */}
                <div className="flex flex-col items-center mb-8">
                    <div className="w-12 h-12 bg-blue-600 rounded-xl flex items-center justify-center mb-4">
                        <GitBranch className="w-7 h-7 text-white" />
                    </div>
                    <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
                        Sign in to {APP_NAME}
                    </h1>
                </div>

                <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6 shadow-sm">
                    {error && (
                        <Alert type="error" className="mb-4">{error}</Alert>
                    )}

                    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
                        <Input
                            label="Email address"
                            type="email"
                            autoComplete="email"
                            error={errors.email?.message}
                            {...register("email")}
                        />

                        <div>
                            <div className="relative">
                                <Input
                                    label="Password"
                                    type={showPw ? "text" : "password"}
                                    autoComplete="current-password"
                                    error={errors.password?.message}
                                    {...register("password")}
                                />
                                <button
                                    type="button"
                                    onClick={() => setShowPw(!showPw)}
                                    className="absolute right-3 top-8 text-gray-400 hover:text-gray-600"
                                >
                                    {showPw ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                                </button>
                            </div>
                            <div className="flex justify-end mt-1">
                                <Link to="/forgot-password"
                                      className="text-xs text-blue-600 hover:underline dark:text-blue-400">
                                    Forgot password?
                                </Link>
                            </div>
                        </div>

                        <Button type="submit" className="w-full" loading={isSubmitting}>
                            Sign in
                        </Button>
                    </form>
                </div>

                <p className="mt-4 text-center text-sm text-gray-600 dark:text-gray-400">
                    Don't have an account?{" "}
                    <Link to="/register" className="text-blue-600 hover:underline dark:text-blue-400 font-medium">
                        Create one
                    </Link>
                </p>
            </div>
        </div>
    );
}