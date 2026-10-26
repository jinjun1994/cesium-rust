# Bundled Label font

Globe Sans Regular is a renamed, weight-400 subset derived from Noto Sans SC. It is distributed under the SIL Open Font License in OFL.txt. The original copyright and license notices remain in the font and license file.

Coverage: GB2312 repertoire plus selected Latin/Greek/Cyrillic and punctuation ranges. Missing glyphs are rejected. This is not a full Unicode font or a shaping engine. The font is shared by native and WASM builds; no operating-system font is required.

source.json records source and output SHA-256 hashes. To generate the subset from the corresponding downloaded NotoSansSC-VF.ttf, install fontTools and run `python3 scripts/subset-label-font.py /path/to/NotoSansSC-VF.ttf`. The font family and PostScript names are changed to avoid reserved upstream names.
