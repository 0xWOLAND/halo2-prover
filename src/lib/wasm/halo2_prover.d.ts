/* tslint:disable */
/* eslint-disable */
/**
* @returns {string}
*/
export function hello_world(): string;
/**
* @param {number} k
* @returns {Uint8Array}
*/
export function setup(k: number): Uint8Array;
/**
* @param {Uint8Array} _params
* @param {string} s
* @param {number} circuit
* @returns {Uint8Array}
*/
export function wasm_generate_proof(_params: Uint8Array, s: string, circuit: number): Uint8Array;
/**
* @param {Uint8Array} _params
* @param {Uint8Array} proof
* @returns {boolean}
*/
export function wasm_verify_proof(_params: Uint8Array, proof: Uint8Array): boolean;
/**
* @returns {number}
*/
export function get_circuit_count(): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly hello_world: (a: number) => void;
  readonly setup: (a: number) => number;
  readonly wasm_generate_proof: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly wasm_verify_proof: (a: number, b: number, c: number, d: number) => number;
  readonly get_circuit_count: () => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
