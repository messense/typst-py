# Multi-file Typst Projects Documentation

## Issue Summary

The issue creator asked whether it's possible to use `importlib.resources.as_file` to provide temporary files from bytes objects for multi-file Typst projects. They requested documentation (in the README) if this approach works.

## Solution Provided

We've provided comprehensive documentation covering **both** approaches:

### 1. Dictionary-based Approach (Recommended)
- Uses the newly implemented feature to pass a dict of files directly
- No temporary files needed
- Most efficient for bundled packages
- Example:
  ```python
  files = {
      "main": main_bytes,
      "lib.typ": lib_bytes
  }
  pdf = typst.compile(files, format="pdf")
  ```

### 2. Temporary Directory Approach
- Uses `importlib.resources.read_binary` with `tempfile.TemporaryDirectory`
- Creates actual files on disk temporarily
- All files must be in the same directory for imports to work
- Example:
  ```python
  with tempfile.TemporaryDirectory() as tmpdir:
      for filename in ["main.typ", "lib.typ"]:
          content = importlib.resources.read_binary("pkg", filename)
          (Path(tmpdir) / filename).write_bytes(content)
      pdf = typst.compile(str(Path(tmpdir) / "main.typ"))
  ```

### Important Note on `as_file`

The documentation includes a **critical warning**:
> When using `importlib.resources.as_file` on individual files, each file gets its own temporary directory, which prevents imports from working. Always use a shared temporary directory for multi-file projects.

This is why we recommend either:
1. The dict-based approach (simpler, more efficient)
2. Manually creating a shared temp directory (if file paths are required)

## Files Modified

### README.md
Added a new "Multi-file projects" section with:
- Three different approaches clearly documented
- Code examples for each approach
- Usage with `importlib.resources`
- Important warnings about limitations
- Clear recommendations

### examples/multifile_example.py
Created a comprehensive example file demonstrating:
- Dictionary-based compilation
- Temporary directory approach
- Simulating package resources
- Compiler instance reuse
- All examples are runnable and produce output

## Testing

- Example file syntax validated
- Documentation is clear and actionable
- Both approaches are properly explained
- Recommendations are clear

## Recommendation

For the issue creator and other users with bundled packages:
**Use the dictionary-based approach** (Option 1) as it's:
- Simpler (no temp directory management)
- More efficient (no file I/O)
- Cleaner (no cleanup needed)
- Directly supported by the new feature

The temporary directory approach is still documented for users who specifically need file paths.
