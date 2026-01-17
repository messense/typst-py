import pytest
import pathlib

import typst


# Fixtures
@pytest.fixture
def hello_typ_path():
    return pathlib.Path("tests/hello.typ")


# typst.Fonts tests
def test_fonts_default():
    fonts = typst.Fonts()
    assert fonts is not None


def test_fonts_no_system_fonts():
    fonts = typst.Fonts(include_system_fonts=False)
    assert fonts is not None


def test_fonts_no_embedded_fonts():
    fonts = typst.Fonts(include_embedded_fonts=False)
    assert fonts is not None


def test_fonts_with_paths():
    fonts = typst.Fonts(font_paths=["/usr/share/fonts"])
    assert fonts is not None


def test_compile_with_fonts_object(hello_typ_path):
    fonts = typst.Fonts(include_system_fonts=True)
    result = typst.compile(hello_typ_path, font_paths=fonts, format="pdf")
    assert isinstance(result, bytes)
    assert result.startswith(b"%PDF-")


# Font discovery methods tests
def test_fonts_families():
    fonts_obj = typst.Fonts()
    families = fonts_obj.families()
    assert isinstance(families, list)
    assert len(families) > 0
    assert all(isinstance(f, str) for f in families)


def test_fonts_fonts_returns_list():
    fonts_obj = typst.Fonts()
    fonts_list = fonts_obj.fonts()
    assert isinstance(fonts_list, list)
    assert len(fonts_list) > 0


def test_font_info_attributes():
    fonts_obj = typst.Fonts()
    fonts_list = fonts_obj.fonts()
    font = fonts_list[0]

    assert isinstance(font.family, str)
    assert font.style in ("normal", "italic", "oblique")
    assert isinstance(font.weight, int)
    assert 100 <= font.weight <= 900
    assert isinstance(font.stretch, float)
    assert font.path is None or isinstance(font.path, str)
    assert isinstance(font.index, int)


def test_fonts_no_system_fonts_families():
    fonts_obj = typst.Fonts(include_system_fonts=False)
    families = fonts_obj.families()
    assert isinstance(families, list)


def test_fonts_no_embedded_fonts_families():
    fonts_obj = typst.Fonts(include_embedded_fonts=False)
    families = fonts_obj.families()
    assert isinstance(families, list)


def test_fonts_embedded_only():
    fonts_obj = typst.Fonts(include_system_fonts=False, include_embedded_fonts=True)
    fonts_list = fonts_obj.fonts()
    assert isinstance(fonts_list, list)
    assert len(fonts_list) > 0
    for font in fonts_list:
        assert font.path is None


def test_font_info_repr():
    fonts_obj = typst.Fonts()
    fonts_list = fonts_obj.fonts()
    font = fonts_list[0]
    repr_str = repr(font)
    assert "FontInfo" in repr_str
    assert font.family in repr_str
