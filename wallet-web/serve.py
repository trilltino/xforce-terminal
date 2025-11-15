#!/usr/bin/env python3
"""
Simple HTTP server for wallet connection page
Serves the simple-connect.html file on port 8080
"""
import http.server
import socketserver
import os

PORT = 8080
DIRECTORY = os.path.dirname(__file__)

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

    def do_GET(self):
        # Serve simple-connect.html for the root path or any path with token param
        if self.path == '/' or self.path.startswith('/?token='):
            self.path = '/public/simple-connect.html' + self.path.replace('/', '?', 1) if '?' in self.path else '/public/simple-connect.html'
        return super().do_GET()

with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
    print(f"Wallet connection server running at http://localhost:{PORT}/")
    print("Press Ctrl+C to stop")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped")
