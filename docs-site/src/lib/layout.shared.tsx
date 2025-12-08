import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: 'Eventwork',
      url: '/',
    },
    links: [
      {
        text: 'Documentation',
        url: '/docs',
        active: 'nested-url',
      },
      {
        text: 'Blog',
        url: '/blog',
        active: 'nested-url',
      },
      {
        text: 'Showcase',
        url: '/showcase', // User asked for it, I'll create a stub
        active: 'nested-url',
      },
      {
        text: 'Sponsors',
        url: 'https://www.vertec.io',
        external: true,
      },
    ],
    githubUrl: 'https://github.com/jamescarterbell/bevy_eventwork',
  };
}
