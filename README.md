# typst-py

![CI](https://github.com/messense/typst-py/workflows/CI/badge.svg)
[![PyPI](https://img.shields.io/pypi/v/typst.svg)](https://pypi.org/project/typst)

Python binding to [typst](https://github.com/typst/typst),
a new markup-based typesetting system that is powerful and easy to learn.

## Installation

```bash
pip install typst
```

## Usage

```python
import typst


# Compile `hello.typ` to PDF and save as `hello.pdf`
typst.compile("hello.typ", output="hello.pdf")

# Compile `hello.typ` to PNG and save as `hello.png`
typst.compile("hello.typ", output="hello.png", format="png", ppi=144.0)

# Or return PDF content as bytes
pdf_bytes = typst.compile("hello.typ")

# Also for svg
svg_bytes = typst.compile("hello.typ", format="svg")

# Or use Compiler class to avoid reinitialization
compiler = typst.Compiler("hello.typ")
compiler.compile(format="png", ppi=144.0)
```

## License

This work is released under the Apache-2.0 license. A copy of the license is provided in the [LICENSE](./LICENSE) file.
