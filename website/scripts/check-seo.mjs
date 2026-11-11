import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const dist = fileURLToPath(new URL('../dist/', import.meta.url));
const files = [];
function collect(folder) {
  for (const entry of readdirSync(folder, { withFileTypes: true })) {
    const path = join(folder, entry.name);
    if (entry.isDirectory()) collect(path);
    else if (entry.name === 'index.html') files.push(path);
  }
}
collect(join(dist, 'docs'));
const errors = [];
const descriptions = new Map();
for (const file of files) {
  const html = readFileSync(file, 'utf8');
  const route = '/' + file.slice(dist.length).replaceAll('\\', '/').replace(/index\.html$/, '');
  const h1s = [...html.matchAll(/<h1(?:\s|>)/g)].length;
  if (h1s !== 1) errors.push(`${route}: expected one H1, got ${h1s}`);
  const meta = /<meta name="description" content="([^"]+)"/.exec(html);
  if (!meta || !meta[1]) errors.push(`${route}: missing meta description`);
  else {
    if (descriptions.has(meta[1])) errors.push(`${route}: repeated description with ${descriptions.get(meta[1])}`);
    descriptions.set(meta[1], route);
  }
  for (const [, href] of html.matchAll(/href="([^"]+)"/g)) {
    if (/^(https?:|mailto:|data:|#)/i.test(href)) continue;
    const pathname = href.split(/[?#]/)[0];
    if (!pathname) continue;
    const target = pathname.startsWith('/') ? resolve(dist, '.' + pathname) : resolve(dirname(file), pathname);
    if (/\.md$/i.test(pathname)) errors.push(`${route}: source Markdown link ${href}`);
    else if (!existsSync(target) && !existsSync(join(target, 'index.html')))
      errors.push(`${route}: missing local target ${href}`);
  }
}
if (errors.length) {
  console.error(errors.slice(0, 50).join('\n'));
  console.error(`${errors.length} verification failures`);
  process.exitCode = 1;
} else console.log(`Verified ${files.length} docs pages: links, H1, unique descriptions.`);
