#set document(
  title: "Hello Typst", 
  author: "Juno Takano", 
  date: auto,
  keywords: ("typst", "typesetting"),
)
#set page(paper: "a6", margin: (x: 0.8cm, y: 1cm), fill: rgb("#fffddf"))
#set heading(numbering: "1.")
#set par(justify: true, leading: 0.7em,)
#set quote(block: true, quotes: true)
#set footnote.entry(gap: 1em, clearance: 1em)
#show link: underline

= Typst
Typst is a typesetting system that takes code in and outputs PDFs.

This file is an example of several features you can use in it.

== Math notation
The first example Typst shows you is for writing the 
Fibonacci sequence's definition through its
recurrence relation $F_n = F_(n-1) + F_(n-2)$. That's inline math for you.

You can also do math on its own, centered paragraph:

$ F_n = round(1 / sqrt(5) phi.alt^n), quad
  phi.alt = (1 + sqrt(5)) / 2 $

== Code blocks
Typst also supports code blocks. The code for the previous formula, for instance, was:

#block( fill: luma(230), inset: 8pt, radius: 4pt, breakable: false)[
```typst
$ F_n = round(1 / sqrt(5) phi.alt^n), quad
  phi.alt = (1 + sqrt(5)) / 2 $
```]

== Code  mode
You can define and use code logic for Typst to evaluate on compile:

#block( fill: luma(230), inset: 8pt, radius: 4pt, breakable: false)[
```typst
#let count = 8
#let nums = range(1, count + 1)
#let fib(n) = (
  if n <= 2 { 1 }
  else { fib(n - 1) + fib(n - 2) }
)
```]

#let count = 8
#let nums = range(1, count + 1)
#let fib(n) = (
  if n <= 2 { 1 }
  else { fib(n - 1) + fib(n - 2) }
)

Using the `#count` and `#nums` values just set, we can render the following table:

#align(center, table(
  columns: count,
  ..nums.map(n => $F_#n$),
  ..nums.map(n => str(fib(n))),
))

== Formatting
This *bold text* is created using `*asterisks*`. _Italics_ are made using `_underlines_`.

- An unordered list
- with a few
- items uses hyphens 
- for markers

+ This numbered list 
+ uses instead
+ the ```typst +``` sign
+ for each item

#pagebreak()

== Quotes
There is also a `#quote` function:

#quote(attribution: [#link("https://typst.app/docs/tutorial/writing-in-typst/")[Typst Docs, _Writing in typst_]])[
  The caption consists of arbitrary markup. To give markup to 
  a function, we enclose it in square brackets. This construct 
  is called a content block.
]

== Footnotes
Speaking of quotes, footnotes append linked references at the end of the document.
#footnote[
  #" "#link("https://typst.app/docs/reference/meta/footnote/")[Typst reference, _footnote_]
]<footnote-1>

You can use ```typst #" "``` or #link("https://typst.app/docs/reference/layout/h/")[horizontal spacing] to adjust the distance between the superscript number and the text.
#footnote([#" "Though I'd rather use a parameter in `set footnote.entry()`. Also, they are a bit hard to click.])<footnote-2>

They can also be labeled so you may reference them multiple times. This line uses the same reference as the first footnote.
#footnote(<footnote-1>)

== A math lorem
$ 1.62 theta +
  sum_(i=0)^nabla R_n / "10p" arrow
  p := vec(x_1, y_2, z_3) arrow " ?" $
