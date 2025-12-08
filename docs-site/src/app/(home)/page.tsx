import Link from 'next/link';
// Button import removed
import { Github, Book } from 'lucide-react';

export default function HomePage() {
  return (
    <main className="flex flex-col items-center justify-center min-h-[calc(100vh-4rem)] text-center px-4 overflow-hidden relative pb-20">
      <div className="absolute inset-0 -z-10 h-full w-full bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:14px_24px]"></div>

      <div className="absolute top-0 z-[-2] h-screen w-screen bg-background bg-[radial-gradient(ellipse_80%_80%_at_50%_-20%,rgba(234,179,8,0.3),rgba(255,255,255,0))]" />

      <h1 className="text-5xl font-extrabold tracking-tight sm:text-7xl mb-6 max-w-5xl mx-auto pt-20">
        Build ECS driven<br />
        <span className="text-transparent bg-clip-text bg-gradient-to-r from-yellow-400 to-amber-600">
          realtime apps fast.
        </span>
      </h1>

      <p className="max-w-3xl mx-auto text-lg text-fd-muted-foreground mb-10 leading-relaxed">
        **bevy_eventwork** is built for the Bevy ECS. While powerful for games, it is designed to empower **industrial applications** and **web apps** with a stateful, event-driven architecture.
      </p>

      <div className="flex flex-col sm:flex-row gap-4 items-center justify-center mb-24">
        <Link href="/docs">
          <button className="inline-flex items-center justify-center h-12 px-8 text-sm font-semibold transition-all rounded-full bg-fd-primary text-fd-primary-foreground hover:bg-fd-primary/90 hover:scale-105 active:scale-95 shadow-lg shadow-amber-500/20">
            <Book className="w-4 h-4 mr-2" />
            Documentation
          </button>
        </Link>
        <Link href="https://github.com/jamescarterbell/bevy_eventwork" target="_blank">
          <button className="inline-flex items-center justify-center h-12 px-8 text-sm font-semibold transition-all rounded-full border border-fd-border bg-fd-background hover:bg-fd-accent hover:text-fd-accent-foreground">
            <Github className="w-4 h-4 mr-2" />
            GitHub
          </button>
        </Link>
      </div>

      <div className="w-full max-w-6xl mx-auto space-y-24">
        {/* Feature Grid */}
        <div className="grid md:grid-cols-3 gap-8 text-left p-8 rounded-xl border border-fd-border/50 bg-fd-card/50 backdrop-blur-sm shadow-2xl relative overflow-hidden">
          <div className="absolute -top-12 -left-12 w-64 h-64 bg-amber-500/10 rounded-full blur-3xl -z-10"></div>
          <div className="absolute -bottom-12 -right-12 w-64 h-64 bg-amber-500/10 rounded-full blur-3xl -z-10"></div>

          <Feature
            title="Industrial Grade"
            desc="Reliable, low-latency messaging for complex industrial systems and control software. Not just for games."
          />
          <Feature
            title="Transport Agnostic"
            desc="Swap between TCP, WebSocket, or Rtc transports easily to fit your deployment environment."
          />
          <Feature
            title="Stateful ECS"
            desc="Leverage Bevy's Entity Component System to manage complex application state with ease."
          />
        </div>

        {/* Web Apps Section */}
        <div className="flex flex-col md:flex-row items-center gap-12 text-left">
          <div className="flex-1 space-y-6">
            <h2 className="text-3xl font-bold">Web Apps with Leptos</h2>
            <p className="text-fd-muted-foreground text-lg">
              Build reactive, high-performance web frontends using **Leptos**, backed by a powerful ECS engine. Sync components automatically between your server and web client, creating a seamless realtime experience.
            </p>
            <ul className="space-y-2 text-fd-muted-foreground">
              <li className="flex items-center"><span className="w-2 h-2 bg-amber-500 rounded-full mr-3"></span>Shared Rust Types</li>
              <li className="flex items-center"><span className="w-2 h-2 bg-amber-500 rounded-full mr-3"></span>WASM Compilation</li>
              <li className="flex items-center"><span className="w-2 h-2 bg-amber-500 rounded-full mr-3"></span>Reactive UI Updates</li>
            </ul>
          </div>
          <div className="flex-1 h-64 w-full bg-gradient-to-br from-neutral-900 to-neutral-800 rounded-xl border border-fd-border flex items-center justify-center">
            <span className="text-fd-muted font-mono">Leptos Integration Demo</span>
          </div>
        </div>

        {/* World Inspector Section */}
        <div className="flex flex-col md:flex-row-reverse items-center gap-12 text-left">
          <div className="flex-1 space-y-6">
            <h2 className="text-3xl font-bold">World Inspector Dev Tools</h2>
            <p className="text-fd-muted-foreground text-lg">
              Visualize and debug your application state in real-time. The World Inspector system allows you to query entities, view component data, and monitor network events as they happen.
            </p>
            <ul className="space-y-2 text-fd-muted-foreground">
              <li className="flex items-center"><span className="w-2 h-2 bg-amber-500 rounded-full mr-3"></span>Realtime State Visualization</li>
              <li className="flex items-center"><span className="w-2 h-2 bg-amber-500 rounded-full mr-3"></span>Entity Querying</li>
              <li className="flex items-center"><span className="w-2 h-2 bg-amber-500 rounded-full mr-3"></span>Network Traffic Monitoring</li>
            </ul>
          </div>
          <div className="flex-1 h-64 w-full bg-gradient-to-bl from-neutral-900 to-neutral-800 rounded-xl border border-fd-border flex items-center justify-center">
            <span className="text-fd-muted font-mono">Inspector UI Placeholder</span>
          </div>
        </div>
      </div>
    </main>
  );
}

function Feature({ title, desc }: { title: string; desc: string }) {
  return (
    <div className="space-y-2">
      <h3 className="text-xl font-bold text-fd-foreground">{title}</h3>
      <p className="text-sm text-fd-muted-foreground">{desc}</p>
    </div>
  )
}
