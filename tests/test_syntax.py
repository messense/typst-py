import typst


# Syntax parsing tests
def test_parse_simple_markup():
    source = "= Hello\nThis is a test."
    markup = typst.syntax.parse(source)
    assert markup is not None
    assert isinstance(markup, typst.syntax.Markup)
    node = markup.to_untyped()
    assert node.kind().name() == "markup"
    assert len(node) > 0  # Has some length


def test_parse_code():
    source = "#let x = 1\n#x"
    markup = typst.syntax.parse(source)
    assert markup is not None
    assert isinstance(markup, typst.syntax.Markup)


def test_syntax_node_kind():
    source = "= Heading"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    kind = node.kind()
    assert isinstance(kind, typst.syntax.SyntaxKind)
    assert isinstance(kind.name(), str)
    assert len(kind.name()) > 0


def test_syntax_node_text():
    source = "Hello"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    # The root node doesn't have text, but its children do
    assert isinstance(node.text(), str)


def test_syntax_node_children():
    source = "= Hello\nWorld"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    children = node.children()
    assert isinstance(children, list)
    assert len(children) > 0
    for child in children:
        assert isinstance(child, typst.syntax.SyntaxNode)


def test_syntax_node_is_empty():
    source = ""
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    # Empty source may not result in an empty node due to structure
    assert isinstance(node.is_empty(), bool)


def test_syntax_node_erroneous():
    # Valid syntax
    valid_source = "= Hello"
    valid_markup = typst.syntax.parse(valid_source)
    valid_node = valid_markup.to_untyped()
    assert not valid_node.erroneous()

    # Invalid syntax
    invalid_source = "#let x ="  # Incomplete assignment
    invalid_markup = typst.syntax.parse(invalid_source)
    invalid_node = invalid_markup.to_untyped()
    assert invalid_node.erroneous()


def test_syntax_kind_methods():
    source = "// comment"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()

    # Get first child which should be the comment
    children = node.children()
    if children:
        comment_kind = children[0].kind()
        assert comment_kind.is_trivia()  # Comments are trivia
        assert not comment_kind.is_keyword()
        assert not comment_kind.is_error()


def test_syntax_node_repr():
    source = "= Hello"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    repr_str = repr(node)
    assert "SyntaxNode" in repr_str
    assert "kind=" in repr_str


def test_syntax_kind_repr():
    source = "= Hello"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    kind = node.kind()
    repr_str = repr(kind)
    assert "SyntaxKind" in repr_str


def test_parse_math():
    source = "$ x^2 + y^2 = z^2 $"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    assert node is not None
    assert not node.erroneous()


def test_parse_complex_document():
    source = """
= Introduction

This is a paragraph with *bold* and _italic_ text.

== Subsection

- List item 1
- List item 2

#let x = 42
#x

$ sum_(i=1)^n i = (n(n+1))/2 $
"""
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    assert node is not None
    assert not node.erroneous()
    children = node.children()
    assert len(children) > 0


def test_syntax_tree_traversal():
    source = "= Hello\n*World*"
    markup = typst.syntax.parse(source)
    root = markup.to_untyped()

    # Traverse the tree
    def count_nodes(node):
        count = 1
        for child in node.children():
            count += count_nodes(child)
        return count

    total_nodes = count_nodes(root)
    assert total_nodes > 1  # Should have multiple nodes


def test_parse_with_errors():
    # Test that parsing invalid syntax still produces a tree with errors
    source = "#let x = \n#("
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    assert node is not None
    assert node.erroneous()


def test_parse_empty_string():
    source = ""
    markup = typst.syntax.parse(source)
    assert markup is not None
    # Empty source creates a valid but minimal tree
    assert isinstance(markup, typst.syntax.Markup)


def test_span_basic():
    """Test that span() returns a Span object."""
    source = "= Hello"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    span = node.span()

    assert span is not None
    assert isinstance(span, typst.syntax.Span)


def test_span_equality():
    """Test that spans from the same node are equal."""
    source = "= Hello"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    span1 = node.span()
    span2 = node.span()

    assert span1 == span2


def test_span_different_nodes():
    """Test that spans from different nodes can be different."""
    source = "= Hello\n*World*"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    children = node.children()

    if len(children) >= 2:
        span1 = children[0].span()
        span2 = children[1].span()
        # Spans from different children might be different
        # (we don't assert inequality since it depends on implementation)
        assert isinstance(span1, typst.syntax.Span)
        assert isinstance(span2, typst.syntax.Span)


def test_span_hashable():
    """Test that spans are hashable and can be used in sets/dicts."""
    source = "= Hello\n*World*"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()

    # Should be able to create a set of spans
    span_set = {node.span()}
    assert len(span_set) == 1

    # Should be able to use as dict key
    span_dict = {node.span(): "root"}
    assert span_dict[node.span()] == "root"


def test_span_is_detached():
    """Test the is_detached method."""
    source = "= Hello"
    markup = typst.syntax.parse(source)
    node = markup.to_untyped()
    span = node.span()

    # Normal parsed nodes should not be detached
    # (though this depends on Typst's implementation)
    is_detached = span.is_detached()
    assert isinstance(is_detached, bool)


# Typed AST Tests


def test_markup_exprs():
    """Test that Markup.exprs() returns a list of typed expressions."""
    source = "= Hello\nWorld"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    assert isinstance(exprs, list)
    assert len(exprs) > 0
    for expr in exprs:
        assert isinstance(expr, typst.syntax.Expr)


def test_expr_variant():
    """Test that Expr.variant() returns the correct type name."""
    source = "= Hello"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    # Find the heading expression
    heading_found = False
    for expr in exprs:
        variant = expr.variant()
        assert isinstance(variant, str)
        if variant == "Heading":
            heading_found = True
    assert heading_found, "Expected to find a Heading expression"


def test_expr_to_untyped():
    """Test that Expr.to_untyped() returns a SyntaxNode."""
    source = "= Hello"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    for expr in exprs:
        node = expr.to_untyped()
        assert isinstance(node, typst.syntax.SyntaxNode)


def test_heading_isinstance():
    """Test isinstance check for Heading expressions."""
    source = "= Hello"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    heading_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.Heading):
            heading_found = True
            assert expr.variant() == "Heading"
    assert heading_found, "Expected to find a Heading instance"


def test_heading_depth():
    """Test Heading.depth property."""
    source = "= Level 1\n== Level 2\n=== Level 3"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    depths = []
    for expr in exprs:
        if isinstance(expr, typst.syntax.Heading):
            depths.append(expr.depth)

    assert 1 in depths, "Expected depth 1 heading"
    assert 2 in depths, "Expected depth 2 heading"
    assert 3 in depths, "Expected depth 3 heading"


def test_heading_body():
    """Test Heading.body property returns Markup."""
    source = "= Hello World"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    for expr in exprs:
        if isinstance(expr, typst.syntax.Heading):
            body = expr.body
            assert body is not None
            assert isinstance(body, typst.syntax.Markup)
            break


def test_strong_isinstance():
    """Test isinstance check for Strong expressions."""
    source = "*bold text*"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    strong_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.Strong):
            strong_found = True
            assert expr.variant() == "Strong"
            body = expr.body
            assert body is not None
            assert isinstance(body, typst.syntax.Markup)
    assert strong_found, "Expected to find a Strong instance"


def test_emph_isinstance():
    """Test isinstance check for Emph expressions."""
    source = "_italic text_"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    emph_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.Emph):
            emph_found = True
            assert expr.variant() == "Emph"
            body = expr.body
            assert body is not None
            assert isinstance(body, typst.syntax.Markup)
    assert emph_found, "Expected to find an Emph instance"


def test_text_isinstance():
    """Test isinstance check for Text expressions."""
    source = "Hello World"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    text_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.Text):
            text_found = True
            assert expr.variant() == "Text"
    assert text_found, "Expected to find a Text instance"


def test_raw_isinstance():
    """Test isinstance check for Raw expressions."""
    source = "`code`"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    raw_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.Raw):
            raw_found = True
            assert expr.variant() == "Raw"
    assert raw_found, "Expected to find a Raw instance"


def test_link_isinstance():
    """Test isinstance check for Link expressions."""
    source = "https://example.com"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    link_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.Link):
            link_found = True
            assert expr.variant() == "Link"
    assert link_found, "Expected to find a Link instance"


def test_equation_isinstance():
    """Test isinstance check for Equation expressions."""
    source = "$ x^2 + y^2 = z^2 $"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    equation_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.Equation):
            equation_found = True
            assert expr.variant() == "Equation"
    assert equation_found, "Expected to find an Equation instance"


def test_list_item_isinstance():
    """Test isinstance check for ListItem expressions."""
    source = "- Item 1\n- Item 2"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    list_item_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.ListItem):
            list_item_found = True
            assert expr.variant() == "ListItem"
            body = expr.body
            assert body is not None
            assert isinstance(body, typst.syntax.Markup)
    assert list_item_found, "Expected to find a ListItem instance"


def test_enum_item_isinstance():
    """Test isinstance check for EnumItem expressions."""
    source = "+ Item 1\n+ Item 2"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    enum_item_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.EnumItem):
            enum_item_found = True
            assert expr.variant() == "EnumItem"
            body = expr.body
            assert body is not None
            assert isinstance(body, typst.syntax.Markup)
    assert enum_item_found, "Expected to find an EnumItem instance"


def test_term_item_isinstance():
    """Test isinstance check for TermItem expressions."""
    source = "/ Term: Definition"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    term_item_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.TermItem):
            term_item_found = True
            assert expr.variant() == "TermItem"
            term = expr.term
            description = expr.description
            assert term is not None
            assert description is not None
            assert isinstance(term, typst.syntax.Markup)
            assert isinstance(description, typst.syntax.Markup)
    assert term_item_found, "Expected to find a TermItem instance"


def test_ref_isinstance():
    """Test isinstance check for Ref expressions."""
    source = "@label"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    ref_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.Ref):
            ref_found = True
            assert expr.variant() == "Ref"
    assert ref_found, "Expected to find a Ref instance"


def test_label_isinstance():
    """Test isinstance check for Label expressions."""
    source = "<label>"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    label_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.Label):
            label_found = True
            assert expr.variant() == "Label"
    assert label_found, "Expected to find a Label instance"


def test_linebreak_isinstance():
    """Test isinstance check for Linebreak expressions."""
    source = "Line 1\\\nLine 2"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    linebreak_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.Linebreak):
            linebreak_found = True
            assert expr.variant() == "Linebreak"
    assert linebreak_found, "Expected to find a Linebreak instance"


def test_smartquote_isinstance():
    """Test isinstance check for SmartQuote expressions."""
    source = '"quoted"'
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    smartquote_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.SmartQuote):
            smartquote_found = True
            assert expr.variant() == "SmartQuote"
    assert smartquote_found, "Expected to find a SmartQuote instance"


def test_code_block_isinstance():
    """Test isinstance check for code expressions via hash syntax."""
    source = "#let x = 1"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    # Code blocks introduce Let expressions
    let_found = False
    for expr in exprs:
        if isinstance(expr, typst.syntax.LetBinding):
            let_found = True
            assert expr.variant() == "LetBinding"
    assert let_found, "Expected to find a LetBinding instance"


def test_multiple_expression_types():
    """Test a document with multiple expression types."""
    source = """= Heading

This is *bold* and _italic_ text.

- List item

$ E = mc^2 $
"""
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    variants = {expr.variant() for expr in exprs}

    # Check that we have multiple different types
    assert "Heading" in variants
    # Strong and Emph may be nested inside other expressions
    assert len(variants) > 1


def test_expr_inheritance():
    """Test that typed expressions inherit from Expr."""
    source = "= Hello"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    for expr in exprs:
        if isinstance(expr, typst.syntax.Heading):
            # Should also be an instance of Expr
            assert isinstance(expr, typst.syntax.Expr)
            # Should have methods from Expr
            assert hasattr(expr, "variant")
            assert hasattr(expr, "to_untyped")
            break


def test_nested_expressions():
    """Test expressions nested inside other expressions."""
    source = "= *Bold Heading*"
    markup = typst.syntax.parse(source)
    exprs = markup.exprs()

    # Find the heading
    for expr in exprs:
        if isinstance(expr, typst.syntax.Heading):
            body = expr.body
            assert body is not None
            # The body contains the strong expression
            body_exprs = body.exprs()
            strong_found = False
            for body_expr in body_exprs:
                if isinstance(body_expr, typst.syntax.Strong):
                    strong_found = True
            assert strong_found, "Expected Strong inside Heading body"
            break
