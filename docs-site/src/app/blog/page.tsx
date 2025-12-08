export default function BlogPage() {
    return (
        <main className="min-h-screen">
            {/* Hero Section */}
            <div className="w-full h-64 md:h-96 relative bg-gradient-to-r from-amber-900/20 to-neutral-900 border-b border-fd-border">
                {/* Placeholder for Generated Image since API quota exceeded */}
                <div className="absolute inset-0 flex items-center justify-center">
                    <span className="text-fd-muted font-mono text-xs md:text-sm uppercase tracking-widest">[Abstract Industrial ECS Banner]</span>
                </div>
                <div className="absolute inset-0 bg-gradient-to-t from-background to-transparent opacity-80"></div>
                <div className="absolute bottom-6 left-0 right-0 md:bottom-12">
                    <div className="container max-w-4xl mx-auto px-6">
                        <h1 className="text-4xl md:text-5xl font-extrabold tracking-tight text-white mb-4">
                            Industrial ECS:<br />
                            <span className="text-amber-500">Beyond Gaming</span>
                        </h1>
                        <p className="text-white/80 text-lg">Why we're bringing Entity Component Systems to the factory floor.</p>
                    </div>
                </div>
            </div>

            {/* Article Content */}
            <div className="container max-w-4xl mx-auto px-6 py-12">
                <article className="prose prose-neutral dark:prose-invert max-w-none">
                    <p className="lead text-xl text-fd-muted-foreground mb-8">
                        Bevy Eventwork wasn't just built for games. It originated from a need to synchronize complex, stateful systems in industrial environments—places where reliability, realtime performance, and clear data flow are paramount.
                    </p>

                    <h2>The Experiment: Synchronizing Realtime Apps</h2>
                    <p>
                        Most industrial control software relies on archaic loop architectures or bloated object-oriented hierarchies. We asked a different question:
                        <strong> What if we treated a factory line like a game level?</strong>
                    </p>
                    <p>
                        By leveraging the <strong>Entity Component System (ECS)</strong> architecture, specifically Bevy ECS, we decouple data (Components) from behavior (Systems). This allows us to build:
                    </p>
                    <ul>
                        <li><strong>Reactive Dashboards</strong>: Web apps (via Leptos) that reflect the exact state of the machine in real-time.</li>
                        <li><strong>Predictable State</strong>: Every event is strictly typed and processed in a deterministic order.</li>
                        <li><strong>Transport Independence</strong>: Whether over local TCP cables or WebSockets to a cloud dashboard, the logic remains the same.</li>
                    </ul>

                    <h2>Why Eventwork?</h2>
                    <p>
                        Standard web frameworks struggle with high-frequency state synchronization. Game networking libraries are often too tightly coupled to game-specific concepts (lobbies, matchmaking).
                    </p>
                    <p>
                        <strong>bevy_eventwork</strong> sits in the middle. It provides the <strong>transport-agnostic</strong> plumbing to send strongly-typed Rust structs as messages, and the <strong>sync tools</strong> to automatically replicate ECS components to connected clients.
                    </p>

                    <blockquote>
                        "This may not be the best architecture for all applications—simple CRUD apps don't need an ECS. But for applications that require complex, realtime state management, we believe this is the future."
                    </blockquote>

                    <h2>The Road Ahead</h2>
                    <p>
                        We are committed to making this path easier. With new tools like the <strong>World Inspector</strong> and direct integration with <strong>Leptos</strong> for web clients, we are bridging the gap between high-performance Rust backends and modern, reactive frontends.
                    </p>
                </article>
            </div>
        </main>
    );
}
