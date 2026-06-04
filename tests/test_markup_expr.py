import typst


def test_parse_returns_markup():
    """Test that parse() returns a Markup object."""
    source = "= Heading\n*bold*"
    markup = typst.syntax.parse(source)

    assert markup is not None
    assert isinstance(markup, typst.syntax.Markup)


def test_markup_exprs():
    """Test that Markup.exprs() returns expressions."""
    source = "= Heading\n*bold* and _italic_"
    markup = typst.syntax.parse(source)

    exprs = markup.exprs()
    assert isinstance(exprs, list)
    assert len(exprs) > 0

    # All should be Expr objects
    for expr in exprs:
        assert isinstance(expr, typst.syntax.Expr)


def test_expr_variant():
    """Test that Expr.variant() returns the variant name."""
    source = "= Heading\n*bold* and _italic_"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    # Check we have various expression types
    variants = [expr.variant() for expr in exprs]
    assert "Heading" in variants
    assert "Strong" in variants
    assert "Emph" in variants
    assert "Text" in variants


def test_expr_text():
    """Test that Expr.text() returns the text content."""
    source = "hello world"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    # Should have a Text expression
    text_exprs = [e for e in exprs if e.variant() == "Text"]
    assert len(text_exprs) > 0

    # Check text content
    assert "hello" in text_exprs[0].text() or "world" in text_exprs[0].text()


def test_expr_to_untyped():
    """Test that Expr.to_untyped() returns SyntaxNode."""
    source = "= Heading"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    for expr in exprs:
        node = expr.to_untyped()
        assert isinstance(node, typst.syntax.SyntaxNode)


def test_markup_to_untyped():
    """Test that Markup.to_untyped() returns SyntaxNode."""
    source = "= Heading"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()

    assert isinstance(node, typst.syntax.SyntaxNode)


def test_complex_document():
    """Test parsing a complex document with various expression types."""
    source = """
= Main Heading

This is a paragraph with *bold* and _italic_ text.

== Subsection

- List item
- Another item

#let x = 42
"""
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    variants = [expr.variant() for expr in exprs]

    # Should contain various types
    assert "Heading" in variants
    assert "Strong" in variants
    assert "Emph" in variants
    assert "Text" in variants
    assert "List" in variants or "ListItem" in variants
    assert "LetBinding" in variants
