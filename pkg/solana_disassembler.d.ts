/* tslint:disable */
/* eslint-disable */
/**
*/
export class ElfFile {
  free(): void;
/**
* @returns {ElfFile}
*/
  static new(): ElfFile;
/**
* @param {Uint8Array} data
*/
  load(data: Uint8Array): void;
/**
*/
  list_sections(): void;
/**
*/
  disassemble(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_elffile_free: (a: number) => void;
  readonly elffile_new: () => number;
  readonly elffile_load: (a: number, b: number, c: number) => void;
  readonly elffile_list_sections: (a: number) => void;
  readonly elffile_disassemble: (a: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
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
