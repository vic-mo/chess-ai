// Simple WebSocket test client
const WebSocket = require('ws');

const ws = new WebSocket('ws://127.0.0.1:8080');

ws.on('open', () => {
  console.log('✅ Connected to WebSocket server');

  // Send analyze request
  const request = {
    type: 'analyze',
    id: 'test-1',
    fen: 'startpos',
    limit: { kind: 'depth', depth: 5 },
  };

  console.log('📤 Sending:', JSON.stringify(request, null, 2));
  ws.send(JSON.stringify(request));
});

ws.on('message', (data) => {
  const msg = JSON.parse(data.toString());
  console.log('📥 Received:', msg.type, '-', JSON.stringify(msg.payload, null, 2));

  // Close after receiving bestMove
  if (msg.type === 'bestMove') {
    console.log('✅ Test successful! Best move:', msg.payload.best);
    setTimeout(() => {
      ws.close();
      process.exit(0);
    }, 100);
  }
});

ws.on('error', (error) => {
  console.error('❌ WebSocket error:', error);
  process.exit(1);
});

ws.on('close', () => {
  console.log('Connection closed');
});
