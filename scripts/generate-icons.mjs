import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = resolve(import.meta.dirname, '..');
const source = join(root, 'assets', 'branding', 'orbit-symbol-icon-large.png');
const output = join(root, 'src-tauri', 'icons');

mkdirSync(output, { recursive: true });
copyFileSync(source, join(root, 'public', 'favicon.png'));

const tauri = join(root, 'node_modules', '.bin', process.platform === 'win32' ? 'tauri.cmd' : 'tauri');
const run = (args) => spawnSync(tauri, args, {
  cwd: root,
  stdio: 'inherit',
  shell: process.platform === 'win32',
});
const result = run(['icon', source, '-o', output, '-p', '16,24,32,48,64,128,256']);
if (result.status !== 0) process.exit(result.status ?? 1);

// Tauri's generated ICO omits 128 px. Pack the canonical Windows set ourselves
// from the PNGs generated in the same step so the executable and installer have
// matching high-DPI entries on every production build.
const sizes = [16, 24, 32, 48, 64, 128, 256];
const pngs = sizes.map((size) => {
  const file = `${size}x${size}.png`;
  return readFileSync(join(output, file));
});
const header = Buffer.alloc(6 + sizes.length * 16);
header.writeUInt16LE(0, 0);
header.writeUInt16LE(1, 2);
header.writeUInt16LE(sizes.length, 4);
let offset = header.length;
for (let i = 0; i < sizes.length; i += 1) {
  const entry = 6 + i * 16;
  header[entry] = sizes[i] === 256 ? 0 : sizes[i];
  header[entry + 1] = sizes[i] === 256 ? 0 : sizes[i];
  header[entry + 2] = 0;
  header[entry + 3] = 0;
  header.writeUInt16LE(1, entry + 4);
  header.writeUInt16LE(32, entry + 6);
  header.writeUInt32LE(pngs[i].length, entry + 8);
  header.writeUInt32LE(offset, entry + 12);
  offset += pngs[i].length;
}
writeFileSync(join(output, 'icon.ico'), Buffer.concat([header, ...pngs]));
