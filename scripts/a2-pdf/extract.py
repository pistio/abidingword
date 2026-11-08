"""Run one engine on one PDF in its own process.
usage: python3 -I extract.py <engine> <mode:text|struct|null> <pdf> <out>
text  -> writes page texts joined by \\f to <out> and a small JSON with page counts to <out>.json
struct-> writes JSON with geometry/structure availability and counts to <out>
"""
import sys, json, time, traceback
eng, mode, pdf, out = sys.argv[1:5]

def w(obj):
    with open(out, 'w') as f: json.dump(obj, f, ensure_ascii=False, indent=1)

def pdfium_text(variant):
    import pypdfium2 as pdfium
    doc = pdfium.PdfDocument(pdf)
    pages = []; errs = 0
    for i in range(len(doc)):
        try:
            pg = doc[i]; tp = pg.get_textpage()
            pages.append(tp.get_text_range()); tp.close(); pg.close()
        except Exception as e:
            errs += 1; pages.append('')
    n = len(doc); doc.close()
    return pages, {'page_count': n, 'page_errors': errs}

def mupdf_text(variant):
    import pymupdf
    pymupdf.TOOLS.mupdf_display_errors(False)
    doc = pymupdf.open(pdf)
    pages = []; errs = 0
    for pg in doc:
        try:
            pages.append(pg.get_text('text', sort=(variant == 'sort')))
        except Exception:
            errs += 1; pages.append('')
    warn = pymupdf.TOOLS.mupdf_warnings()
    return pages, {'page_count': doc.page_count, 'page_errors': errs, 'repaired': bool(doc.is_repaired),
                   'warnings': warn[:2000]}

def pdfminer_text(variant):
    from pdfminer.high_level import extract_text
    t = extract_text(pdf)
    pages = t.split('\f')
    if pages and pages[-1] == '': pages = pages[:-1]
    return pages, {'page_count': len(pages), 'page_errors': 0}

def struct_pdfium():
    import pypdfium2 as pdfium, pypdfium2.raw as r, ctypes
    doc = pdfium.PdfDocument(pdf)
    res = {'pages': len(doc)}
    chars = boxes = fontinfo = 0; sizes = set(); fonts = set(); links = 0; labels = 0; struct_pages = 0
    buf = ctypes.create_string_buffer(256); flags = ctypes.c_int()
    for i in range(len(doc)):
        pg = doc[i]; tp = pg.get_textpage()
        n = tp.count_chars(); chars += n
        for j in range(0, n, max(1, n // 40 or 1)):   # sample ~40 chars/page
            try:
                if tp.get_charbox(j): boxes += 1
            except Exception: pass
            l = r.FPDFText_GetFontInfo(tp.raw, j, buf, 256, ctypes.byref(flags))
            if l > 0: fontinfo += 1; fonts.add(buf.value.decode('utf-8', 'replace'))
            sizes.add(round(r.FPDFText_GetFontSize(tp.raw, j), 1))
        pos = ctypes.c_int(0); lk = r.FPDF_LINK()
        while r.FPDFLink_Enumerate(pg.raw, ctypes.byref(pos), ctypes.byref(lk)): links += 1
        L = r.FPDF_GetPageLabel(doc.raw, i, None, 0)
        if L > 2: labels += 1
        st = r.FPDF_StructTree_GetForPage(pg.raw)
        if st:
            if r.FPDF_StructTree_CountChildren(st) > 0: struct_pages += 1
            r.FPDF_StructTree_Close(st)
        tp.close(); pg.close()
    toc = 0
    try:
        for _ in doc.get_toc(max_depth=30): toc += 1
    except Exception as e: res['toc_error'] = str(e)
    res.update(chars=chars, char_boxes_sampled_ok=boxes, font_name_samples_ok=fontinfo,
               distinct_fonts_sampled=len(fonts), distinct_sizes_sampled=len(sizes), link_annots=links,
               pages_with_label=labels, pages_with_struct_tree=struct_pages, outline_entries=toc)
    return res

def struct_mupdf():
    import pymupdf
    pymupdf.TOOLS.mupdf_display_errors(False)
    doc = pymupdf.open(pdf)
    res = {'pages': doc.page_count}
    chars = spans = 0; fonts = set(); sizes = set(); links = 0; labels = 0
    for pg in doc:
        d = pg.get_text('rawdict')
        for b in d['blocks']:
            for l in b.get('lines', []):
                for s in l['spans']:
                    spans += 1; fonts.add(s['font']); sizes.add(round(s['size'], 1)); chars += len(s['chars'])
        links += len(pg.get_links())
        try:
            if pg.get_label(): labels += 1
        except Exception: pass
    toc = len(doc.get_toc(simple=True))
    cat = doc.pdf_catalog()
    st = doc.xref_get_key(cat, 'StructTreeRoot')
    mi = doc.xref_get_key(cat, 'MarkInfo')
    # ActualText occurrences: in object dictionaries and in content streams (raw scan, a document property)
    at_obj = at_stream = 0
    for x in range(1, doc.xref_length()):
        try:
            o = doc.xref_object(x, compressed=False)
            if '/ActualText' in o: at_obj += 1
            if doc.xref_is_stream(x):
                s = doc.xref_stream(x)
                if s and b'/ActualText' in s: at_stream += 1
        except Exception: pass
    res.update(chars=chars, spans_with_font_and_size=spans, distinct_fonts=len(fonts), distinct_sizes=len(sizes),
               link_annots=links, pages_with_label=labels, outline_entries=toc,
               struct_tree_root=st[0] != 'null', mark_info=mi[1] if mi[0] != 'null' else None,
               actualtext_objects=at_obj, actualtext_streams=at_stream)
    return res

def struct_pdfminer():
    from pdfminer.pdfparser import PDFParser
    from pdfminer.pdfdocument import PDFDocument
    from pdfminer.pdfpage import PDFPage
    from pdfminer.pdftypes import resolve1
    with open(pdf, 'rb') as f:
        doc = PDFDocument(PDFParser(f))
        res = {}
        try: res['outline_entries'] = sum(1 for _ in doc.get_outlines())
        except Exception as e: res['outline_entries'] = 0; res['outline_error'] = type(e).__name__
        try: res['page_labels'] = sum(1 for _ in doc.get_page_labels())
        except Exception as e: res['page_labels'] = 0
        res['struct_tree_root'] = 'StructTreeRoot' in doc.catalog
        links = 0; n = 0
        for p in PDFPage.create_pages(doc):
            n += 1
            for a in resolve1(p.annots) or []:
                a = resolve1(a)
                if a and str(a.get('Subtype')) == "/'Link'": links += 1
        res['pages'] = n; res['link_annots'] = links
    return res

try:
    if mode == 'null':
        if eng == 'pdfium': import pypdfium2
        elif eng.startswith('mupdf'): import pymupdf
        elif eng == 'pdfminer': import pdfminer.high_level
        w({'ok': True}); sys.exit(0)
    if mode == 'text':
        base, _, variant = eng.partition(':')
        fn = {'pdfium': pdfium_text, 'mupdf': mupdf_text, 'pdfminer': pdfminer_text}[base]
        pages, meta = fn(variant)
        with open(out, 'w', encoding='utf-8') as f: f.write('\f'.join(pages))
        meta['ok'] = True; meta['pages_with_text'] = sum(1 for p in pages if p.strip())
        with open(out + '.json', 'w') as f: json.dump(meta, f)
    else:
        fn = {'pdfium': struct_pdfium, 'mupdf': struct_mupdf, 'pdfminer': struct_pdfminer}[eng]
        r = fn(); r['ok'] = True; w(r)
except Exception as e:
    err = {'ok': False, 'error': f'{type(e).__name__}: {e}'[:500], 'trace': traceback.format_exc()[-1500:]}
    with open(out + ('.json' if mode == 'text' else ''), 'w') as f: json.dump(err, f)
    sys.exit(3)
