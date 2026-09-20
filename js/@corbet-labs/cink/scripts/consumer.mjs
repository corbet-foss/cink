// Copy next to a fresh installation to test the actual npm tarball.
import { createRequire } from 'node:module';
import { verify } from './verify-api.mjs';
verify(await import('@corbet-labs/cink'));
verify(createRequire(import.meta.url)('@corbet-labs/cink'));
verify(await import('@corbet-labs/cink/browser'));
console.log('cink: installed ESM, CommonJS, and browser exports passed');
