/* tslint:disable */
/* eslint-disable */

export class WkWasmDecoder {
    free(): void;
    [Symbol.dispose](): void;
    decode(): WkWasmImage;
    constructor(data: Uint8Array);
}

export class WkWasmImage {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    get_pixels(): Uint8Array;
    readonly color_type: string;
    readonly compression: string;
    readonly height: number;
    readonly quality: number;
    readonly width: number;
}

export function decode_wk(data: Uint8Array): WkWasmImage;

export function init_panic_hook(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wkwasmdecoder_free: (a: number, b: number) => void;
    readonly __wbg_wkwasmimage_free: (a: number, b: number) => void;
    readonly decode_wk: (a: number, b: number) => [number, number, number];
    readonly wkwasmdecoder_decode: (a: number) => [number, number, number];
    readonly wkwasmdecoder_new: (a: number, b: number) => number;
    readonly wkwasmimage_color_type: (a: number) => [number, number];
    readonly wkwasmimage_compression: (a: number) => [number, number];
    readonly wkwasmimage_get_pixels: (a: number) => [number, number];
    readonly wkwasmimage_height: (a: number) => number;
    readonly wkwasmimage_quality: (a: number) => number;
    readonly wkwasmimage_width: (a: number) => number;
    readonly init_panic_hook: () => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
