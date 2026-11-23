import { cp, mkdir, readdir, rm } from 'node:fs/promises';

const client = new URL('../client/', import.meta.url);
const dist = new URL('../dist/', import.meta.url);
await rm(dist, { recursive: true, force: true });
await mkdir(dist);
for (const arquivo of await readdir(client)) {
    if (arquivo === 'assets' || arquivo === 'dist' || /\.(html|css)$/.test(arquivo)) {
        await cp(new URL(arquivo, client), new URL(arquivo, dist), { recursive: true });
    }
}
