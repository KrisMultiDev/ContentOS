// Generates placeholder app icons (pine-green rounded square) without any
// image libraries — writes PNG chunks by hand. Rerun with: npm run icons
import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const OUT = join(dirname(fileURLToPath(import.meta.url)), "..", "src-tauri", "icons");
mkdirSync(OUT, { recursive: true });

const PINE = [0x2f, 0x6b, 0x4f];
const LEAF = [0x7c, 0xc0, 0x8a];

const crcTable = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) c = crcTable[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

function makePng(size) {
  const r = Math.round(size * 0.22); // corner radius
  const inset = Math.max(1, Math.round(size * 0.04));
  const stripeTop = Math.round(size * 0.62);
  const stripeBottom = Math.round(size * 0.74);

  const rows = [];
  for (let y = 0; y < size; y++) {
    const row = Buffer.alloc(1 + size * 4); // filter byte 0 + RGBA
    for (let x = 0; x < size; x++) {
      let alpha = 255;
      const lo = inset + r;
      const hi = size - 1 - inset - r;
      const cx = x < lo ? lo : x > hi ? hi : x;
      const cy = y < lo ? lo : y > hi ? hi : y;
      const dx = x - cx;
      const dy = y - cy;
      if (x < inset || y < inset || x >= size - inset || y >= size - inset) alpha = 0;
      else if (dx * dx + dy * dy > r * r) alpha = 0;
      const leaf = y >= stripeTop && y < stripeBottom;
      const [cr, cg, cb] = leaf ? LEAF : PINE;
      const o = 1 + x * 4;
      row[o] = cr;
      row[o + 1] = cg;
      row[o + 2] = cb;
      row[o + 3] = alpha;
    }
    rows.push(row);
  }

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // RGBA
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(Buffer.concat(rows), { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

function makeIco(png, size) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); // reserved
  header.writeUInt16LE(1, 2); // type: icon
  header.writeUInt16LE(1, 4); // count
  const entry = Buffer.alloc(16);
  entry[0] = size >= 256 ? 0 : size; // 0 means 256
  entry[1] = size >= 256 ? 0 : size;
  entry.writeUInt16LE(1, 4); // planes
  entry.writeUInt16LE(32, 6); // bpp
  entry.writeUInt32LE(png.length, 8);
  entry.writeUInt32LE(22, 12); // offset
  return Buffer.concat([header, entry, png]);
}

writeFileSync(join(OUT, "32x32.png"), makePng(32));
writeFileSync(join(OUT, "128x128.png"), makePng(128));
writeFileSync(join(OUT, "128x128@2x.png"), makePng(256));
writeFileSync(join(OUT, "icon.png"), makePng(512));
writeFileSync(join(OUT, "icon.ico"), makeIco(makePng(256), 256));
console.log("icons written to", OUT);
