const WebSocket = require('ws');

// Create WebSocket connection
const ws = new WebSocket('ws://127.0.0.1:8081');

ws.on('open', function open() {
  console.log('Connected to WebSocket server');
  
  // Create a UserChatMessage
  const message = {
    message: "Hello from Node.js!"
  };
  
  // Serialize using a simple approach (we'll use JSON first to test)
  const jsonData = JSON.stringify(message);
  console.log('Sending JSON:', jsonData);
  ws.send(jsonData);
});

ws.on('message', function incoming(data) {
  console.log('Received:', data);
  console.log('Type:', typeof data);
  console.log('Buffer:', Buffer.from(data));
});

ws.on('error', function error(err) {
  console.error('WebSocket error:', err);
});

ws.on('close', function close() {
  console.log('Disconnected from WebSocket server');
});

