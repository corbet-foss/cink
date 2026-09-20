/**
 * Handwritten signature image rendering for formal correspondence.
 *
 * Pure TypeScript port of the cink Rust crate: zero dependencies, zero
 * Node APIs (base64 via global atob), synchronous, no I/O. Pixels, not
 * signatures — no cryptography, no identity, no legal ceremony.
 *
 * Behavior is defined by `tables/defaults.json` plus the byte-level rules
 * in `tables/README.md`; `tests/vectors/*.json` with fixtures in
 * `tests/fixtures/` is the shared conformance suite.
 */
import { DEFAULTS } from './generated/tables.ts';

export interface Defaults {
    max_pixels: number;
    formats: string[];
}

const defaults = DEFAULTS as Defaults;

export type ImageMime = 'png' | 'jpeg' | 'svg';

export interface DecodedImage {
    mime: ImageMime;
    width: number | null;
    height: number | null;
    bytes: Uint8Array;
}

/** Default oversize threshold in pixels, from `tables/defaults.json`. */
export function defaultMaxPixels(): number {
    return defaults.max_pixels;
}

/** Accepted image formats, from `tables/defaults.json`. */
export function supportedFormats(): string[] {
    return [...defaults.formats].sort();
}

const PNG_MAGIC = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

function pngDimensions(bytes: Uint8Array): [number, number] | null {
    if (bytes.length < 33) return null;
    for (let i = 0; i < 8; i++) if (bytes[i] !== PNG_MAGIC[i]) return null;
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    if (view.getUint32(8) !== 13) return null;
    if (view.getUint32(12) !== 0x49484452) return null;
    const width = view.getUint32(16);
    const height = view.getUint32(20);
    if (width === 0 || height === 0 || width > 0x7fffffff || height > 0x7fffffff) return null;
    const depth = bytes[24];
    const color = bytes[25];
    const validDepth = color === 0 ? [1, 2, 4, 8, 16].includes(depth)
        : color === 3 ? [1, 2, 4, 8].includes(depth)
        : [2, 4, 6].includes(color) && [8, 16].includes(depth);
    if (!validDepth || bytes[26] !== 0 || bytes[27] !== 0 || bytes[28] > 1) return null;
    return [width, height];
}

const SOF_MARKERS = new Set([
    0xc0, 0xc1, 0xc2, 0xc3, 0xc5, 0xc6, 0xc7, 0xc9, 0xca, 0xcb, 0xcd, 0xce, 0xcf,
]);

function jpegDimensions(bytes: Uint8Array): [number, number] | null {
    if (bytes.length < 2 || bytes[0] !== 0xff || bytes[1] !== 0xd8) return null;
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    let pos = 2;
    while (pos < bytes.length) {
        if (bytes[pos] !== 0xff) return null;
        while (pos < bytes.length && bytes[pos] === 0xff) pos++;
        if (pos >= bytes.length) return null;
        const marker = bytes[pos];
        pos++;
        if (marker === 0x01) continue;
        if (marker === 0x00 || (marker >= 0xd0 && marker <= 0xda)) return null;
        if (pos + 2 > bytes.length) return null;
        const len = view.getUint16(pos);
        if (len < 2 || len > bytes.length - pos) return null;
        if (SOF_MARKERS.has(marker)) {
            if (len < 8) return null;
            const components = bytes[pos + 7];
            if (components === 0 || len !== 8 + 3 * components) return null;
            const height = view.getUint16(pos + 3);
            const width = view.getUint16(pos + 5);
            if (width === 0 || height === 0) return null;
            return [width, height];
        }
        pos += len;
    }
    return null;
}

function sniff(bytes: Uint8Array): DecodedImage | null {
    if (bytes.length >= 8 && PNG_MAGIC.every((b, i) => bytes[i] === b)) {
        const dims = pngDimensions(bytes);
        if (dims === null) return null;
        return { mime: 'png', width: dims[0], height: dims[1], bytes };
    }
    if (bytes.length >= 2 && bytes[0] === 0xff && bytes[1] === 0xd8) {
        const dims = jpegDimensions(bytes);
        if (dims === null) return null;
        return { mime: 'jpeg', width: dims[0], height: dims[1], bytes };
    }
    let start = 0;
    while (start < bytes.length && [9, 10, 12, 13, 32].includes(bytes[start])) start++;
    const head = String.fromCharCode(...bytes.slice(start, start + 4));
    if (head === '<svg') return { mime: 'svg', width: null, height: null, bytes };
    return null;
}

function decodeBase64(payload: string): Uint8Array | null {
    if (payload === '') return null;
    try {
        const text = atob(payload);
        // Rust STANDARD accepts only canonical base64, including required padding.
        if (btoa(text) !== payload) return null;
        const bytes = new Uint8Array(text.length);
        for (let i = 0; i < text.length; i++) bytes[i] = text.charCodeAt(i);
        return bytes.length > 0 ? bytes : null;
    } catch {
        return null;
    }
}

/**
 * Recognize a signature image given as a `data:` URL or raw canonical base64.
 * Required padding must be present; whitespace and nonzero unused bits are rejected.
 * Returns `null` for empty, undecodable, corrupt, or unsupported inputs.
 */
export function normalize(input: string): DecodedImage | null {
    let payload = input;
    const comma = input.indexOf(',');
    if (comma >= 0 && input.startsWith('data:')) {
        const meta = input.slice(0, comma);
        if (!meta.endsWith(';base64')) return null;
        payload = input.slice(comma + 1);
    }
    const bytes = decodeBase64(payload);
    if (bytes === null) return null;
    return sniff(bytes);
}

/**
 * Whether the raster exceeds a pixel budget. Dimension-less formats (SVG)
 * scale without loss and never exceed.
 */
export function exceedsLimits(image: DecodedImage, maxPixels: number): boolean {
    if (image.width === null || image.height === null) return false;
    return image.width * image.height > maxPixels;
}

/**
 * Linear scale factor that fits the image into a pixel budget, never
 * above 1.0 (never upscales). The caller resamples and re-submits.
 */
export function scaleToFit(image: DecodedImage, maxPixels: number): number {
    if (image.width === null || image.height === null) return 1.0;
    const pixels = image.width * image.height;
    if (pixels <= maxPixels) return 1.0;
    return Math.sqrt(maxPixels / pixels);
}

/**
 * Point size for a target height, preserving aspect ratio, clamped to an
 * optional maximum width. Returns `null` for missing or invalid dimensions,
 * nonpositive/nonfinite point sizes, or a nonrepresentable result.
 */
export function signatureSize(
    image: DecodedImage,
    heightPt: number,
    maxWidthPt?: number,
): [number, number] | null {
    if (image.width === null || image.height === null
        || !Number.isInteger(image.width) || !Number.isInteger(image.height)
        || image.width <= 0 || image.height <= 0
        || image.width > 0xffffffff || image.height > 0xffffffff
        || !Number.isFinite(heightPt) || heightPt <= 0
        || (maxWidthPt !== undefined && (!Number.isFinite(maxWidthPt) || maxWidthPt <= 0))) return null;
    const ratio = image.width / image.height;
    const natural = heightPt * ratio;
    let width = natural;
    let height = heightPt;
    if (maxWidthPt !== undefined && natural > maxWidthPt) {
        width = maxWidthPt;
        height = maxWidthPt / ratio;
    }
    return Number.isFinite(width) && Number.isFinite(height) && width > 0 && height > 0
        ? [width, height] : null;
}

function escapePath(path: string): string {
    return path.replace(/["\\\u0000-\u001f\u007f-\u009f]/g, (character) => {
        if (character === '"') return '\\"';
        if (character === '\\') return '\\\\';
        if (character === '\n') return '\\n';
        if (character === '\r') return '\\r';
        if (character === '\t') return '\\t';
        return `\\u{${character.charCodeAt(0).toString(16)}}`;
    });
}

/**
 * Typst `#image` call for a sized signature. Width is emitted only when a
 * max-width clamp applied; layout and placement stay with the caller.
 */
export function imageSnippet(path: string, heightPt: number, widthPt?: number): string {
    const safe = escapePath(path);
    if (widthPt === undefined) return `#image("${safe}", height: ${heightPt}pt)`;
    return `#image("${safe}", height: ${heightPt}pt, width: ${widthPt}pt)`;
}
