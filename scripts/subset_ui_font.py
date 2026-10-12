#!/usr/bin/env python3
"""Build a redistributable Noto CJK UI subset. Requires fonttools, not a runtime dependency.
Usage: python3 scripts/subset_ui_font.py /path/to/NotoSansCJKjp-Regular.otf
Source: https://github.com/notofonts/noto-cjk (SIL OFL 1.1; see assets/fonts/OFL-CJK.txt)
"""
from pathlib import Path
import sys
from fontTools import subset
from fontTools.ttLib import TTFont
text=Path('src/i18n.rs').read_text()+''.join(chr(n) for n in range(32,127))+'简体中文English日本語한국어×…≡'
options=subset.Options();options.name_IDs=['*'];options.name_legacy=True;options.name_languages=['*']
font=TTFont(sys.argv[1]);tool=subset.Subsetter(options=options);tool.populate(unicodes={ord(c) for c in text});tool.subset(font)
# A renamed subset avoids claiming it is the complete upstream Noto font.
for record in font['name'].names:
    names={1:'CavalryKit UI CJK',4:'CavalryKit UI CJK Regular',6:'CavalryKitUICJK-Regular'}
    if record.nameID in names:record.string=names[record.nameID].encode(record.getEncoding(),errors='replace')
font.save('assets/fonts/CavalryKit-UI-CJK.otf')
print('Wrote UI-only OFL subset; full font not included in the repository')
