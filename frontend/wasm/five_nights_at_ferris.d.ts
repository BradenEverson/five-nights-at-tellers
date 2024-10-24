/* tslint:disable */
/* eslint-disable */
/**
 * The full driver for a game responsible for holding both the enemies and the game state
 */
export class Game {
  free(): void;
  /**
   * Ticks the game forward
   * @returns {boolean}
   */
  tick(): boolean;
  /**
   * Create a new game (trait impls aren't accessible to wasm_bindgen
   * @returns {Game}
   */
  static new(): Game;
  /**
   * Close the left door
   */
  toggle_left(): void;
  /**
   * Close the right door
   */
  toggle_right(): void;
  /**
   * Check the current power draw
   * @returns {number}
   */
  power(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_game_free: (a: number, b: number) => void;
  readonly game_tick: (a: number) => number;
  readonly game_new: () => number;
  readonly game_toggle_left: (a: number) => void;
  readonly game_toggle_right: (a: number) => void;
  readonly game_power: (a: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
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
