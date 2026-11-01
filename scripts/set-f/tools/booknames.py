# Book codes for the scrollmapper/bible_databases English book names, and the
# book titles set F prints for sources that carry no book titles of their own.
# (The titles are standard printed titles in each language, supplied by the
# test builder; they appear as \h / \toc / \mt in the reference USFM.)

SCROLL_CODES = {
    'Genesis': 'GEN', 'Exodus': 'EXO', 'Leviticus': 'LEV', 'Numbers': 'NUM', 'Deuteronomy': 'DEU',
    'Joshua': 'JOS', 'Judges': 'JDG', 'Ruth': 'RUT', 'I Samuel': '1SA', 'II Samuel': '2SA',
    'I Kings': '1KI', 'II Kings': '2KI', 'I Chronicles': '1CH', 'II Chronicles': '2CH', 'Ezra': 'EZR',
    'Nehemiah': 'NEH', 'Esther': 'EST', 'Job': 'JOB', 'Psalms': 'PSA', 'Proverbs': 'PRO',
    'Ecclesiastes': 'ECC', 'Song of Solomon': 'SNG', 'Isaiah': 'ISA', 'Jeremiah': 'JER',
    'Lamentations': 'LAM', 'Ezekiel': 'EZK', 'Daniel': 'DAN', 'Hosea': 'HOS', 'Joel': 'JOL',
    'Amos': 'AMO', 'Obadiah': 'OBA', 'Jonah': 'JON', 'Micah': 'MIC', 'Nahum': 'NAM', 'Habakkuk': 'HAB',
    'Zephaniah': 'ZEP', 'Haggai': 'HAG', 'Zechariah': 'ZEC', 'Malachi': 'MAL',
    'Matthew': 'MAT', 'Mark': 'MRK', 'Luke': 'LUK', 'John': 'JHN', 'Acts': 'ACT', 'Romans': 'ROM',
    'I Corinthians': '1CO', 'II Corinthians': '2CO', 'Galatians': 'GAL', 'Ephesians': 'EPH',
    'Philippians': 'PHP', 'Colossians': 'COL', 'I Thessalonians': '1TH', 'II Thessalonians': '2TH',
    'I Timothy': '1TI', 'II Timothy': '2TI', 'Titus': 'TIT', 'Philemon': 'PHM', 'Hebrews': 'HEB',
    'James': 'JAS', 'I Peter': '1PE', 'II Peter': '2PE', 'I John': '1JN', 'II John': '2JN',
    'III John': '3JN', 'Jude': 'JUD', 'Revelation of John': 'REV',
    'Judith': 'JDT', 'Wisdom': 'WIS', 'Tobit': 'TOB', 'Sirach': 'SIR', 'Baruch': 'BAR',
    'I Maccabees': '1MA', 'II Maccabees': '2MA', 'Additions to Esther': 'ESG',
    'Additions to Daniel': 'DAG', 'Prayer of Manasses': 'MAN', 'I Esdras': '1ES', 'II Esdras': '2ES',
    'Additional Psalm': 'PS2', 'Laodiceans': 'LAO',
}

OT = 'GEN EXO LEV NUM DEU JOS JDG RUT 1SA 2SA 1KI 2KI 1CH 2CH EZR NEH EST JOB PSA PRO ECC SNG ISA JER LAM EZK DAN HOS JOL AMO OBA JON MIC NAM HAB ZEP HAG ZEC MAL'.split()
NT = 'MAT MRK LUK JHN ACT ROM 1CO 2CO GAL EPH PHP COL 1TH 2TH 1TI 2TI TIT PHM HEB JAS 1PE 2PE 1JN 2JN 3JN JUD REV'.split()
DC = 'TOB JDT ESG WIS SIR BAR LJE S3Y SUS BEL 1MA 2MA 3MA 4MA 1ES 2ES MAN PS2 DAG LAO'.split()

def _t(s):
    return s.strip().split('\n')

# (short head title, full title)
NAMES = {}

NAMES['eng'] = dict(zip(OT + NT, [
    ('Genesis', 'The First Book of Moses, Called Genesis'), ('Exodus', 'The Second Book of Moses, Called Exodus'),
    ('Leviticus', 'The Third Book of Moses, Called Leviticus'), ('Numbers', 'The Fourth Book of Moses, Called Numbers'),
    ('Deuteronomy', 'The Fifth Book of Moses, Called Deuteronomy'), ('Joshua', 'The Book of Joshua'),
    ('Judges', 'The Book of Judges'), ('Ruth', 'The Book of Ruth'), ('1 Samuel', 'The First Book of Samuel'),
    ('2 Samuel', 'The Second Book of Samuel'), ('1 Kings', 'The First Book of Kings'), ('2 Kings', 'The Second Book of Kings'),
    ('1 Chronicles', 'The First Book of Chronicles'), ('2 Chronicles', 'The Second Book of Chronicles'),
    ('Ezra', 'The Book of Ezra'), ('Nehemiah', 'The Book of Nehemiah'), ('Esther', 'The Book of Esther'),
    ('Job', 'The Book of Job'), ('Psalms', 'The Psalms'), ('Proverbs', 'The Proverbs'), ('Ecclesiastes', 'Ecclesiastes'),
    ('Song of Songs', 'The Song of Songs'), ('Isaiah', 'The Book of Isaiah'), ('Jeremiah', 'The Book of Jeremiah'),
    ('Lamentations', 'The Lamentations of Jeremiah'), ('Ezekiel', 'The Book of Ezekiel'), ('Daniel', 'The Book of Daniel'),
    ('Hosea', 'Hosea'), ('Joel', 'Joel'), ('Amos', 'Amos'), ('Obadiah', 'Obadiah'), ('Jonah', 'Jonah'), ('Micah', 'Micah'),
    ('Nahum', 'Nahum'), ('Habakkuk', 'Habakkuk'), ('Zephaniah', 'Zephaniah'), ('Haggai', 'Haggai'),
    ('Zechariah', 'Zechariah'), ('Malachi', 'Malachi'),
    ('Matthew', 'The Gospel According to Matthew'), ('Mark', 'The Gospel According to Mark'),
    ('Luke', 'The Gospel According to Luke'), ('John', 'The Gospel According to John'), ('Acts', 'The Acts of the Apostles'),
    ('Romans', 'The Letter to the Romans'), ('1 Corinthians', 'The First Letter to the Corinthians'),
    ('2 Corinthians', 'The Second Letter to the Corinthians'), ('Galatians', 'The Letter to the Galatians'),
    ('Ephesians', 'The Letter to the Ephesians'), ('Philippians', 'The Letter to the Philippians'),
    ('Colossians', 'The Letter to the Colossians'), ('1 Thessalonians', 'The First Letter to the Thessalonians'),
    ('2 Thessalonians', 'The Second Letter to the Thessalonians'), ('1 Timothy', 'The First Letter to Timothy'),
    ('2 Timothy', 'The Second Letter to Timothy'), ('Titus', 'The Letter to Titus'), ('Philemon', 'The Letter to Philemon'),
    ('Hebrews', 'The Letter to the Hebrews'), ('James', 'The Letter of James'), ('1 Peter', 'The First Letter of Peter'),
    ('2 Peter', 'The Second Letter of Peter'), ('1 John', 'The First Letter of John'), ('2 John', 'The Second Letter of John'),
    ('3 John', 'The Third Letter of John'), ('Jude', 'The Letter of Jude'), ('Revelation', 'The Revelation to John'),
]))

_deu_ot = [('1. Mose', 'Das erste Buch Mose (Genesis)'), ('2. Mose', 'Das zweite Buch Mose (Exodus)'),
           ('3. Mose', 'Das dritte Buch Mose (Levitikus)'), ('4. Mose', 'Das vierte Buch Mose (Numeri)'),
           ('5. Mose', 'Das fünfte Buch Mose (Deuteronomium)'), ('Josua', 'Das Buch Josua'), ('Richter', 'Das Buch der Richter'),
           ('Rut', 'Das Buch Rut'), ('1. Samuel', 'Das erste Buch Samuel'), ('2. Samuel', 'Das zweite Buch Samuel'),
           ('1. Könige', 'Das erste Buch der Könige'), ('2. Könige', 'Das zweite Buch der Könige'),
           ('1. Chronik', 'Das erste Buch der Chronik'), ('2. Chronik', 'Das zweite Buch der Chronik'), ('Esra', 'Das Buch Esra'),
           ('Nehemia', 'Das Buch Nehemia'), ('Ester', 'Das Buch Ester'), ('Hiob', 'Das Buch Hiob'), ('Psalmen', 'Die Psalmen'),
           ('Sprüche', 'Die Sprüche Salomos'), ('Prediger', 'Der Prediger Salomo'), ('Hoheslied', 'Das Hohelied Salomos'),
           ('Jesaja', 'Der Prophet Jesaja'), ('Jeremia', 'Der Prophet Jeremia'), ('Klagelieder', 'Die Klagelieder Jeremias'),
           ('Hesekiel', 'Der Prophet Hesekiel'), ('Daniel', 'Der Prophet Daniel'), ('Hosea', 'Der Prophet Hosea'),
           ('Joel', 'Der Prophet Joel'), ('Amos', 'Der Prophet Amos'), ('Obadja', 'Der Prophet Obadja'), ('Jona', 'Der Prophet Jona'),
           ('Micha', 'Der Prophet Micha'), ('Nahum', 'Der Prophet Nahum'), ('Habakuk', 'Der Prophet Habakuk'),
           ('Zephanja', 'Der Prophet Zephanja'), ('Haggai', 'Der Prophet Haggai'), ('Sacharja', 'Der Prophet Sacharja'),
           ('Maleachi', 'Der Prophet Maleachi')]
_deu_nt = [('Matthäus', 'Das Evangelium nach Matthäus'), ('Markus', 'Das Evangelium nach Markus'),
           ('Lukas', 'Das Evangelium nach Lukas'), ('Johannes', 'Das Evangelium nach Johannes'),
           ('Apostelgeschichte', 'Die Apostelgeschichte'), ('Römer', 'Der Brief des Paulus an die Römer'),
           ('1. Korinther', 'Der erste Brief des Paulus an die Korinther'), ('2. Korinther', 'Der zweite Brief des Paulus an die Korinther'),
           ('Galater', 'Der Brief des Paulus an die Galater'), ('Epheser', 'Der Brief des Paulus an die Epheser'),
           ('Philipper', 'Der Brief des Paulus an die Philipper'), ('Kolosser', 'Der Brief des Paulus an die Kolosser'),
           ('1. Thessalonicher', 'Der erste Brief des Paulus an die Thessalonicher'),
           ('2. Thessalonicher', 'Der zweite Brief des Paulus an die Thessalonicher'),
           ('1. Timotheus', 'Der erste Brief des Paulus an Timotheus'), ('2. Timotheus', 'Der zweite Brief des Paulus an Timotheus'),
           ('Titus', 'Der Brief des Paulus an Titus'), ('Philemon', 'Der Brief des Paulus an Philemon'),
           ('Hebräer', 'Der Brief an die Hebräer'), ('Jakobus', 'Der Brief des Jakobus'), ('1. Petrus', 'Der erste Brief des Petrus'),
           ('2. Petrus', 'Der zweite Brief des Petrus'), ('1. Johannes', 'Der erste Brief des Johannes'),
           ('2. Johannes', 'Der zweite Brief des Johannes'), ('3. Johannes', 'Der dritte Brief des Johannes'),
           ('Judas', 'Der Brief des Judas'), ('Offenbarung', 'Die Offenbarung des Johannes')]
NAMES['deu'] = dict(zip(OT + NT, _deu_ot + _deu_nt))
NAMES['deu'].update({
    'JDT': ('Judit', 'Das Buch Judit'), 'WIS': ('Weisheit', 'Die Weisheit Salomos'), 'TOB': ('Tobias', 'Das Buch Tobias'),
    'SIR': ('Sirach', 'Das Buch Jesus Sirach'), 'BAR': ('Baruch', 'Das Buch Baruch'),
    '1MA': ('1. Makkabäer', 'Das erste Buch der Makkabäer'), '2MA': ('2. Makkabäer', 'Das zweite Buch der Makkabäer'),
    'ESG': ('Stücke zu Ester', 'Stücke zu Ester'), 'DAG': ('Stücke zu Daniel', 'Stücke zu Daniel'),
    'MAN': ('Gebet Manasses', 'Das Gebet Manasses'),
})

NAMES['fra'] = dict(zip(OT + NT, [(a, b) for a, b in [
    ('Genèse', 'La Genèse'), ('Exode', "L'Exode"), ('Lévitique', 'Le Lévitique'), ('Nombres', 'Les Nombres'),
    ('Deutéronome', 'Le Deutéronome'), ('Josué', 'Josué'), ('Juges', 'Les Juges'), ('Ruth', 'Ruth'),
    ('1 Samuel', 'Premier livre de Samuel'), ('2 Samuel', 'Second livre de Samuel'), ('1 Rois', 'Premier livre des Rois'),
    ('2 Rois', 'Second livre des Rois'), ('1 Chroniques', 'Premier livre des Chroniques'),
    ('2 Chroniques', 'Second livre des Chroniques'), ('Esdras', 'Esdras'), ('Néhémie', 'Néhémie'), ('Esther', 'Esther'),
    ('Job', 'Job'), ('Psaumes', 'Les Psaumes'), ('Proverbes', 'Les Proverbes'), ('Ecclésiaste', "L'Ecclésiaste"),
    ('Cantique', 'Le Cantique des cantiques'), ('Ésaïe', 'Ésaïe'), ('Jérémie', 'Jérémie'), ('Lamentations', 'Les Lamentations'),
    ('Ézéchiel', 'Ézéchiel'), ('Daniel', 'Daniel'), ('Osée', 'Osée'), ('Joël', 'Joël'), ('Amos', 'Amos'), ('Abdias', 'Abdias'),
    ('Jonas', 'Jonas'), ('Michée', 'Michée'), ('Nahum', 'Nahum'), ('Habakuk', 'Habakuk'), ('Sophonie', 'Sophonie'),
    ('Aggée', 'Aggée'), ('Zacharie', 'Zacharie'), ('Malachie', 'Malachie'),
    ('Matthieu', 'Évangile selon saint Matthieu'), ('Marc', 'Évangile selon saint Marc'), ('Luc', 'Évangile selon saint Luc'),
    ('Jean', 'Évangile selon saint Jean'), ('Actes', 'Les Actes des Apôtres'), ('Romains', 'Épître aux Romains'),
    ('1 Corinthiens', 'Première épître aux Corinthiens'), ('2 Corinthiens', 'Seconde épître aux Corinthiens'),
    ('Galates', 'Épître aux Galates'), ('Éphésiens', 'Épître aux Éphésiens'), ('Philippiens', 'Épître aux Philippiens'),
    ('Colossiens', 'Épître aux Colossiens'), ('1 Thessaloniciens', 'Première épître aux Thessaloniciens'),
    ('2 Thessaloniciens', 'Seconde épître aux Thessaloniciens'), ('1 Timothée', 'Première épître à Timothée'),
    ('2 Timothée', 'Seconde épître à Timothée'), ('Tite', 'Épître à Tite'), ('Philémon', 'Épître à Philémon'),
    ('Hébreux', 'Épître aux Hébreux'), ('Jacques', 'Épître de saint Jacques'), ('1 Pierre', 'Première épître de saint Pierre'),
    ('2 Pierre', 'Seconde épître de saint Pierre'), ('1 Jean', 'Première épître de saint Jean'),
    ('2 Jean', 'Seconde épître de saint Jean'), ('3 Jean', 'Troisième épître de saint Jean'), ('Jude', 'Épître de saint Jude'),
    ('Apocalypse', 'Apocalypse de saint Jean')]]))

NAMES['spa'] = dict(zip(OT + NT, [(a, b) for a, b in [
    ('Génesis', 'El Primer Libro de Moisés: Génesis'), ('Éxodo', 'El Segundo Libro de Moisés: Éxodo'),
    ('Levítico', 'El Tercer Libro de Moisés: Levítico'), ('Números', 'El Cuarto Libro de Moisés: Números'),
    ('Deuteronomio', 'El Quinto Libro de Moisés: Deuteronomio'), ('Josué', 'El Libro de Josué'), ('Jueces', 'El Libro de los Jueces'),
    ('Rut', 'El Libro de Rut'), ('1 Samuel', 'El Primer Libro de Samuel'), ('2 Samuel', 'El Segundo Libro de Samuel'),
    ('1 Reyes', 'El Primer Libro de los Reyes'), ('2 Reyes', 'El Segundo Libro de los Reyes'),
    ('1 Crónicas', 'El Primer Libro de las Crónicas'), ('2 Crónicas', 'El Segundo Libro de las Crónicas'),
    ('Esdras', 'El Libro de Esdras'), ('Nehemías', 'El Libro de Nehemías'), ('Ester', 'El Libro de Ester'),
    ('Job', 'El Libro de Job'), ('Salmos', 'El Libro de los Salmos'), ('Proverbios', 'Los Proverbios'),
    ('Eclesiastés', 'El Eclesiastés'), ('Cantares', 'El Cantar de los Cantares'), ('Isaías', 'El Profeta Isaías'),
    ('Jeremías', 'El Profeta Jeremías'), ('Lamentaciones', 'Las Lamentaciones de Jeremías'), ('Ezequiel', 'El Profeta Ezequiel'),
    ('Daniel', 'El Profeta Daniel'), ('Oseas', 'Oseas'), ('Joel', 'Joel'), ('Amós', 'Amós'), ('Abdías', 'Abdías'),
    ('Jonás', 'Jonás'), ('Miqueas', 'Miqueas'), ('Nahum', 'Nahum'), ('Habacuc', 'Habacuc'), ('Sofonías', 'Sofonías'),
    ('Hageo', 'Hageo'), ('Zacarías', 'Zacarías'), ('Malaquías', 'Malaquías'),
    ('San Mateo', 'El Santo Evangelio según San Mateo'), ('San Marcos', 'El Santo Evangelio según San Marcos'),
    ('San Lucas', 'El Santo Evangelio según San Lucas'), ('San Juan', 'El Santo Evangelio según San Juan'),
    ('Hechos', 'Los Hechos de los Apóstoles'), ('Romanos', 'La Epístola a los Romanos'),
    ('1 Corintios', 'La Primera Epístola a los Corintios'), ('2 Corintios', 'La Segunda Epístola a los Corintios'),
    ('Gálatas', 'La Epístola a los Gálatas'), ('Efesios', 'La Epístola a los Efesios'), ('Filipenses', 'La Epístola a los Filipenses'),
    ('Colosenses', 'La Epístola a los Colosenses'), ('1 Tesalonicenses', 'La Primera Epístola a los Tesalonicenses'),
    ('2 Tesalonicenses', 'La Segunda Epístola a los Tesalonicenses'), ('1 Timoteo', 'La Primera Epístola a Timoteo'),
    ('2 Timoteo', 'La Segunda Epístola a Timoteo'), ('Tito', 'La Epístola a Tito'), ('Filemón', 'La Epístola a Filemón'),
    ('Hebreos', 'La Epístola a los Hebreos'), ('Santiago', 'La Epístola Universal de Santiago'),
    ('1 San Pedro', 'La Primera Epístola Universal de San Pedro'), ('2 San Pedro', 'La Segunda Epístola Universal de San Pedro'),
    ('1 San Juan', 'La Primera Epístola Universal de San Juan'), ('2 San Juan', 'La Segunda Epístola de San Juan'),
    ('3 San Juan', 'La Tercera Epístola de San Juan'), ('San Judas', 'La Epístola Universal de San Judas'),
    ('Apocalipsis', 'El Apocalipsis o Revelación de San Juan')]]))

NAMES['por'] = dict(zip(OT + NT, [(a, a) for a in [
    'Gênesis', 'Êxodo', 'Levítico', 'Números', 'Deuteronômio', 'Josué', 'Juízes', 'Rute', '1 Samuel', '2 Samuel', '1 Reis', '2 Reis',
    '1 Crônicas', '2 Crônicas', 'Esdras', 'Neemias', 'Ester', 'Jó', 'Salmos', 'Provérbios', 'Eclesiastes', 'Cânticos', 'Isaías',
    'Jeremias', 'Lamentações', 'Ezequiel', 'Daniel', 'Oseias', 'Joel', 'Amós', 'Obadias', 'Jonas', 'Miqueias', 'Naum',
    'Habacuque', 'Sofonias', 'Ageu', 'Zacarias', 'Malaquias',
    'Mateus', 'Marcos', 'Lucas', 'João', 'Atos', 'Romanos', '1 Coríntios', '2 Coríntios', 'Gálatas', 'Efésios', 'Filipenses',
    'Colossenses', '1 Tessalonicenses', '2 Tessalonicenses', '1 Timóteo', '2 Timóteo', 'Tito', 'Filemom', 'Hebreus', 'Tiago',
    '1 Pedro', '2 Pedro', '1 João', '2 João', '3 João', 'Judas', 'Apocalipse']]))

NAMES['kor'] = dict(zip(OT + NT, [(a, a) for a in [
    '창세기', '출애굽기', '레위기', '민수기', '신명기', '여호수아', '사사기', '룻기', '사무엘상', '사무엘하', '열왕기상', '열왕기하',
    '역대상', '역대하', '에스라', '느헤미야', '에스더', '욥기', '시편', '잠언', '전도서', '아가', '이사야', '예레미야', '예레미야애가',
    '에스겔', '다니엘', '호세아', '요엘', '아모스', '오바댜', '요나', '미가', '나훔', '하박국', '스바냐', '학개', '스가랴', '말라기',
    '마태복음', '마가복음', '누가복음', '요한복음', '사도행전', '로마서', '고린도전서', '고린도후서', '갈라디아서', '에베소서',
    '빌립보서', '골로새서', '데살로니가전서', '데살로니가후서', '디모데전서', '디모데후서', '디도서', '빌레몬서', '히브리서',
    '야고보서', '베드로전서', '베드로후서', '요한일서', '요한이서', '요한삼서', '유다서', '요한계시록']]))

NAMES['ell'] = dict(zip(OT + NT, [(a, a) for a in [
    'Γένεσις', 'Έξοδος', 'Λευϊτικόν', 'Αριθμοί', 'Δευτερονόμιον', 'Ιησούς του Ναυή', 'Κριταί', 'Ρούθ', 'Σαμουήλ Α΄', 'Σαμουήλ Β΄',
    'Βασιλέων Α΄', 'Βασιλέων Β΄', 'Χρονικών Α΄', 'Χρονικών Β΄', 'Έσδρας', 'Νεεμίας', 'Εσθήρ', 'Ιώβ', 'Ψαλμοί', 'Παροιμίαι',
    'Εκκλησιαστής', 'Άσμα Ασμάτων', 'Ησαΐας', 'Ιερεμίας', 'Θρήνοι', 'Ιεζεκιήλ', 'Δανιήλ', 'Ωσηέ', 'Ιωήλ', 'Αμώς', 'Οβδιού',
    'Ιωνάς', 'Μιχαίας', 'Ναούμ', 'Αββακούμ', 'Σοφονίας', 'Αγγαίος', 'Ζαχαρίας', 'Μαλαχίας',
    'Κατά Ματθαίον', 'Κατά Μάρκον', 'Κατά Λουκάν', 'Κατά Ιωάννην', 'Πράξεις των Αποστόλων', 'Προς Ρωμαίους',
    'Προς Κορινθίους Α΄', 'Προς Κορινθίους Β΄', 'Προς Γαλάτας', 'Προς Εφεσίους', 'Προς Φιλιππησίους', 'Προς Κολοσσαείς',
    'Προς Θεσσαλονικείς Α΄', 'Προς Θεσσαλονικείς Β΄', 'Προς Τιμόθεον Α΄', 'Προς Τιμόθεον Β΄', 'Προς Τίτον', 'Προς Φιλήμονα',
    'Προς Εβραίους', 'Ιακώβου', 'Πέτρου Α΄', 'Πέτρου Β΄', 'Ιωάννου Α΄', 'Ιωάννου Β΄', 'Ιωάννου Γ΄', 'Ιούδα', 'Αποκάλυψις Ιωάννου']]))

NAMES['lat'] = {
    'ISA': ('Isaias', 'Prophetia Isaiae'), 'JER': ('Ieremias', 'Prophetia Ieremiae'), 'LAM': ('Threni', 'Lamentationes Ieremiae Prophetae'),
    'BAR': ('Baruch', 'Prophetia Baruch'), 'EZK': ('Ezechiel', 'Prophetia Ezechielis'), 'DAN': ('Daniel', 'Prophetia Danielis'),
    'HOS': ('Osee', 'Prophetia Osee'), 'JOL': ('Ioel', 'Prophetia Ioel'), 'AMO': ('Amos', 'Prophetia Amos'),
    'OBA': ('Abdias', 'Prophetia Abdiae'), 'JON': ('Ionas', 'Prophetia Ionae'), 'MIC': ('Michaeas', 'Prophetia Michaeae'),
    'NAM': ('Nahum', 'Prophetia Nahum'), 'HAB': ('Habacuc', 'Prophetia Habacuc'), 'ZEP': ('Sophonias', 'Prophetia Sophoniae'),
    'HAG': ('Aggaeus', 'Prophetia Aggaei'), 'ZEC': ('Zacharias', 'Prophetia Zachariae'), 'MAL': ('Malachias', 'Prophetia Malachiae'),
}
