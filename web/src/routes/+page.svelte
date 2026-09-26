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
		term = new Terminal();
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
<div class="flex h-full">
	<div bind:this={editorEl} class="h-full w-full"></div>
	<div bind:this={terminalEl} class="h-full w-full"></div>
</div>
