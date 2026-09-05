from typing import List, Optional

class Span:
    """A span in a source file.

    A span is an opaque identifier for a location in the source code.
    It can be used to associate nodes with their source positions.
    """

    def is_detached(self) -> bool:
        """Check if this is a detached span (not associated with any source)."""
        ...

    def __eq__(self, other: object) -> bool:
        """Check if two spans are equal."""
        ...

    def __hash__(self) -> int:
        """Return hash of the span."""
        ...

class SyntaxKind:
    """A syntactical building block of a Typst file."""

    def name(self) -> str:
        """Get the name of this syntax kind."""
        ...

    def is_keyword(self) -> bool:
        """Check if this is a keyword."""
        ...

    def is_trivia(self) -> bool:
        """Check if this is trivia (whitespace, comments)."""
        ...

    def is_error(self) -> bool:
        """Check if this is an error."""
        ...

class SyntaxNode:
    """A node in the untyped syntax tree."""

    def kind(self) -> SyntaxKind:
        """The type of the node."""
        ...

    def text(self) -> str:
        """The text of the node if it's a leaf or error node.
        Returns the empty string if this is an inner node."""
        ...

    def is_empty(self) -> bool:
        """Return `true` if the length is 0."""
        ...

    def erroneous(self) -> bool:
        """Whether the node or its children contain an error."""
        ...

    def children(self) -> List[SyntaxNode]:
        """The node's children."""
        ...

    def span(self) -> Span:
        """The span of this node in the source file."""
        ...

    def __len__(self) -> int:
        """The byte length of the node in the source text."""
        ...

class Markup:
    """Markup content - the root of a Typst document."""

    def exprs(self) -> List["Expr"]:
        """Get all expressions in this markup."""
        ...

    def to_untyped(self) -> SyntaxNode:
        """Get the underlying syntax node."""
        ...

    def text(self) -> str:
        """The text content of this markup."""
        ...

class Expr:
    """An expression in Typst markup or code.

    This is the base class for all expression types. Use the `variant()` method
    to check the specific type, or use `isinstance()` to check for subclasses
    like `Heading`.
    """

    def variant(self) -> str:
        """Get the variant name of this expression (e.g. 'Heading', 'Strong', 'Text')."""
        ...

    def to_untyped(self) -> SyntaxNode:
        """Get the underlying syntax node."""
        ...

    def text(self) -> str:
        """The text content of this expression."""
        ...

    def span(self) -> Span:
        """The span of this expression."""
        ...

# =============================================================================
# Markup elements
# =============================================================================

class Heading(Expr):
    """A heading expression: `= Heading`."""

    @property
    def depth(self) -> int:
        """The nesting depth of this heading (number of `=` signs)."""
        ...

    @property
    def body(self) -> Optional[Markup]:
        """Get the body of this heading as a Markup node."""
        ...

class Text(Expr):
    """A text expression: just plain text."""

    @property
    def content(self) -> str:
        """The text content."""
        ...

class Strong(Expr):
    """Strong (bold) content: `*strong*`."""

    @property
    def body(self) -> Optional[Markup]:
        """Get the body of this strong element as a Markup node."""
        ...

class Emph(Expr):
    """Emphasis (italic) content: `_emph_`."""

    @property
    def body(self) -> Optional[Markup]:
        """Get the body of this emphasis element as a Markup node."""
        ...

class Space(Expr):
    """A space expression (whitespace between content)."""

    ...

class Parbreak(Expr):
    """A paragraph break (blank line)."""

    ...

class Raw(Expr):
    """Raw text (code block): `` `code` `` or ``` ```code``` ```."""

    @property
    def block(self) -> bool:
        """Whether this is a block-level raw element."""
        ...

    @property
    def lang(self) -> Optional[str]:
        """The language tag, if any."""
        ...

    @property
    def lines(self) -> List[str]:
        """Get the text content of all lines in this raw element."""
        ...

class Equation(Expr):
    """A math equation: `$x$` or `$ x $`."""

    @property
    def block(self) -> bool:
        """Whether this is a block-level equation (display math)."""
        ...

class Linebreak(Expr):
    """A line break: `\\`."""

    ...

class Escape(Expr):
    """An escape sequence: `\\#`, `\\*`, etc."""

    @property
    def character(self) -> str:
        """The escaped character."""
        ...

class Shorthand(Expr):
    """A shorthand for a unicode codepoint: `~`, `---`, `--`, `...`."""

    @property
    def character(self) -> str:
        """The resolved unicode character."""
        ...

class SmartQuote(Expr):
    """A smart quote: `'` or `\"`."""

    @property
    def double(self) -> bool:
        """Whether this is a double quote."""
        ...

class Ref(Expr):
    """A reference to a label: `@label`."""

    @property
    def target(self) -> str:
        """The target label name."""
        ...

class Label(Expr):
    """A label: `<label>`."""

    @property
    def name(self) -> str:
        """The label name."""
        ...

class ListItem(Expr):
    """A list item: `- item`."""

    @property
    def body(self) -> Optional[Markup]:
        """Get the body of this list item as a Markup node."""
        ...

class EnumItem(Expr):
    """An enum item: `+ item` or `1. item`."""

    @property
    def number(self) -> Optional[int]:
        """The explicit number, if any."""
        ...

    @property
    def body(self) -> Optional[Markup]:
        """Get the body of this enum item as a Markup node."""
        ...

class TermItem(Expr):
    """A term list item: `/ term: description`."""

    @property
    def term(self) -> Optional[Markup]:
        """Get the term as a Markup node."""
        ...

    @property
    def description(self) -> Optional[Markup]:
        """Get the description as a Markup node."""
        ...

class Link(Expr):
    """A link to a URL: `https://example.com`.

    The URL can be retrieved via the inherited `text()` method.
    """

    ...

# =============================================================================
# Literals
# =============================================================================

class Ident(Expr):
    """An identifier: `name`."""

    @property
    def name(self) -> str:
        """The identifier name."""
        ...

class None_(Expr):
    """A none literal: `none`."""

    ...

class Auto(Expr):
    """An auto literal: `auto`."""

    ...

class Bool(Expr):
    """A boolean literal: `true` or `false`."""

    @property
    def value(self) -> bool:
        """The boolean value."""
        ...

class Int(Expr):
    """An integer literal: `123`."""

    @property
    def value(self) -> int:
        """The integer value."""
        ...

class Float(Expr):
    """A floating-point literal: `1.5`."""

    @property
    def value(self) -> float:
        """The float value."""
        ...

class Str(Expr):
    """A string literal: `"hello"`."""

    @property
    def value(self) -> str:
        """The string value."""
        ...

class Array(Expr):
    """An array expression: `(1, 2, 3)`."""

    def items(self) -> List[Expr]:
        """Get the items in this array."""
        ...

class Dict(Expr):
    """A dictionary expression: `(a: 1, b: 2)`."""

    ...

# =============================================================================
# Code structures
# =============================================================================

class CodeBlock(Expr):
    """A code block: `{ let x = 1; x + 1 }`."""

    ...

class ContentBlock(Expr):
    """A content block: `[*hello* world]`."""

    @property
    def body(self) -> Optional[Markup]:
        """Get the body of this content block as a Markup node."""
        ...

class LetBinding(Expr):
    """A let binding: `let x = 1`."""

    @property
    def init(self) -> Optional[Expr]:
        """Get the init expression, if any."""
        ...

class SetRule(Expr):
    """A set rule: `set text(red)`."""

    ...

class ShowRule(Expr):
    """A show rule: `show: it => it`."""

    ...

class Conditional(Expr):
    """A conditional expression: `if cond { ... } else { ... }`."""

    @property
    def condition(self) -> Optional[Expr]:
        """Get the condition expression."""
        ...

    @property
    def if_body(self) -> Optional[Expr]:
        """Get the if-body expression."""
        ...

    @property
    def else_body(self) -> Optional[Expr]:
        """Get the else-body expression, if any."""
        ...

class WhileLoop(Expr):
    """A while loop: `while cond { ... }`."""

    @property
    def condition(self) -> Optional[Expr]:
        """Get the condition expression."""
        ...

    @property
    def body(self) -> Optional[Expr]:
        """Get the body expression."""
        ...

class ForLoop(Expr):
    """A for loop: `for x in iter { ... }`."""

    @property
    def iterable(self) -> Optional[Expr]:
        """Get the iterable expression."""
        ...

    @property
    def body(self) -> Optional[Expr]:
        """Get the body expression."""
        ...

class ModuleImport(Expr):
    """A module import: `import "file.typ"`."""

    @property
    def source(self) -> Optional[Expr]:
        """Get the source expression."""
        ...

class ModuleInclude(Expr):
    """A module include: `include "file.typ"`."""

    @property
    def source(self) -> Optional[Expr]:
        """Get the source expression."""
        ...

class FuncReturn(Expr):
    """A function return: `return value`."""

    @property
    def body(self) -> Optional[Expr]:
        """Get the return value expression, if any."""
        ...

class Contextual(Expr):
    """A contextual expression: `context text.lang`."""

    @property
    def body(self) -> Optional[Expr]:
        """Get the body expression."""
        ...

# =============================================================================
# Math expressions
# =============================================================================

class Math(Expr):
    """Math content in an equation."""

    ...

class MathText(Expr):
    """Text in math: `x`, `25`, `=`."""

    ...

class MathIdent(Expr):
    """An identifier in math: `pi`, `alpha`."""

    @property
    def name(self) -> str:
        """The identifier name."""
        ...

class MathShorthand(Expr):
    """A shorthand in math: `<=`, `=>`."""

    ...

class MathAlignPoint(Expr):
    """An alignment point in math: `&`."""

    ...

class MathDelimited(Expr):
    """Delimited math content: `(x + y)`, `{x}`, `[x]`."""

    ...

class MathAttach(Expr):
    """Attached scripts in math: `x^2`, `x_1`, `x_1^2`."""

    ...

class MathPrimes(Expr):
    """Primes in math: `x'`, `x''`."""

    @property
    def count(self) -> int:
        """The number of prime marks."""
        ...

class MathFrac(Expr):
    """A fraction in math: `x / y`."""

    ...

class MathRoot(Expr):
    """A root in math: `sqrt(x)`, `root(3, x)`."""

    ...

# =============================================================================
# Additional expressions
# =============================================================================

class Numeric(Expr):
    """A numeric literal with a unit: `12pt`, `1em`."""

    @property
    def value(self) -> float:
        """The numeric value."""
        ...

class Parenthesized(Expr):
    """A parenthesized expression: `(x + y)`."""

    @property
    def expr(self) -> Optional[Expr]:
        """Get the inner expression."""
        ...

class Unary(Expr):
    """A unary operation: `-x`, `not x`."""

    @property
    def expr(self) -> Optional[Expr]:
        """Get the operand expression."""
        ...

class Binary(Expr):
    """A binary operation: `x + y`, `x and y`."""

    @property
    def lhs(self) -> Optional[Expr]:
        """Get the left-hand side expression."""
        ...

    @property
    def rhs(self) -> Optional[Expr]:
        """Get the right-hand side expression."""
        ...

class FieldAccess(Expr):
    """A field access: `x.y`."""

    @property
    def target(self) -> Optional[Expr]:
        """Get the target expression."""
        ...

    @property
    def field(self) -> Optional[str]:
        """Get the field name."""
        ...

class FuncCall(Expr):
    """A function call: `f(x)`, `f(x, y)`."""

    @property
    def callee(self) -> Optional[Expr]:
        """Get the callee expression."""
        ...

class Closure(Expr):
    """A closure: `(x) => x + 1`."""

    @property
    def body(self) -> Optional[Expr]:
        """Get the body expression."""
        ...

class DestructAssign(Expr):
    """A destructuring assignment: `(a, b) = (1, 2)`."""

    @property
    def value(self) -> Optional[Expr]:
        """Get the value expression."""
        ...

class LoopBreak(Expr):
    """A loop break: `break`."""

    ...

class LoopContinue(Expr):
    """A loop continue: `continue`."""

    ...

# =============================================================================
# Functions
# =============================================================================

def parse(input: str) -> Markup:
    """Parse Typst source code into a markup tree.

    Args:
        input: The Typst source code to parse.

    Returns:
        The root Markup node of the parsed document.
    """
    ...
