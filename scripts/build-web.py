#!/usr/bin/env python3
"""Build the pinned WASM SDK and generate native ES modules."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
root = Path(__file__).resolve().parents[1]
target_dir=Path(os.environ.get('CARGO_TARGET_DIR',root/'target'))
if not target_dir.is_absolute():target_dir=root/target_dir
bindgen = os.environ.get('WASM_BINDGEN') or shutil.which('wasm-bindgen')
if not bindgen:
    sys.exit('Install: cargo install wasm-bindgen-cli --version 0.2.129 --locked')
if '0.2.129' not in subprocess.check_output([bindgen, '--version'], text=True):
    sys.exit('wasm-bindgen-cli must match the locked wasm-bindgen version: 0.2.129')
subprocess.run(['cargo','build','--locked','-p','cesium-viewer','--lib','--target','wasm32-unknown-unknown','--release'],cwd=root,check=True)
subprocess.run([bindgen,str(target_dir/'wasm32-unknown-unknown/release/cesium_viewer.wasm'),'--target','web','--out-dir',str(root/'bindings/web/pkg')],cwd=root,check=True)
print('Serve: python3 -m http.server 8000\nOpen: http://localhost:8000/bindings/web/')
