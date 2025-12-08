'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { ChevronDown, Check } from 'lucide-react';
import { useState, useRef, useEffect } from 'react';
import { cn } from '@/lib/utils'; // Assuming standard utils, or no utils. Fumadocs might export 'cn' from somewhere? I'll implement simple clsx/twMerge if needed or just use template literals.
// I'll assume basic className usage for now.

const MODES = [
    { title: 'Core', url: '/docs/core', description: 'Networking & Transports' },
    { title: 'Sync', url: '/docs/sync', description: 'Component Sync' },
    { title: 'Client', url: '/docs/client', description: 'Leptos Client' },
];

export function RootToggle() {
    const pathname = usePathname();
    const [open, setOpen] = useState(false);
    const containerRef = useRef<HTMLDivElement>(null);

    const currentMode = MODES.find((mode) => pathname?.startsWith(mode.url)) || MODES[0];

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
                setOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    return (
        <div ref={containerRef} className="relative w-full">
            <button
                onClick={() => setOpen(!open)}
                className="flex items-center justify-between w-full px-3 py-2 text-sm font-medium transition-colors rounded-md hover:bg-fd-accent hover:text-fd-accent-foreground text-fd-foreground"
            >
                <span className="flex flex-col items-start text-left">
                    <span className="text-xs text-fd-muted-foreground font-normal">Module</span>
                    <span className="font-semibold">{currentMode.title}</span>
                </span>
                <ChevronDown className="w-4 h-4 ml-2 opacity-50" />
            </button>

            {open && (
                <div className="absolute top-full left-0 z-50 w-full mt-1 overflow-hidden border rounded-md shadow-md bg-fd-popover border-fd-border animate-in fade-in-0 zoom-in-95">
                    <div className="p-1">
                        {MODES.map((mode) => (
                            <Link
                                key={mode.url}
                                href={mode.url}
                                onClick={() => setOpen(false)}
                                className="flex items-center justify-between px-3 py-2 text-sm rounded-sm hover:bg-fd-accent hover:text-fd-accent-foreground"
                            >
                                <div className="flex flex-col">
                                    <span className="font-medium">{mode.title}</span>
                                    <span className="text-xs text-fd-muted-foreground">{mode.description}</span>
                                </div>
                                {/* Checkmark if active? */}
                                {pathname?.startsWith(mode.url) && <Check className="w-4 h-4 ml-2 text-fd-primary" />}
                            </Link>
                        ))}
                        <div className="h-px my-1 bg-fd-border" />
                        <Link
                            href="/docs"
                            className="block px-3 py-2 text-xs text-center text-fd-muted-foreground hover:bg-fd-accent hover:text-fd-accent-foreground"
                        >
                            View all modules
                        </Link>
                    </div>
                </div>
            )}
        </div>
    );
}
