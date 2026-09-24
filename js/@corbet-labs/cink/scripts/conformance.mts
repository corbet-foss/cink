/**
 * TypeScript side of the cross-language conformance gate: runs every
 * `tests/vectors/*.json` vector through `src/index.ts`, reading fixtures
 * from `tests/fixtures/`, and compares with `expected` exactly. Exits
 * non-zero with the first mismatch.
 *
 * Run from the package root:
 *   bun ./scripts/conformance.mts
 */
import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join, resolve } from 'node:path';
import {
    exceedsLimits,
    imageSnippet,
    normalize,
    scaleToFit,
    signatureSize,
    supportedFormats,
    defaultMaxPixels,
    type DecodedImage,
} from '../src/index.js';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../../../..');

interface Vector {
    name: string;
    fn: string;
    fixture?: string;
    input?: string;
    max_pixels?: number;
    height_pt?: number;
    max_width_pt?: number;
    path?: string;
    width_pt?: number;
    expected: unknown;
}

function normalizeFixture(file: string): DecodedImage | null {
    const bytes = readFileSync(join(ROOT, 'tests/fixtures', file));
    return normalize(Buffer.from(bytes).toString('base64'));
}

function imageValue(image: DecodedImage): unknown {
    return { mime: image.mime, width: image.width, height: image.height };
}

function runVector(file: string, vector: Vector): void {
    const what = `${file} :: ${vector.name}`;
    let actual: unknown;
    switch (vector.fn) {
        case 'default_max_pixels':
            actual = defaultMaxPixels();
            break;
        case 'supported_formats':
            actual = supportedFormats();
            break;        case 'normalize':
            if (vector.fixture !== undefined) {
                const image = normalizeFixture(vector.fixture);
                actual = image === null ? null : imageValue(image);
            } else {
                const image = normalize(vector.input ?? '');
                actual = image === null ? null : imageValue(image);
            }
            break;
        case 'exceeds_limits': {
            if (vector.fixture === undefined || vector.max_pixels === undefined) {
                throw new Error(`${what}: missing fixture/max_pixels`);
            }
            const image = normalizeFixture(vector.fixture);
            if (image === null) throw new Error(`${what}: fixture must normalize`);
            actual = exceedsLimits(image, vector.max_pixels);
            break;
        }
        case 'scale_to_fit': {
            if (vector.fixture === undefined || vector.max_pixels === undefined) {
                throw new Error(`${what}: missing fixture/max_pixels`);
            }
            const image = normalizeFixture(vector.fixture);
            if (image === null) throw new Error(`${what}: fixture must normalize`);
            actual = scaleToFit(image, vector.max_pixels);
            break;
        }
        case 'signature_size': {
            if (vector.fixture === undefined || vector.height_pt === undefined) {
                throw new Error(`${what}: missing fixture/height_pt`);
            }
            const image = normalizeFixture(vector.fixture);
            if (image === null) {
                actual = null;
            } else {
                actual = signatureSize(image, vector.height_pt, vector.max_width_pt);
            }
            break;
        }
        case 'image_snippet': {
            if (vector.path === undefined || vector.height_pt === undefined) {
                throw new Error(`${what}: missing path/height_pt`);
            }
            actual = imageSnippet(vector.path, vector.height_pt, vector.width_pt);
            break;
        }
        default:
            throw new Error(`${what}: unknown fn ${vector.fn}`);
    }
    const got = JSON.stringify(actual) ?? 'undefined';
    const want = JSON.stringify(vector.expected) ?? 'undefined';
    if (got !== want) throw new Error(`${what}: ${got} vs ${want}`);
}

const dir = join(ROOT, 'tests/vectors');
const files = readdirSync(dir)
    .filter((file) => file.endsWith('.json'))
    .sort();
if (files.length === 0) throw new Error('no vector files in tests/vectors');
let count = 0;
for (const file of files) {
    const vectors = JSON.parse(readFileSync(join(dir, file), 'utf8')) as Vector[];
    for (const vector of vectors) {
        runVector(file, vector);
        count += 1;
    }
}
console.log(`TS conformance green: ${count} vectors across ${files.length} files`);
