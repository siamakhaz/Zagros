import { readdirSync, existsSync, statSync } from 'node:fs';
import { resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const contentRoot = fileURLToPath(new URL('./src/content/docs/', import.meta.url));
const repoRoot = fileURLToPath(new URL('../', import.meta.url));
const srcDocs = resolve(repoRoot, 'docs');
const byName = new Map();
function walk(folder, prefix = '') {
  for (const entry of readdirSync(folder, { withFileTypes: true })) {
    const name = prefix ? prefix + '/' + entry.name : entry.name;
    if (entry.isDirectory()) walk(resolve(folder, entry.name), name);
    else if (entry.isFile() && /\.md$/i.test(entry.name)) {
      const route = '/docs/' + name.slice(0, -3).toLowerCase() + '/';
      const k = entry.name.toLowerCase();
      if (!byName.has(k)) byName.set(k, route);
    }
  }
}
walk(contentRoot);

export function rehypeZagrosLinks() {
  return (tree) => {
    function visit(node) {
      if (node.tagName === 'a' && node.properties && typeof node.properties.href === 'string') {
        const href = node.properties.href;
        if (!href.startsWith('/') && !/^[a-z][a-z\d+.-]*:/i.test(href) && !href.startsWith('#')) {
          const parts = /^([^?#]+\.md)([?#].*)?$/i.exec(href);
          if (parts) {
            const filename = parts[1].split('/').pop().toLowerCase();
            const sitePage = byName.get(filename);
            if (sitePage) node.properties.href = sitePage + (parts[2] ?? '');
            else {
              const absolute = resolve(srcDocs, parts[1]);
              const rel = relative(repoRoot, absolute).replaceAll('\\', '/');
              if (rel && !rel.startsWith('..') && !rel.startsWith('/') && existsSync(absolute) && statSync(absolute).isFile()) {
                node.properties.href = 'https://github.com/siamakhaz/Zagros/blob/main/' + rel.split('/').map(encodeURIComponent).join('/') + (parts[2] ?? '');
              }
            }
          }
        }
      }
      if (Array.isArray(node.children)) node.children.forEach(visit);
    }
    visit(tree);
  };
}
