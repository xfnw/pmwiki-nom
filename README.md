# pmwiki-nom
nom parser for a subset of pmwiki-like markup language

note that this deviates from pmwiki's image formatting since
pmwiki's image format is inconsistent and weird

status:
```
image_tests ... ok
linebreak_tests ... ok
link_tests ... ok
heading_tests ... ok
mixed_tests ... ok
other_tests ... ok
table_tests ... FAILED
parser_tests ... ok
text_style_tests ... ok
text_tests ... ok
list_tests ... ok
```

# todo
## pmwiki
* metadata `(:title stuff:)`
* text size `[+big+] [++bigger++] [-small-] [--smaller--]`
* `:term:definition` lists
* indention and hanging text `-> indented -< hanging`
* subscript/superscript ` '^superscript^' '_subscript_'`
* inserted/deleted (strikethrough) `{+inserted+} {-deleted-}`
## extensions
* footnotes/citations `[^im a footnote^] references: [^#^]`
* citation needed `{{cn}} {{cn|date=2022-08-02}} {{cn|date=2022-08-02|reason=a good reason}}`
