"""Audit the shipped subset against original metrics, outlines, and all UI glyphs."""
import json,struct
from pathlib import Path
from fontTools.ttLib import TTFont
from fontTools.pens.boundsPen import BoundsPen
from prepare_languages import ROOT,records,required_characters
original=TTFont(ROOT/'assets/NotoSansCJKkr-Regular.otf');font=TTFont(ROOT/'assets/languages/font.shared.otf')
a,b=original.getBestCmap(),font.getBestCmap();ag,bg=original.getGlyphSet(),font.getGlyphSet()
chars=required_characters(list(records((ROOT/'assets/ui-names.bin').read_bytes())),list(records((ROOT/'assets/ui-text.bin').read_bytes())))
for lang,wanted in enumerate(chars):assert wanted<=b.keys(),(lang,'missing glyphs',wanted-b.keys())
for c in set.union(*chars):
 assert original['hmtx'][a[c]]==font['hmtx'][b[c]],(c,'advance changed')
 p,q=BoundsPen(ag),BoundsPen(bg);ag[a[c]].draw(p);bg[b[c]].draw(q);assert p.bounds==q.bounds,(c,'outline bounds changed')
assert (original['hhea'].ascent,original['hhea'].descent)==(font['hhea'].ascent,font['hhea'].descent)
print('Shared subset: all 9 languages covered;',len(set.union(*chars)),'glyph metrics and outline bounds match the original.')
