# typst-py

![CI](https://github.com/messense/typst-py/workflows/CI/badge.svg)
[![PyPI](https://img.shields.io/pypi/v/typst.svg)](https://pypi.org/project/typst)
[![Documentation Status](https://readthedocs.org/projects/typst-py/badge/?version=latest)](https://typst-py.readthedocs.io/en/latest/?badge=latest)

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

# Or pass `hello.typ` content as bytes
with open("hello.typ", "rb") as f:
    typst.compile(f.read(), output="hello.pdf")

# Or return PDF content as bytes
pdf_bytes = typst.compile("hello.typ")

# Also for svg
svg_bytes = typst.compile("hello.typ", format="svg")

# For multi-page export (the template is the same as the typst cli)
images = typst.compile("hello.typ", output="hello{n}.png", format="png")

# Or use Compiler class to avoid reinitialization
compiler = typst.Compiler()
compiler.compile(input="hello.typ", format="png", ppi=144.0)

# Query something
import json

values = json.loads(typst.query("hello.typ", "<note>", field="value", one=True))
```

## Multi-file projects

Typst supports importing other files using `#import`. When working with multi-file projects in Python, you have several options:

### Option 1: Using a dictionary (recommended for bundled packages)

Pass a dictionary mapping filenames to their content (as bytes or paths):

```python
import typst

# Define your files as bytes
main_content = b'#import "lib.typ": greet\n= Hello\n#greet("World")'
lib_content = b'#let greet(name) = [Hello, #name!]'

files = {
    "main": main_content,        # Main file (key can be "main" or "main.typ")
    "lib.typ": lib_content,      # Imported file
}

# Compile the multi-file project
pdf = typst.compile(files, format="pdf")
```

This is especially useful when files are bundled as Python package resources:

```python
import typst
import importlib.resources

# Read files from your package
files = {}
for filename in ["main.typ", "lib.typ", "utils.typ"]:
    content = importlib.resources.read_binary("mypackage.typst_files", filename)
    files[filename] = content

# Compile the project
pdf = typst.compile(files, format="pdf")
```

### Option 2: Using temporary files for package resources

If you prefer working with actual file paths (for example when you need to use `importlib.resources.as_file`), write your resources into a shared temporary directory:

```python
import typst
import importlib.resources
import tempfile
from pathlib import Path

# For multiple files, write them all to the same temporary directory
with tempfile.TemporaryDirectory() as tmpdir:
    # Read each resource and write to temp directory
    for filename in ["main.typ", "lib.typ"]:
        content = importlib.resources.read_binary("mypackage.typst_files", filename)
        (Path(tmpdir) / filename).write_bytes(content)
    
    # Compile using the main file path
    main_path = Path(tmpdir) / "main.typ"
    pdf = typst.compile(str(main_path), format="pdf")
```

**Note:** When using `importlib.resources.as_file` on individual files, each file gets its own temporary directory, which prevents imports from working. Always use a shared temporary directory for multi-file projects.

### Option 3: Regular file paths

If your Typst files are regular files on disk:

```python
import typst

# Just compile the main file - imports will work automatically
pdf = typst.compile("path/to/main.typ", format="pdf")
```

## Passing values

You can pass values to the compiled Typst file with the `sys_inputs` argument. For example:

```python
import json
import typst

persons = [{"name": "John", "age": 35}, {"name": "Xoliswa", "age": 45}]
sys_inputs = {"persons": json.dumps(persons)}

typst.compile(input="main.typ", output="ages.pdf", sys_inputs=sys_inputs)
```

The following example shows how the passed data can be used in a Typst file.

```
#let persons = json(bytes(sys.inputs.persons))

#for person in persons [
  #person.name is #person.age years old. \
]
```

## License

This work is released under the Apache-2.0 license. A copy of the license is provided in the [LICENSE](./LICENSE) file.
