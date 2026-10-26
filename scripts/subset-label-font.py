#!/usr/bin/env python3
"""Produce the bundled OFL font subset from a downloaded Noto Sans SC variable TTF."""
import argparse, hashlib, json
from pathlib import Path
from fontTools.ttLib import TTFont
from fontTools import subset
from fontTools.varLib.instancer import instantiateVariableFont
p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('--output',type=Path,default=Path('assets/fonts/GlobeSans-Regular.ttf'));a=p.parse_args()
chars=set(range(0x20,0x250))|set(range(0x300,0x530))|set(range(0x2000,0x2070))
for lead in range(0xa1,0xf8):
 for trail in range(0xa1,0xff):
  try:chars.update(map(ord,bytes([lead,trail]).decode('gb2312')))
  except UnicodeDecodeError:pass
font=TTFont(a.source);options=subset.Options();options.name_IDs=['*'];options.name_legacy=True;options.name_languages=['*']
s=subset.Subsetter(options=options);s.populate(unicodes=chars);s.subset(font);instantiateVariableFont(font,{'wght':400},inplace=True)
# Modified font names avoid reserved upstream names.
for n in font['name'].names:
 if n.nameID in (1,2,3,4,6,16,17):
  name='Regular' if n.nameID in (2,17) else ('GlobeSans-Regular' if n.nameID==6 else 'Globe Sans Regular')
  n.string=name.encode(n.getEncoding(),errors='replace')
font.save(a.output);print(a.output,a.output.stat().st_size)
(a.output.parent/'source.json').write_text(json.dumps({'source':'https://github.com/notofonts/noto-cjk/tree/main/Sans/Variable/TTF/Subset','source_sha256':hashlib.sha256(a.source.read_bytes()).hexdigest(),'font_sha256':hashlib.sha256(a.output.read_bytes()).hexdigest(),'subset':'GB2312 + Latin/Greek/Cyrillic ranges; missing glyphs rejected; weight 400','license':'OFL.txt'},indent=2)+'\n')
