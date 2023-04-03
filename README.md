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


compiler = typst.Compiler(".")
compiler.compile("hello.typ")
```

## License

This work is released under the Apache-2.0 license. A copy of the license is provided in the [LICENSE](./LICENSE) file.
