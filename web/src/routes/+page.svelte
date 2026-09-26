<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import '@xterm/xterm/css/xterm.css';
	import { FitAddon } from '@xterm/addon-fit';
	let terminalEl: HTMLDivElement;
	let term: import('@xterm/xterm').Terminal;
	let fitAddon: import('@xterm/addon-fit').FitAddon;
	let resizeObserver: ResizeObserver;
	let editorEl: HTMLDivElement;
	let editor: import('monaco-editor').editor.IStandaloneCodeEditor;

	const CONTROL_BYTES = 4;
	const DATA_BYTES = 1024;
	const sab = new SharedArrayBuffer(CONTROL_BYTES + DATA_BYTES);
	const control = new Int32Array(sab, 0, 1);
	const dataBuf = new Uint8Array(sab, 4);
	let worker: Worker;
	const dark2026Theme = {
		background: '#181818',
		foreground: '#cccccc',
		cursor: '#cccccc',
		cursorAccent: '#181818',
		selectionBackground: '#264f78',
		black: '#000000',
		red: '#cd3131',
		green: '#0dbc79',
		yellow: '#e5e510',
		blue: '#2472c8',
		magenta: '#bc3fbc',
		cyan: '#11a8cd',
		white: '#e5e5e5',
		brightBlack: '#666666',
		brightRed: '#f14c4c',
		brightGreen: '#23d18b',
		brightYellow: '#f5f543',
		brightBlue: '#3b8eea',
		brightMagenta: '#d670d6',
		brightCyan: '#29b8db',
		brightWhite: '#e5e5e5'
	};

	function run() {
		term.clear();
		console.log(worker);
		worker.postMessage({
			sab,
			source: editor?.getValue()
		});
	}
	onMount(async () => {
		const monaco = await import('monaco-editor');
		const editorWorker = (await import('monaco-editor/esm/vs/editor/editor.worker?worker')).default;

		self.MonacoEnvironment = {
			getWorker() {
				return new editorWorker();
			}
		};

		worker = new Worker(new URL('../lib/interp-worker.js', import.meta.url), {
			type: 'module'
		});

		worker.onmessage = ({ data }) => {
			if (data.type == 'stdout') {
				term?.write(data.text);
				console.log(data);
			}
		};

		function submitLine(line: string) {
			const bytes = new TextEncoder().encode(line + '\n');
			dataBuf.set(bytes);
			Atomics.store(control, 0, bytes.length);
			Atomics.notify(control, 0);
		}

		const { Terminal } = await import('@xterm/xterm');

		fitAddon = new FitAddon();
		term = new Terminal({
			theme: dark2026Theme
		});
		term.loadAddon(fitAddon);
		term.open(terminalEl);
		fitAddon.fit();

		resizeObserver = new ResizeObserver(() => fitAddon.fit());
		resizeObserver.observe(terminalEl);

		let lineBuffer = '';
		editor = monaco.editor.create(editorEl, {
			value: '// write your program here, then hit the run button at the top\n',
			language: 'plaintext',
			theme: 'vs-dark',
			automaticLayout: true
		});
		console.log(term);
		term.onData((data) => {
			// basically set it into cooked mode
			if (data === '\r') {
				term.write('\r\n');
				submitLine(lineBuffer);
				lineBuffer = '';
			} else if (data === '\u007F') {
				if (lineBuffer.length) {
					lineBuffer = lineBuffer.slice(0, -1);
					term?.write('\b \b');
				}
			} else {
				lineBuffer += data;
				term?.write(data);
			}
		});
	});

	onDestroy(() => {
		term?.dispose();
	});
</script>

<div><button class="btn w-full" onclick={run}>run</button></div>
<div class="flex h-full max-md:flex-col">
	<div bind:this={editorEl} class="h-full w-full"></div>
	<div bind:this={terminalEl} class="h-full w-full"></div>
</div>
