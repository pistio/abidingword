#!/usr/bin/env python3
"""F11: Η Αγία Γραφή - Neophytos Vamvas translation (1850), complete, as plain text.
Generator: this script. UTF-8 with byte-order mark, CRLF line ends, hard-wrapped
at 76 columns; book names in capitals between rules of '=' signs; chapters
headed 'ΚΕΦΑΛΑΙΟΝ' + Greek alphabetic numeral (the \\cp of the reference);
each chapter one paragraph with the verse numbers (Arabic digits) running
inline; a contents list at the top."""
import os, sys, textwrap, unicodedata
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import load_zip, clean_items, DOCS, short_name, NT

REF = os.path.join(DOCS, 'F11-ell-vamvas.usfm.zip')
OUT = os.path.join(DOCS, 'F11-ell-vamvas.txt')
W = 76


def caps(s):
    s = unicodedata.normalize('NFD', s.upper())
    s = ''.join(ch for ch in s if unicodedata.category(ch) != 'Mn')
    return unicodedata.normalize('NFC', s)


def main():
    books = load_zip(REF)
    L = []
    L += ['Η ΑΓΙΑ ΓΡΑΦΗ', '', 'ΗΤΟΙ', 'ΤΑ ΙΕΡΑ ΓΡΑΜΜΑΤΑ ΤΗΣ ΠΑΛΑΙΑΣ ΚΑΙ ΤΗΣ ΚΑΙΝΗΣ ΔΙΑΘΗΚΗΣ',
          'μεταφρασθέντα εκ του πρωτοτύπου υπό Νεοφύτου Βάμβα', '',
          '(Κείμενο κοινής χρήσης - public domain. Πηγή: scrollmapper/bible_databases, GreVamvas.)', '', '',
          'ΠΕΡΙΕΧΟΜΕΝΑ', '']
    for b in books:
        L.append('    ' + short_name(b))
    L += ['', '']
    for b in books:
        if b['code'] == 'GEN':
            L += ['', '*' * W, 'Η ΠΑΛΑΙΑ ΔΙΑΘΗΚΗ'.center(W).rstrip(), '*' * W, '', '']
        if b['code'] == 'MAT':
            L += ['', '*' * W, 'Η ΚΑΙΝΗ ΔΙΑΘΗΚΗ'.center(W).rstrip(), '*' * W, '', '']
        L += ['=' * W, caps(short_name(b)).center(W).rstrip(), '=' * W, '']
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                L += ['', 'ΚΕΦΑΛΑΙΟΝ ' + (bl['cp'] or bl['n']), '']
                continue
            if bl['kind'] != 'para':
                continue
            words = []
            for it in clean_items(bl['items']):
                if it[0] == 'v':
                    words.append(it[1] + ' ')
                elif it[0] == 't':
                    words.append(it[1] + ' ')
            text = ''.join(words).strip().replace('  ', ' ')
            wrapped = textwrap.wrap(text, W, break_long_words=False, break_on_hyphens=False)
            L += [w.replace(' ', ' ') for w in wrapped]
        L += ['', '']
    with open(OUT, 'w', encoding='utf-8-sig', newline='\r\n') as f:
        f.write('\n'.join(L) + '\n')
    print(OUT)


if __name__ == '__main__':
    main()
