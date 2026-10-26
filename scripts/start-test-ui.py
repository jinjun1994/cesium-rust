#!/usr/bin/env python3
"""Start the test workbench on localhost using the bundled WASM SDK."""
import argparse, functools, http.server, pathlib, webbrowser
ROOT=pathlib.Path(__file__).resolve().parents[1]
class Handler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path=='/':
            self.send_response(302);self.send_header('Location','/bindings/web/');self.end_headers();return
        super().do_GET()
    def end_headers(self):
        self.send_header('Cache-Control','no-cache');super().end_headers()
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--port',type=int,default=8000);p.add_argument('--no-browser',action='store_true');a=p.parse_args()
    server=http.server.ThreadingHTTPServer(('127.0.0.1',a.port),functools.partial(Handler,directory=str(ROOT)))
    url=f'http://127.0.0.1:{server.server_port}/';print('Test UI: '+url,flush=True)
    if not a.no_browser:webbrowser.open(url)
    try:server.serve_forever()
    except KeyboardInterrupt:pass
    finally:server.server_close()
