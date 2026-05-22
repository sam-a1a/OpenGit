import { Link } from "react-router-dom";
import { GitBranch, Shield, Zap, Globe, GitPullRequest, CircleDot, Play, Star } from "lucide-react";
import { Button } from "../components/ui/Button";
import { useAuthStore } from "../stores/auth";
import { APP_NAME } from "../lib/constants";

const features = [
    { icon: <GitBranch className="w-6 h-6" />,       title: "Git hosting",        desc: "Full Git over HTTP and SSH. Clone, push, pull — everything you need." },
    { icon: <GitPullRequest className="w-6 h-6" />,  title: "Pull requests",      desc: "Code review with inline comments, approvals, and merge strategies." },
    { icon: <CircleDot className="w-6 h-6" />,       title: "Issues",             desc: "Bug tracking with labels, milestones, assignees, and reactions." },
    { icon: <Play className="w-6 h-6" />,            title: "CI/CD",              desc: "Built-in workflows with self-hosted runners and artifact storage." },
    { icon: <Shield className="w-6 h-6" />,          title: "Security",           desc: "2FA, SSH keys, secret scanning, and OAuth 2.0 provider built in." },
    { icon: <Globe className="w-6 h-6" />,           title: "Self-hosted",        desc: "Your data, your server. Single Docker Compose command to run." },
];

export default function LandingPage() {
    const user = useAuthStore((s) => s.user);

    return (
        <div className="min-h-screen bg-white dark:bg-gray-950">
            {/* nav */}
            <nav className="border-b border-gray-100 dark:border-gray-900 px-6 py-4 flex items-center justify-between max-w-7xl mx-auto">
                <div className="flex items-center gap-2">
                    <div className="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center">
                        <GitBranch className="w-5 h-5 text-white" />
                    </div>
                    <span className="font-bold text-gray-900 dark:text-white text-lg">{APP_NAME}</span>
                </div>
                <div className="flex items-center gap-3">
                    <Link to="/explore"
                          className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white font-medium">
                        Explore
                    </Link>
                    {user ? (
                        <Link to="/explore">
                            <Button size="sm">Dashboard</Button>
                        </Link>
                    ) : (
                        <>
                            <Link to="/login">
                                <Button variant="ghost" size="sm">Sign in</Button>
                            </Link>
                            <Link to="/register">
                                <Button size="sm">Get started</Button>
                            </Link>
                        </>
                    )}
                </div>
            </nav>

            {/* hero */}
            <section className="max-w-5xl mx-auto px-6 pt-20 pb-16 text-center">
                <div className="inline-flex items-center gap-2 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 px-4 py-1.5 rounded-full text-sm font-medium mb-6">
                    <Star className="w-4 h-4" />
                    Open source · Self-hosted
                </div>
                <h1 className="text-5xl sm:text-6xl font-bold text-gray-900 dark:text-white mb-6 leading-tight">
                    Your own Git platform,
                    <span className="text-blue-600 dark:text-blue-400"> without compromise</span>
                </h1>
                <p className="text-xl text-gray-500 dark:text-gray-400 mb-10 max-w-2xl mx-auto">
                    {APP_NAME} is a fully-featured, self-hosted Git platform.
                    Everything GitHub offers — issues, PRs, CI/CD, packages — on your own infrastructure.
                </p>
                <div className="flex items-center justify-center gap-4 flex-wrap">
                    <Link to="/register">
                        <Button size="lg" icon={<GitBranch className="w-5 h-5" />}>
                            Get started for free
                        </Button>
                    </Link>
                    <Link to="/explore">
                        <Button variant="outline" size="lg">
                            Explore repos
                        </Button>
                    </Link>
                </div>

                {/* terminal preview */}
                <div className="mt-14 bg-gray-950 rounded-2xl p-6 text-left max-w-lg mx-auto shadow-2xl">
                    <div className="flex items-center gap-2 mb-4">
                        <div className="w-3 h-3 rounded-full bg-red-500" />
                        <div className="w-3 h-3 rounded-full bg-yellow-500" />
                        <div className="w-3 h-3 rounded-full bg-green-500" />
                    </div>
                    <div className="font-mono text-sm space-y-1.5 text-gray-300">
                        <p><span className="text-green-400">$</span> docker compose up -d</p>
                        <p className="text-gray-500">Starting opengit_postgres ... done</p>
                        <p className="text-gray-500">Starting opengit_valkey  ... done</p>
                        <p className="text-gray-500">Starting opengit_minio   ... done</p>
                        <p className="text-gray-500">Starting opengit_backend ... done</p>
                        <p className="mt-2"><span className="text-green-400">$</span> git clone git@opengit.io:sam/myproject.git</p>
                        <p className="text-gray-500">Cloning into 'myproject'...</p>
                        <p className="text-gray-500">remote: Enumerating objects: 142</p>
                        <p><span className="text-blue-400">✓</span> <span className="text-green-300">Done.</span></p>
                    </div>
                </div>
            </section>

            {/* features */}
            <section className="max-w-6xl mx-auto px-6 py-16">
                <h2 className="text-3xl font-bold text-center text-gray-900 dark:text-white mb-3">
                    Everything you need
                </h2>
                <p className="text-center text-gray-500 mb-12">
                    No feature behind a paywall. No vendor lock-in.
                </p>
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
                    {features.map((f) => (
                        <div key={f.title}
                             className="bg-gray-50 dark:bg-gray-900 rounded-xl p-6 border border-gray-100 dark:border-gray-800">
                            <div className="w-10 h-10 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-lg flex items-center justify-center mb-4">
                                {f.icon}
                            </div>
                            <h3 className="font-semibold text-gray-900 dark:text-white mb-2">{f.title}</h3>
                            <p className="text-sm text-gray-500 dark:text-gray-400">{f.desc}</p>
                        </div>
                    ))}
                </div>
            </section>

            {/* CTA */}
            <section className="max-w-4xl mx-auto px-6 py-20 text-center">
                <div className="bg-blue-600 rounded-3xl p-12 text-white">
                    <h2 className="text-3xl font-bold mb-4">Ready to host your code?</h2>
                    <p className="text-blue-100 mb-8 text-lg">
                        One command. Your data. Full control.
                    </p>
                    <div className="flex items-center justify-center gap-4 flex-wrap">
                        <Link to="/register">
                            <Button
                                className="bg-white text-blue-600 hover:bg-blue-50"
                                size="lg"
                            >
                                Create an account
                            </Button>
                        </Link>

                        href="https://github.com/sam-a1a/OpenGit"
                        target="_blank"
                        rel="noopener noreferrer"
                        >
                        <Button variant="outline"
                                className="border-white text-white hover:bg-blue-700"
                                size="lg"
                        >
                            View on GitHub
                        </Button>
                    </a>
                </div>
        </div>
</section>

    {/* footer */}
    <footer className="border-t border-gray-100 dark:border-gray-900 px-6 py-8">
        <div className="max-w-6xl mx-auto flex items-center justify-between flex-wrap gap-4 text-sm text-gray-500">
            <div className="flex items-center gap-2">
                <GitBranch className="w-4 h-4 text-blue-600" />
                <span className="font-semibold text-gray-900 dark:text-white">{APP_NAME}</span>
                <span>— Open source Git platform</span>
            </div>
            <div className="flex items-center gap-6">
                <Link to="/explore" className="hover:text-gray-900 dark:hover:text-white">Explore</Link>
                <a href="https://github.com/sam-a1a/OpenGit" target="_blank" rel="noopener noreferrer"
                   className="hover:text-gray-900 dark:hover:text-white">GitHub</a>
            </div>
        </div>
    </footer>
</div>
);
}