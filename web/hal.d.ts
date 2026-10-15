export interface HalResult {
	code: number;
	stdout: string;
	stderr: string;
}

/** Run `hal <args...>` with the given stdin. */
export function runHal(args: string[], stdin?: string): Promise<HalResult>;

/** Load hal.wasm from a different URL. Call before the first runHal(). */
export function setWasmUrl(url: string | URL): void;

/**
 * Split a command line into arguments, honoring single and double quotes
 * and backslash escapes, like a POSIX shell (without expansion).
 */
export function splitArgs(line: string): string[];
