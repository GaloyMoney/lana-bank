#!/usr/bin/env python3

"""
SumSub Webhook Receiver - No dependencies required
Usage: python3 sumsub-webhook-receiver.py
"""

import http.server
import json
import socketserver
from datetime import datetime

PORT = 5253

class SumsubWebhookHandler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path == '/sumsub/callback':
            # Get content length from headers
            content_length = int(self.headers['Content-Length'])
            # Read request body
            post_data = self.rfile.read(content_length)
            
            try:
                # Parse JSON payload
                payload = json.loads(post_data.decode('utf-8'))
                timestamp = datetime.now().isoformat()
                
                # Print webhook details
                print('\n===== SUMSUB WEBHOOK RECEIVED =====')
                print(f'Time: {timestamp}')
                print('Path: {self.path}')
                print('Headers:')
                for header in self.headers:
                    print(f'  {header}: {self.headers[header]}')
                print('Payload:')
                print(json.dumps(payload, indent=2))
                print('======================================\n')
                
                # Send success response
                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.end_headers()
                self.wfile.write(b'{}')
            except json.JSONDecodeError:
                print('Error: Received invalid JSON')
                self.send_response(400)
                self.end_headers()
        else:
            self.send_response(404)
            self.end_headers()
    
    # Suppress default HTTP request logging
    def log_message(self, format, *args):
        return

if __name__ == '__main__':
    print(f"SumSub webhook server listening at http://localhost:{PORT}")
    print(f"Ready to receive callbacks at http://localhost:{PORT}/sumsub/callback")
    print("Press Ctrl+C to stop the server")
    
    # Create and start the HTTP server
    with socketserver.TCPServer(("", PORT), SumsubWebhookHandler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print('\nServer stopped')