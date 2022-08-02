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
* metadata
* size
* `:term:definition` lists
* indention and hanging text
* text size
* subscript/superscript
* inserted/deleted (strikethrough)
## extensions
* footnotes
* citation needed
