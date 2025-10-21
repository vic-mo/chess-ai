// Placeholder worker. When wiring the real WASM, move heavy work here.
self.onmessage = (ev: MessageEvent<{ type?: string }>) => {
  const msg = ev.data;
  if (msg?.type === 'ping') {
    self.postMessage({ type: 'pong' });
  }
};
