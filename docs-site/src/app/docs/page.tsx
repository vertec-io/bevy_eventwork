import Link from 'next/link';
import { Network, RefreshCw, Smartphone, ArrowRight, Terminal } from 'lucide-react'; // Added Terminal for CLI metaphor if needed
// Card imports removed

export default function DocsPortalPage() {
    return (
        <div className="min-h-screen flex flex-col items-center justify-center py-20 px-4 relative">
            {/* Background Glow */}
            <div className="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-amber-500/5 rounded-full blur-[100px] -z-10 pointer-events-none"></div>

            <div className="max-w-4xl w-full space-y-12">
                <div className="text-center space-y-4">
                    <h1 className="text-4xl font-extrabold tracking-tight lg:text-5xl">Getting Started</h1>
                    <p className="text-xl text-fd-muted-foreground">Select a module to browse documentation.</p>
                </div>

                <div className="grid gap-6 md:grid-cols-2">
                    {/* Core */}
                    <Link href="/docs/core" className="group">
                        <div className="h-full border border-fd-border bg-fd-card p-6 rounded-xl transition-all hover:border-amber-500/50 hover:shadow-[0_0_30px_-5px_rgba(245,158,11,0.15)]">
                            <div className="mb-4 inline-flex p-3 rounded-lg bg-fd-primary/10 text-fd-primary">
                                <Network className="w-6 h-6" />
                            </div>
                            <h2 className="text-2xl font-bold mb-2">Core</h2>
                            <p className="text-fd-muted-foreground mb-4">
                                The foundation of bevy_eventwork. Learn about transports, message handling, and general networking architecture.
                            </p>
                            <div className="text-sm font-medium text-fd-primary flex items-center opacity-0 group-hover:opacity-100 transition-opacity translate-x-[-10px] group-hover:translate-x-0">
                                Explore <ArrowRight className="ml-1 w-4 h-4" />
                            </div>
                        </div>
                    </Link>

                    {/* Sync */}
                    <Link href="/docs/sync" className="group">
                        <div className="h-full border border-fd-border bg-fd-card p-6 rounded-xl transition-all hover:border-amber-500/50 hover:shadow-[0_0_30px_-5px_rgba(245,158,11,0.15)]">
                            <div className="mb-4 inline-flex p-3 rounded-lg bg-fd-primary/10 text-fd-primary">
                                <RefreshCw className="w-6 h-6" />
                            </div>
                            <h2 className="text-2xl font-bold mb-2">Sync</h2>
                            <p className="text-fd-muted-foreground mb-4">
                                Automatic component synchronization. Replicate formatting and logic across the network effortlessly.
                            </p>
                            <div className="text-sm font-medium text-fd-primary flex items-center opacity-0 group-hover:opacity-100 transition-opacity translate-x-[-10px] group-hover:translate-x-0">
                                Explore <ArrowRight className="ml-1 w-4 h-4" />
                            </div>
                        </div>
                    </Link>

                    {/* Client */}
                    <Link href="/docs/client" className="group">
                        <div className="h-full border border-fd-border bg-fd-card p-6 rounded-xl transition-all hover:border-amber-500/50 hover:shadow-[0_0_30px_-5px_rgba(245,158,11,0.15)]">
                            <div className="mb-4 inline-flex p-3 rounded-lg bg-fd-primary/10 text-fd-primary">
                                <Smartphone className="w-6 h-6" />
                            </div>
                            <h2 className="text-2xl font-bold mb-2">Client</h2>
                            <p className="text-fd-muted-foreground mb-4">
                                Leptos-based client library for building reactive web frontends for your Bevy games.
                            </p>
                            <div className="text-sm font-medium text-fd-primary flex items-center opacity-0 group-hover:opacity-100 transition-opacity translate-x-[-10px] group-hover:translate-x-0">
                                Explore <ArrowRight className="ml-1 w-4 h-4" />
                            </div>
                        </div>
                    </Link>

                    {/* Quick Link to Intro */}
                    <Link href="/docs/core/getting-started" className="group">
                        <div className="h-full border border-fd-border bg-fd-card p-6 rounded-xl transition-all hover:border-amber-500/50 hover:shadow-[0_0_30px_-5px_rgba(245,158,11,0.15)] flex flex-col justify-center items-center text-center">
                            <h2 className="text-xl font-bold mb-2">New to Eventwork?</h2>
                            <p className="text-fd-muted-foreground mb-2">
                                Start here for the basics.
                            </p>
                            <div className="text-sm font-medium text-fd-primary">
                                Read Introduction
                            </div>
                        </div>
                    </Link>

                </div>
            </div>
        </div>
    );
}
