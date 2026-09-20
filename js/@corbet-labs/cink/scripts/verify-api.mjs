// This module also runs in a real browser with no Node shims.
export function verify(api) {
    const check = (actual, expected) => {
        if (JSON.stringify(actual) !== JSON.stringify(expected)) {
            throw new Error(`${JSON.stringify(actual)} !== ${JSON.stringify(expected)}`);
        }
    };
    check(api.supportedFormats(), ['jpeg', 'png', 'svg']);
    check(api.imageSnippet('/ink.png', 30), '#image("/ink.png", height: 30pt)');
    check(api.normalize('invalid'), null);
    // Public callers can construct images; reject values Rust's u32 fields cannot hold.
    check(api.signatureSize({ mime: 'png', width: 0, height: 1, bytes: new Uint8Array() }, 20), null);
    check(api.signatureSize({ mime: 'png', width: 1.5, height: 1, bytes: new Uint8Array() }, 20), null);
    check(api.signatureSize({ mime: 'png', width: 1, height: 1, bytes: new Uint8Array() }, NaN), null);
    check(api.signatureSize({ mime: 'png', width: 1, height: 1, bytes: new Uint8Array() }, Infinity), null);
    check(api.signatureSize({ mime: 'png', width: 1, height: 1, bytes: new Uint8Array() }, 20, Infinity), null);
    check(api.signatureSize({ mime: 'png', width: 2, height: 1, bytes: new Uint8Array() }, Number.MAX_VALUE, 30), [30, 15]);
    check(api.signatureSize({ mime: 'png', width: 1, height: 2, bytes: new Uint8Array() }, Number.MIN_VALUE), null);
}
