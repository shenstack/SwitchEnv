/**
 * 为 Tauri 项目生成一套最小化的有效图标文件
 * Windows 构建必须提供 src-tauri/icons/icon.ico 才能生成 Windows Resource
 * 输出到 src-tauri/icons/ 目录
 *
 *  32x32.png      (普通小图标)
 *  128x128.png    (普通大图标)
 *  128x128@2x.png (高分屏图标, 256x256)
 *  icon.icns      (macOS 图标)
 *  icon.ico       (Windows 图标, 内嵌 16/32/48/256)
 *
 * 图标风格: 橙色圆角背景 + 白色大写字母 "E"
 *
 * @author ssr
 */

import fs from 'node:fs';
import path from 'node:path';
import sharp from 'sharp';

// 使用 process.cwd() 确保在项目根目录执行时路径正确
// (import.meta.url 在 Windows 某些 Node 版本上会解析出 "F:\F:\..." 的错误路径)
const projectRoot = process.cwd();
const outDir = path.join(projectRoot, 'src-tauri', 'icons');

// 确保输出目录存在
fs.mkdirSync(outDir, { recursive: true });

/**
 * 生成指定尺寸的 PNG 图标缓冲区
 * @param {number} size
 * @returns {Promise<Buffer>}
 */
async function makePng(size) {
  // 圆角半径 (越小越方)
  const radius = Math.round(size * 0.18);

  // 构造一个圆角矩形的 SVG: 橙色背景 + 白色圆角描边 + 白色大写字母 E
  const textSize = Math.round(size * 0.55);
  const svg = Buffer.from(`
<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">
  <defs>
    <rect id="clip" x="0" y="0" width="${size}" height="${size}" rx="${radius}" ry="${radius}"/>
    <clipPath id="cp"><use href="#clip"/></clipPath>
  </defs>
  <g clip-path="url(#cp)">
    <rect x="0" y="0" width="${size}" height="${size}" fill="#FB923C"/>
    <rect x="${Math.round(size*0.05)}" y="${Math.round(size*0.05)}" width="${Math.round(size*0.9)}" height="${Math.round(size*0.9)}" fill="none" stroke="#FFFFFF" stroke-width="${Math.max(2, Math.round(size*0.02))}"/>
  </g>
  <text x="${size/2}" y="${size/2 + textSize*0.35}" text-anchor="middle" font-family="Arial, sans-serif" font-size="${textSize}" font-weight="700" fill="#FFFFFF">E</text>
</svg>
  `.trim());

  return sharp(svg).resize(size, size).png().toBuffer();
}

/**
 * 把多张 PNG 按 size 拼为 Windows ICO 文件
 * @param {Map<number,Buffer>} pngBySize   key=边长
 * @returns {Buffer}
 */
function pngsToIco(pngBySize) {
  const sizes = Array.from(pngBySize.keys()).sort((a, b) => a - b);
  const chunks = [];

  // ICONDIR
  //   u16 reserved=0
  //   u16 type=1 (ICO)
  //   u16 count
  const iconDir = Buffer.alloc(6);
  iconDir.writeUInt16LE(0, 0);
  iconDir.writeUInt16LE(1, 2);
  iconDir.writeUInt16LE(sizes.length, 4);
  chunks.push(iconDir);

  // ICONDIRENTRY 区域 (每图 16 字节), 后续紧跟图片数据
  //   u8  width (0 表示 256)
  //   u8  height (0 表示 256)
  //   u8  colorCount=0
  //   u8  reserved=0
  //   u16 planes=1
  //   u16 bitCount=32
  //   u32 bytesInRes
  //   u32 imageOffset
  let offset = 6 + sizes.length * 16; // 图片数据起始位置
  const entries = [];
  const imageBlocks = [];

  for (const size of sizes) {
    const buf = pngBySize.get(size);
    const entry = Buffer.alloc(16);
    entry.writeUInt8(size >= 256 ? 0 : size, 0);
    entry.writeUInt8(size >= 256 ? 0 : size, 1);
    entry.writeUInt8(0, 2);
    entry.writeUInt8(0, 3);
    entry.writeUInt16LE(1, 4);
    entry.writeUInt16LE(32, 6);
    entry.writeUInt32LE(buf.length, 8);
    entry.writeUInt32LE(offset, 12);
    entries.push(entry);
    imageBlocks.push(buf);
    offset += buf.length;
  }

  chunks.push(...entries, ...imageBlocks);
  return Buffer.concat(chunks);
}

/**
 * 生成一个最小化的 ICNS (仅包含 128px 的 is32 + mask, 实际用于构建占位).
 * 真正可跨版本的做法是直接打包一张 1024 PNG 到 icns 的 ic10 类型,
 * 但 Tauri 构建对 icns 有一定容忍度, 若未验证会用 128.png 替代.
 *
 * 这里用更稳妥的方案: 把 128.png 二进制当作 ic07 (128px) 块写入 icns 容器,
 * macOS 可识别这是一个 PNG icns.
 *
 * @param {Buffer} png128
 * @returns {Buffer}
 */
function pngToIcnsSimple(png128) {
  // icns 总结构:
  //   "icns" (4) + 总长度(4) + 块...
  //   每个块: 块类型(4) + 块长度(4) + 数据(length-8)
  //   现代 macOS 可直接把 PNG 当作 ic07(128) / ic08(256) / ic09(512) / ic10(1024)
  const type = Buffer.from('ic07'); // 128
  const blockLen = 8 + png128.length;
  const totalLen = 8 + blockLen;

  const header = Buffer.alloc(8);
  header.write('icns', 0);
  header.writeUInt32BE(totalLen, 4);

  const blockHeader = Buffer.alloc(8);
  type.copy(blockHeader, 0);
  blockHeader.writeUInt32BE(blockLen, 4);

  return Buffer.concat([header, blockHeader, png128]);
}

async function main() {
  // 1. 生成各种尺寸 PNG
  const png32 = await makePng(32);
  const png128 = await makePng(128);
  const png256 = await makePng(256);

  fs.writeFileSync(path.join(outDir, '32x32.png'), png32);
  console.log('wrote 32x32.png');

  fs.writeFileSync(path.join(outDir, '128x128.png'), png128);
  console.log('wrote 128x128.png');

  fs.writeFileSync(path.join(outDir, '128x128@2x.png'), png256);
  console.log('wrote 128x128@2x.png');

  // 2. 生成 icon.ico (嵌入 16/32/48/256 四档 PNG)
  const [png16, png48] = await Promise.all([makePng(16), makePng(48)]);
  const icoMap = new Map([
    [16, png16],
    [32, png32],
    [48, png48],
    [256, png256],
  ]);
  const ico = pngsToIco(icoMap);
  fs.writeFileSync(path.join(outDir, 'icon.ico'), ico);
  console.log(`wrote icon.ico (${(ico.length / 1024).toFixed(1)} KB)`);

  // 3. 生成 icon.icns (只放 128 PNG 进 ic07 块, 足够用于构建)
  const icns = pngToIcnsSimple(png128);
  fs.writeFileSync(path.join(outDir, 'icon.icns'), icns);
  console.log(`wrote icon.icns (${(icns.length / 1024).toFixed(1)} KB)`);

  console.log('\nDone. 图标文件位于:', outDir);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
