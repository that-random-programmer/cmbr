import init, * as wasmExports from './pkg/cmbr.js';


self.onmessage = async (event: MessageEvent) => {
  const { sab, source } = event.data;
  const control = new Int32Array(sab, 0, 1);
  const dataBuf = new Uint8Array(sab, 4);

  await init();

  const readLine = () => {
    Atomics.store(control, 0, 0);
    Atomics.wait(control, 0, 0);
    const len = Atomics.load(control, 0);
    return new TextDecoder().decode(dataBuf.slice(0, len));
  };

  const print = (s: string) => self.postMessage({ type: 'stdout', text: s });

  wasmExports.run(source, readLine, print); 
};