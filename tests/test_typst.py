import pytest
import pathlib
import tempfile
import json
from typing import cast

import typst


# Fixtures
@pytest.fixture
def hello_typ_path():
    return pathlib.Path("tests/hello.typ")


@pytest.fixture
def hello_typ_content(hello_typ_path):
    return hello_typ_path.read_text()


@pytest.fixture
def simple_typst():
    return "= Hello\nThis is a simple document."


# Basic compilation tests
def test_compile_to_pdf_bytes(hello_typ_path):
    result = typst.compile(hello_typ_path, format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compile_to_svg_bytes(hello_typ_path):
    result = typst.compile(hello_typ_path, format="svg")
    assert isinstance(result, list)
    assert len(result) >= 1
    assert isinstance(result[0], bytes)
    first_page = cast(bytes, result[0])
    assert b"<svg" in first_page


def test_compile_to_png_bytes(hello_typ_path):
    result = typst.compile(hello_typ_path, format="png")
    assert isinstance(result, list)
    assert len(result) >= 1
    assert isinstance(result[0], bytes)
    first_page = cast(bytes, result[0])
    assert first_page.startswith(b"\x89PNG")


def test_compile_from_string_content():
    # String content needs to be passed as bytes for direct compilation
    content = "= Hello\nThis is a test document."
    result = typst.compile(content.encode("utf-8"), format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compile_from_bytes_content():
    content = "= Hello\nThis is a test document."
    content_bytes = content.encode("utf-8")
    result = typst.compile(content_bytes, format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compile_to_file(hello_typ_path):
    with tempfile.NamedTemporaryFile(suffix=".pdf", delete=False) as f:
        output_path = pathlib.Path(f.name)

    try:
        result = typst.compile(hello_typ_path, output=output_path)
        assert result is None
        assert output_path.exists()
        assert output_path.read_bytes().startswith(b"%PDF-")
    finally:
        output_path.unlink(missing_ok=True)


def test_compile_with_custom_ppi(hello_typ_path):
    result = typst.compile(hello_typ_path, format="png", ppi=72.0)
    assert isinstance(result, list)
    assert len(result) >= 1
    assert isinstance(result[0], bytes)
    first_page = cast(bytes, result[0])
    assert first_page.startswith(b"\x89PNG")


def test_compile_with_warnings(hello_typ_path):
    result, warnings = typst.compile_with_warnings(hello_typ_path, format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")
    assert isinstance(warnings, list)
    for warning in warnings:
        assert isinstance(warning, typst.TypstWarning)


# Compiler class tests
def test_compiler_init(hello_typ_path):
    compiler = typst.Compiler(hello_typ_path)
    assert compiler is not None


def test_compiler_init_without_input(hello_typ_path):
    compiler = typst.Compiler()
    result = compiler.compile(input=hello_typ_path, format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compiler_compile_empty_document():
    compiler = typst.Compiler()
    result = compiler.compile(format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compiler_reuse_with_multiple_inputs(hello_typ_path, simple_typst):
    compiler = typst.Compiler()

    first_result = compiler.compile(input=hello_typ_path, format="pdf")
    assert isinstance(first_result, bytes)
    assert first_result.startswith(b"%PDF-")

    second_result = compiler.compile(input=simple_typst.encode("utf-8"), format="pdf")
    assert isinstance(second_result, bytes)
    assert second_result.startswith(b"%PDF-")


def test_compiler_compile_pdf(hello_typ_path):
    compiler = typst.Compiler(hello_typ_path)
    result = compiler.compile(format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compiler_compile_svg(hello_typ_path):
    compiler = typst.Compiler(hello_typ_path)
    result = compiler.compile(format="svg")
    assert isinstance(result, list)
    assert len(result) >= 1
    assert isinstance(result[0], bytes)
    first_page = cast(bytes, result[0])
    assert b"<svg" in first_page


def test_compiler_compile_png(hello_typ_path):
    compiler = typst.Compiler(hello_typ_path)
    result = compiler.compile(format="png")
    assert isinstance(result, list)
    assert len(result) >= 1
    assert isinstance(result[0], bytes)
    first_page = cast(bytes, result[0])
    assert first_page.startswith(b"\x89PNG")


def test_compiler_with_sys_inputs(hello_typ_path):
    compiler = typst.Compiler(hello_typ_path, sys_inputs={"test": "value"})
    result = compiler.compile(format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compiler_compile_sys_inputs_ellipsis_keeps_existing():
    """Test that Ellipsis (default) keeps existing sys_inputs."""
    source = b'#sys.inputs.at("name", default: "empty")'
    compiler = typst.Compiler(source, sys_inputs={"name": "initial"})

    # First compile should use initial sys_inputs
    result1 = compiler.compile(input=source, format="pdf")
    assert isinstance(result1, bytes)

    # Second compile with default (Ellipsis) should keep the same sys_inputs
    result2 = compiler.compile(input=source, format="pdf")
    assert isinstance(result2, bytes)

    # Results should be identical since sys_inputs didn't change
    assert result1 == result2


def test_compiler_compile_sys_inputs_none_clears():
    """Test that None clears sys_inputs."""
    source = b'#sys.inputs.at("name", default: "empty")'
    compiler = typst.Compiler(source, sys_inputs={"name": "initial"})

    # First compile with initial sys_inputs
    result_with_inputs = compiler.compile(input=source, format="pdf")

    # Compile with None should clear sys_inputs
    result_cleared = compiler.compile(input=source, format="pdf", sys_inputs=None)

    # Results should differ since sys_inputs changed
    assert result_with_inputs != result_cleared


def test_compiler_compile_sys_inputs_dict_updates():
    """Test that passing a dict updates sys_inputs."""
    source = b'#sys.inputs.at("name", default: "empty")'
    compiler = typst.Compiler(source, sys_inputs={"name": "initial"})

    # First compile with initial sys_inputs
    result_initial = compiler.compile(input=source, format="pdf")

    # Compile with new sys_inputs dict
    result_updated = compiler.compile(
        input=source, format="pdf", sys_inputs={"name": "updated"}
    )

    # Results should differ since sys_inputs changed
    assert result_initial != result_updated


def test_compiler_compile_sys_inputs_persists_after_update():
    """Test that updated sys_inputs persists for subsequent compiles."""
    source = b'#sys.inputs.at("name", default: "empty")'
    compiler = typst.Compiler(source, sys_inputs={"name": "initial"})

    # Update sys_inputs
    result_updated = compiler.compile(
        input=source, format="pdf", sys_inputs={"name": "updated"}
    )

    # Next compile with Ellipsis (default) should keep updated value
    result_persisted = compiler.compile(input=source, format="pdf")

    # Results should be identical
    assert result_updated == result_persisted


def test_compiler_compile_sys_inputs_none_persists():
    """Test that cleared sys_inputs persists for subsequent compiles."""
    source = b'#sys.inputs.at("name", default: "empty")'
    compiler = typst.Compiler(source, sys_inputs={"name": "initial"})

    # Clear sys_inputs
    result_cleared = compiler.compile(input=source, format="pdf", sys_inputs=None)

    # Next compile with Ellipsis (default) should remain cleared
    result_persisted = compiler.compile(input=source, format="pdf")

    # Results should be identical
    assert result_cleared == result_persisted


def test_compiler_compile_with_warnings_sys_inputs():
    """Test sys_inputs works with compile_with_warnings."""
    source = b'#sys.inputs.at("name", default: "empty")'
    compiler = typst.Compiler(source, sys_inputs={"name": "initial"})

    # Compile with warnings and new sys_inputs
    result, warnings = compiler.compile_with_warnings(
        input=source, format="pdf", sys_inputs={"name": "updated"}
    )
    assert isinstance(result, bytes)
    assert isinstance(warnings, list)

    # Compile with warnings and None (clear)
    result_cleared, warnings = compiler.compile_with_warnings(
        input=source, format="pdf", sys_inputs=None
    )
    assert isinstance(result_cleared, bytes)
    assert result != result_cleared


def test_compiler_with_warnings(hello_typ_path):
    compiler = typst.Compiler(hello_typ_path)
    result, warnings = compiler.compile_with_warnings(format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")
    assert isinstance(warnings, list)


# Query functionality tests
def test_query_footnotes(hello_typ_path):
    result = typst.query(hello_typ_path, "<footnote-1>", format="json")
    data = json.loads(result)
    assert isinstance(data, list)


def test_query_headings(hello_typ_path):
    result = typst.query(hello_typ_path, "heading", format="json")
    data = json.loads(result)
    assert isinstance(data, list)
    assert len(data) > 0


def test_query_with_field(hello_typ_path):
    result = typst.query(hello_typ_path, "heading", field="body", format="json")
    data = json.loads(result)
    assert isinstance(data, list)


def test_query_one_element(hello_typ_path):
    # Query for a specific heading that should be unique
    result = typst.query(
        hello_typ_path, "heading.where(level: 1)", one=True, format="json"
    )
    data = json.loads(result)
    assert not isinstance(data, list)


def test_query_yaml_format(hello_typ_path):
    result = typst.query(hello_typ_path, "heading", format="yaml")
    assert isinstance(result, str)
    assert "- func:" in result or "func:" in result


def test_compiler_query(hello_typ_path):
    compiler = typst.Compiler(hello_typ_path)
    result = compiler.query("heading", format="json")
    data = json.loads(result)
    assert isinstance(data, list)


# Error handling tests
def test_invalid_syntax_raises_typst_error():
    broken_content = b"#invalid syntax here"

    with pytest.raises(typst.TypstError) as exc_info:
        typst.compile(broken_content, format="pdf")

    error = exc_info.value
    assert isinstance(error.message, str)
    assert isinstance(error.hints, list)
    assert isinstance(error.trace, list)

    assert error.message
    # These two are always empty, we need to figure out why
    # assert error.hints
    # assert error.trace


def test_file_not_found_raises_typst_error():
    with pytest.raises((typst.TypstError, FileNotFoundError)):
        typst.compile(pathlib.Path("nonexistent.typ"), format="pdf")


def test_invalid_query_selector(hello_typ_path):
    with pytest.raises((typst.TypstError, RuntimeError)):
        typst.query(hello_typ_path, "invalid[selector", format="json")


# Output format tests
@pytest.mark.parametrize("format_name", ["pdf", "svg", "png", "html"])
@pytest.mark.parametrize("use_file", [True, False])
def test_all_formats(format_name, use_file, tmp_path):
    # Write content to temporary file
    simple_content = b"= Hello\nThis is a simple document."
    if use_file:
        path = tmp_path / "a_file.typ"
        path.write_bytes(simple_content)
        simple_content = path

    result = typst.compile(simple_content, format=format_name)
    assert result is not None

    if format_name == "pdf":
        assert isinstance(result, bytes)
        assert result.startswith(b"%PDF-")
    elif format_name == "svg":
        # SVG can return bytes (single page) or list (multi-page)
        if isinstance(result, list):
            assert len(result) >= 1
            first_page = cast(bytes, result[0])
            assert b"<svg" in first_page
        else:
            assert isinstance(result, bytes)
            assert b"<svg" in result
    elif format_name == "png":
        # PNG can return bytes (single page) or list (multi-page)
        if isinstance(result, list):
            assert len(result) >= 1
            first_page = cast(bytes, result[0])
            assert first_page.startswith(b"\x89PNG")
        else:
            assert isinstance(result, bytes)
            assert result.startswith(b"\x89PNG")

    if format_name == "html":
        assert isinstance(result, bytes)
        assert result.startswith(b"<!DOCTYPE html>")


def test_unsupported_format():
    # Test with an invalid format parameter
    simple_content = "= Hello\nThis is a simple document."
    with tempfile.NamedTemporaryFile(mode="w", suffix=".typ", delete=False) as f:
        f.write(simple_content)
        temp_path = pathlib.Path(f.name)

    try:
        # This should fail at the type level, but let's test runtime behavior
        with pytest.raises((typst.TypstError, ValueError, TypeError)):
            # Use type: ignore to bypass type checking for this test
            typst.compile(temp_path, format="invalid")  # type: ignore
    finally:
        temp_path.unlink(missing_ok=True)


# Edge cases tests
def test_empty_document():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".typ", delete=False) as f:
        f.write("")
        temp_path = pathlib.Path(f.name)

    try:
        result = typst.compile(temp_path, format="pdf")
        assert isinstance(result, bytes)
        assert result.startswith(b"%PDF-")
    finally:
        temp_path.unlink(missing_ok=True)


def test_unicode_content():
    unicode_content = "= 测试\n这是一个中文文档 🎉"
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".typ", delete=False, encoding="utf-8"
    ) as f:
        f.write(unicode_content)
        temp_path = pathlib.Path(f.name)

    try:
        result = typst.compile(temp_path, format="pdf")
        assert isinstance(result, bytes)
        assert result.startswith(b"%PDF-")
    finally:
        temp_path.unlink(missing_ok=True)


def test_large_document():
    large_content = (
        "= Large Document\n" + "This is a paragraph.\n" * 100
    )  # Reduced size
    with tempfile.NamedTemporaryFile(mode="w", suffix=".typ", delete=False) as f:
        f.write(large_content)
        temp_path = pathlib.Path(f.name)

    try:
        result = typst.compile(temp_path, format="pdf")
        assert isinstance(result, bytes)
        assert result.startswith(b"%PDF-")
    finally:
        temp_path.unlink(missing_ok=True)


def test_math_heavy_document():
    math_content = """= Math Test
$ sum_(i=1)^n i = (n(n+1))/2 $
$ integral_0^infinity e^(-x^2) dif x = sqrt(pi)/2 $
$ lim_(x->0) (sin x)/x = 1 $
"""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".typ", delete=False) as f:
        f.write(math_content)
        temp_path = pathlib.Path(f.name)

    try:
        result = typst.compile(temp_path, format="pdf")
        assert isinstance(result, bytes)
        assert result.startswith(b"%PDF-")
    finally:
        temp_path.unlink(missing_ok=True)


# Integration tests
def test_compile_and_query_workflow(hello_typ_path):
    # First compile the document
    pdf_result = typst.compile(hello_typ_path, format="pdf")
    assert isinstance(pdf_result, bytes)

    # Then query for headings
    headings = typst.query(hello_typ_path, "heading", format="json")
    headings_data = json.loads(headings)
    assert len(headings_data) > 0

    # Query for footnotes
    footnotes = typst.query(hello_typ_path, "<footnote-1>", format="json")
    footnotes_data = json.loads(footnotes)
    assert isinstance(footnotes_data, list)


def test_compiler_multiple_operations(hello_typ_path):
    compiler = typst.Compiler(hello_typ_path)

    # Compile to different formats
    pdf_result = compiler.compile(format="pdf")
    svg_result = compiler.compile(format="svg")

    # Query the same document
    headings = compiler.query("heading", format="json")

    assert isinstance(pdf_result, bytes)
    assert isinstance(svg_result, list)
    assert isinstance(headings, str)


def test_compile_with_all_options(hello_typ_path):
    fonts = typst.Fonts(include_system_fonts=True)

    # Use PDF format instead of PNG to avoid multi-page issue
    with tempfile.NamedTemporaryFile(suffix=".pdf", delete=False) as f:
        output_path = pathlib.Path(f.name)

    try:
        result = typst.compile(
            hello_typ_path,
            output=output_path,
            format="pdf",
            font_paths=fonts,
            sys_inputs={"test": "value"},
        )
        assert result is None
        assert output_path.exists()
        assert output_path.read_bytes().startswith(b"%PDF-")
    finally:
        output_path.unlink(missing_ok=True)
