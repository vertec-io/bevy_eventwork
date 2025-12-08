import { source } from '@/lib/source';
import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { baseOptions } from '@/lib/layout.shared';
import { RootToggle } from '@/components/root-toggle';
import { ReactNode } from 'react';

export default async function Layout({ children, params }: { children: ReactNode; params: Promise<{ slug: string[] }> }) {
  const { slug } = await params;
  const rootSlug = slug?.[0];
  // Find the root node (folder) that matches the first slug segment (core, sync, client)
  const rootNode = source.pageTree.children.find(
    (node) => node.type === 'folder' && node.name === rootSlug
  );

  // If found, use that folder's children as the tree. Otherwise fallback to full tree (or handle 404/redirect logic elsewhere)
  // Actually, standard docs layout expects a Root object.
  // We can wrap the children in a root object structure if needed, or if rootNode IS a root-compatible node.
  // DocsLayout tree prop expects a PageTree.Root.
  // A 'folder' node has { type: 'folder', name: string, url: string, index: page, children: [...] }
  // A 'root' node has { type: 'root', name: string, children: [...] }
  // So we construct a synthetic root.

  /*
  const tree = rootNode
    ? {
      name: rootNode.name,
      children: (rootNode as any).children
    }
    : source.pageTree;
  */
  const tree = source.pageTree;

  return (
    <DocsLayout
      tree={tree}
      {...baseOptions()}
      sidebar={{
        banner: <RootToggle />
      }}
    >
      {children}
    </DocsLayout>
  );
}
