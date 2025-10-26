#!/usr/bin/env python3
"""Simple WebSocket test client for chess engine server"""

import asyncio
import websockets
import json

async def test_engine():
    uri = "ws://127.0.0.1:8080"
    print(f"Connecting to {uri}...")

    async with websockets.connect(uri) as websocket:
        print("✅ Connected to WebSocket server")

        # Send analyze request
        request = {
            "type": "analyze",
            "id": "test-1",
            "fen": "startpos",
            "limit": {"kind": "depth", "depth": 5}
        }

        print(f"📤 Sending: {json.dumps(request, indent=2)}")
        await websocket.send(json.dumps(request))

        # Receive messages
        while True:
            try:
                message = await websocket.recv()
                msg = json.loads(message)
                print(f"📥 Received: {msg['type']} - {json.dumps(msg['payload'], indent=2)}")

                if msg['type'] == 'bestMove':
                    print(f"✅ Test successful! Best move: {msg['payload']['best']}")
                    break
            except websockets.exceptions.ConnectionClosed:
                break

if __name__ == "__main__":
    asyncio.run(test_engine())
