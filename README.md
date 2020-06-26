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

## Extensions

Like mentioned previously, the syntax of Publication is very limited
out of the box. This is because, depending on your use-case, not every
kind of formatting makes sense. And enabling them all by default, like
for instance how the Markdown specification suggests, is not going
to be at all what you want in the majority of cases.

Consider, for instance, the careful introduction of new formatting
options in a rich text field; you wouldn't necessarily want your users
to have the full power of HTML at their fingertips. Instead, you
gradually add support for features that make sense, and have a good
implementation when rendered into whatever presentation format you use.

All features that have use-cases where they're _not_ useful, are instead
provided as an **extension**.

Extensions add syntactic structures that are otherwise treated as normal
text, and allows emitters to choose to output those fragments in
specialized ways.

### Built-in Extensions

| Name              | Configuration       | Syntax            | HTML Output                      | TXT Output      |
|:------------------|:--------------------|:------------------|:---------------------------------|:----------------|
| **Italics**       | `--italics` or `-i` | `That's /great/!` | `That's <em>great</em>!`         | `That's great!` |
| **Bold**          | `--bold` or `-b`    | `That's *great*!` | `That's <strong>great</strong>!` | `That's great!` |
