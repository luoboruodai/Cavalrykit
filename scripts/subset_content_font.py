#!/usr/bin/env python3
"""Create a deterministic, all-Unicode content face from the approved Noto CJK SC Regular source.
Usage: python3 scripts/subset_content_font.py /path/to/NotoSansCJKsc-Regular.otf
No user filenames are sampled. Source and license match packaging/THIRD-PARTY-NOTICES.txt.
"""
from pathlib import Path
import sys
from fontTools import subset
from fontTools.ttLib import TTFont
font = TTFont(sys.argv[1], recalcTimestamp=False)
options = subset.Options()
options.name_IDs = ['*']
options.name_legacy = True
options.name_languages = ['*']
subsetter = subset.Subsetter(options=options)
subsetter.populate(unicodes=set(font.getBestCmap()))
subsetter.subset(font)
for record in font['name'].names:
    names = {1:'CavalryKit Content',2:'Regular',4:'CavalryKit Content Regular',6:'CavalryKitContent-Regular',16:'CavalryKit Content',17:'Regular'}
    if record.nameID in names:
        record.string=names[record.nameID].encode(record.getEncoding(), errors='replace')
out=Path('assets/fonts/CavalryKit-Content-Regular.otf')
font.save(out)
print(out, out.stat().st_size, 'bytes;', len(font.getBestCmap()), 'Unicode code points')
