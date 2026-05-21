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
    username: z.string().min(3, "At least 3 characters")
        .max(39, "Max 39 characters")
        .regex(/^[a-zA-Z0-9-_]+$/, "Only letters, numbers, hyphens, underscores"),
    email:    z.string().email("Invalid email address"),
    password: z.string().min(8, "At least 8 characters"),
});

type FormData = z.infer<typeof schema>;

export default function RegisterPage() {
    const { register: registerUser } = useAuth();
    const navigate                   = useNavigate();
    const [error,  setError]         = useState("");
    const [showPw, setShowPw]        = useState(false);

    const { register, handleSubmit, formState: { errors, isSubmitting } } = useForm<FormData>({
        resolver: zodResolver(schema),
    });

    const onSubmit = async (data: FormData) => {
        setError("");
        try {
            await registerUser(data.username, data.email, data.password);
            navigate("/");
        } catch (e: any) {
            setError(e.response?.data?.error ?? "Registration failed");
        }
    };

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950 flex items-center justify-center p-4">
            <div className="w-full max-w-sm">
                <div className="flex flex-col items-center mb-8">
                    <div className="w-12 h-12 bg-blue-600 rounded-xl flex items-center justify-center mb-4">
                        <GitBranch className="w-7 h-7 text-white" />
                    </div>
                    <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
                        Join {APP_NAME}
                    </h1>
                    <p className="text-sm text-gray-500 mt-1">
                        Self-hosted Git platform
                    </p>
                </div>

                <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6 shadow-sm">
                    {error && <Alert type="error" className="mb-4">{error}</Alert>}

                    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
                        <Input
                            label="Username"
                            autoComplete="username"
                            error={errors.username?.message}
                            hint="Only letters, numbers, hyphens, underscores"
                            {...register("username")}
                        />
                        <Input
                            label="Email address"
                            type="email"
                            autoComplete="email"
                            error={errors.email?.message}
                            {...register("email")}
                        />
                        <div className="relative">
                            <Input
                                label="Password"
                                type={showPw ? "text" : "password"}
                                autoComplete="new-password"
                                error={errors.password?.message}
                                hint="At least 8 characters"
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

                        <Button type="submit" className="w-full" loading={isSubmitting}>
                            Create account
                        </Button>
                    </form>
                </div>

                <p className="mt-4 text-center text-sm text-gray-600 dark:text-gray-400">
                    Already have an account?{" "}
                    <Link to="/login" className="text-blue-600 hover:underline dark:text-blue-400 font-medium">
                        Sign in
                    </Link>
                </p>
            </div>
        </div>
    );
}