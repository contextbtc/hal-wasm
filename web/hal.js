// Run the hal CLI (compiled for wasm32-wasip1) in the browser.
import {
	WASI,
	File,
	OpenFile,
	ConsoleStdout,
} from "https://cdn.jsdelivr.net/npm/@bjorn3/browser_wasi_shim@0.4.2/dist/index.js";

let modulePromise = null;

/// Compile hal.wasm once; every run instantiates it afresh.
function loadModule(url = "hal.wasm") {
	if (!modulePromise) {
		modulePromise = WebAssembly.compileStreaming(fetch(url));
	}
	return modulePromise;
}

/// Run `hal <args...>` with the given stdin.
/// Returns { code, stdout, stderr }.
export async function runHal(args, stdin = "") {
	const module = await loadModule();

	const stdout = [];
	const stderr = [];
	const fds = [
		new OpenFile(new File(new TextEncoder().encode(stdin))),
		new ConsoleStdout((buf) => stdout.push(buf.slice())),
		new ConsoleStdout((buf) => stderr.push(buf.slice())),
	];
	const wasi = new WASI(["hal", ...args], [], fds, { debug: false });
	const instance = await WebAssembly.instantiate(module, {
		wasi_snapshot_preview1: wasi.wasiImport,
	});

	let code;
	try {
		code = wasi.start(instance);
	} catch (e) {
		// A trap (e.g. an abort) rather than a clean exit.
		stderr.push(new TextEncoder().encode(String(e)));
		code = 134;
	}

	return { code, stdout: decode(stdout), stderr: decode(stderr) };
}

function decode(chunks) {
	const dec = new TextDecoder();
	return chunks.map((c) => dec.decode(c, { stream: true })).join("") + dec.decode();
}

/// Split a command line into arguments, honoring single and double quotes
/// and backslash escapes, like a POSIX shell (without expansion).
export function splitArgs(line) {
	const args = [];
	let cur = "";
	let inArg = false;
	let quote = null;
	for (let i = 0; i < line.length; i++) {
		const c = line[i];
		if (quote === "'") {
			if (c === "'") quote = null;
			else cur += c;
		} else if (quote === '"') {
			if (c === '"') quote = null;
			else if (c === "\\" && i + 1 < line.length && '"\\$`'.includes(line[i + 1])) cur += line[++i];
			else cur += c;
		} else if (c === "'" || c === '"') {
			quote = c;
			inArg = true;
		} else if (c === "\\" && i + 1 < line.length) {
			cur += line[++i];
			inArg = true;
		} else if (/\s/.test(c)) {
			if (inArg) args.push(cur);
			cur = "";
			inArg = false;
		} else {
			cur += c;
			inArg = true;
		}
	}
	if (quote) throw new Error("unterminated quote");
	if (inArg) args.push(cur);
	return args;
}
