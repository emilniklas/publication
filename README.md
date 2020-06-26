# Publication

_The pluggable markup language._

Publication is a markup language that contains (almost) no syntax out of the box.
Instead, you opt in to syntax features by providing you own implementation
of what to do with it.

```publication
# This is a comment, one of the few syntax construct available in
# the language without any other configuration.

Here is a paragraph of text. It can wrap multiple lines as long as
there isn't a double line break.

Because now, after that double line break, we're in a new paragraph.
```

Comments and paragraphs are the only thing that exists in a plain
Publication document. It can be compiled to different output formats
by using the Publication Compiler, `publc`.

```shell
$ publc example.publ -o example.txt
```

If we take a look at that `example.txt`, here's what it contains:

```txt
Here is a paragraph of text. It can wrap multiple lines as long as there isn't a double line break.

Because now, after that double line break, we're in a new paragraph.
```

We can also output HTML by using the following command instead:

```shell
$ publc example.publ -o example.html
```

```html
<p>
  Here is a paragraph of text. It can wrap multiple lines as long as
  there isn&apos;t a double line break.
</p>
<p>
  Because now, after that double line break, we&apos;re in a new paragraph.
</p>
```
