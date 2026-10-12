#!/usr/bin/env python3
"""Regenerate six regional/weight UI subsets from official Noto CJK OTF files.
Usage: python3 scripts/subset_ui_font.py /path/to/noto-font-directory
Upstream: notofonts/noto-cjk @ f8d157532fbfaeda587e826d4cd5b21a49186f7c
SIL OFL 1.1; keep assets/fonts/OFL-CJK.txt with redistributed derivatives.
No full upstream font, local paths, or source download logs are published.
"""
from pathlib import Path
import sys
from fontTools import subset
from fontTools.ttLib import TTFont
text = (Path('src/i18n.rs').read_text() + ''.join(chr(n) for n in range(32, 127))
        + '简体中文English日本語한국어×…≡关于设置搜索插件脚本名称骨直令')
for region in ['sc', 'jp', 'kr']:
    for weight in ['Regular', 'Medium']:
        source = Path(sys.argv[1]) / f'NotoSansCJK{region}-{weight}.otf'
        options = subset.Options()
        options.name_IDs = ['*']; options.name_legacy = True; options.name_languages = ['*']
        font = TTFont(source)
        tool = subset.Subsetter(options=options)
        tool.populate(unicodes={ord(c) for c in text})
        tool.subset(font)
        family = f'CavalryKit UI {region.upper()}'
        for record in font['name'].names:
            names = {1: family, 2: weight, 4: f'{family} {weight}',
                     6: f'CavalryKitUI{region.upper()}-{weight}', 16: family, 17: weight}
            if record.nameID in names:
                record.string = names[record.nameID].encode(record.getEncoding(), errors='replace')
        path = Path(f'assets/fonts/CavalryKit-UI-{region.upper()}-{weight}.otf')
        font.save(path)
        print(path.name, path.stat().st_size)
