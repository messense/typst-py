import pytest
import typst


def test_compile_from_dict_with_bytes():
    """Test compiling a multi-file project with bytes input."""
    main_content = b'#import "lib.typ": my_func\n= Hello\n#my_func()'
    lib_content = b'#let my_func() = [This is from the library!]'
    
    files = {
        "main": main_content,
        "lib.typ": lib_content,
    }
    
    result = typst.compile(files, format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compile_from_dict_without_main_key():
    """Test that the first file is used as main when 'main' key is not present."""
    # When there's no 'main' key, the first entry should be the main file
    # Note: dict order is preserved in Python 3.7+
    main_content = b'#import "helper.typ": helper\n= Document\n#helper()'
    helper_content = b'#let helper() = [Helper text]'
    
    files = {
        "document.typ": main_content,
        "helper.typ": helper_content,
    }
    
    result = typst.compile(files, format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compile_from_dict_single_file():
    """Test compiling with a dict containing a single file."""
    content = b'= Single File\nThis is a simple document.'
    
    files = {"main": content}
    
    result = typst.compile(files, format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compile_from_dict_multiple_imports():
    """Test compiling with multiple import dependencies."""
    main_content = b'''
#import "utils.typ": format_title
#import "data.typ": get_data

#format_title("My Document")

#get_data()
'''
    utils_content = b'#let format_title(title) = text(size: 20pt)[= #title]'
    data_content = b'#let get_data() = [Data from module]'
    
    files = {
        "main": main_content,
        "utils.typ": utils_content,
        "data.typ": data_content,
    }
    
    result = typst.compile(files, format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compile_from_empty_dict_raises_error():
    """Test that compiling with an empty dict raises an error."""
    files = {}
    
    with pytest.raises((typst.TypstError, RuntimeError, ValueError)):
        typst.compile(files, format="pdf")


def test_compiler_with_dict_input():
    """Test using Compiler class with dict input."""
    main_content = b'#import "lib.typ": greet\n#greet("World")'
    lib_content = b'#let greet(name) = [Hello, #name!]'
    
    files = {
        "main": main_content,
        "lib.typ": lib_content,
    }
    
    compiler = typst.Compiler(files)
    result = compiler.compile(format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


def test_compiler_reuse_with_dict_input():
    """Test reusing a Compiler instance with different dict inputs."""
    # First compilation
    files1 = {
        "main": b'#import "lib1.typ": func1\n#func1()',
        "lib1.typ": b'#let func1() = [First]',
    }
    
    compiler = typst.Compiler()
    result1 = compiler.compile(input=files1, format="pdf")
    assert isinstance(result1, bytes)
    assert result1.startswith(b"%PDF-")
    
    # Second compilation with different files
    files2 = {
        "main": b'#import "lib2.typ": func2\n#func2()',
        "lib2.typ": b'#let func2() = [Second]',
    }
    
    result2 = compiler.compile(input=files2, format="pdf")
    assert isinstance(result2, bytes)
    assert result2.startswith(b"%PDF-")
    
    # Results should be different
    assert result1 != result2


def test_compile_from_dict_svg_format():
    """Test compiling multi-file project to SVG."""
    main_content = b'#import "lib.typ": content\n= Title\n#content()'
    lib_content = b'#let content() = [Library content]'
    
    files = {
        "main": main_content,
        "lib.typ": lib_content,
    }
    
    result = typst.compile(files, format="svg")
    # SVG can return bytes (single page) or list (multi-page)
    if isinstance(result, list):
        assert len(result) >= 1
        assert b"<svg" in result[0]
    else:
        assert isinstance(result, bytes)
        assert b"<svg" in result


def test_compile_from_dict_png_format():
    """Test compiling multi-file project to PNG."""
    main_content = b'#import "lib.typ": content\n= Title\n#content()'
    lib_content = b'#let content() = [Library content]'
    
    files = {
        "main": main_content,
        "lib.typ": lib_content,
    }
    
    result = typst.compile(files, format="png")
    # PNG can return bytes (single page) or list (multi-page)
    if isinstance(result, list):
        assert len(result) >= 1
        assert result[0].startswith(b"\x89PNG")
    else:
        assert isinstance(result, bytes)
        assert result.startswith(b"\x89PNG")
