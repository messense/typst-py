use pyo3::prelude::*;

use typst::syntax::{
    self,
    ast::{self, AstNode},
};

/// A span in a source file.
///
/// A span is an opaque identifier for a location in the source code.
/// It can be used to associate nodes with their source positions.
#[pyclass(name = "Span")]
#[derive(Clone, Copy)]
pub struct PySpan(syntax::Span);

#[pymethods]
impl PySpan {
    /// Check if this is a detached span (not associated with any source).
    fn is_detached(&self) -> bool {
        self.0.is_detached()
    }

    fn __repr__(&self) -> String {
        format!("Span({:?})", self.0)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}

/// A wrapper for SyntaxKind to expose it to Python
#[pyclass(name = "SyntaxKind")]
#[derive(Clone, Copy)]
pub struct PySyntaxKind(syntax::SyntaxKind);

#[pymethods]
impl PySyntaxKind {
    /// Get the name of this syntax kind
    fn name(&self) -> &str {
        self.0.name()
    }

    /// Check if this is a keyword
    fn is_keyword(&self) -> bool {
        self.0.is_keyword()
    }

    /// Check if this is trivia (whitespace, comments)
    fn is_trivia(&self) -> bool {
        self.0.is_trivia()
    }

    /// Check if this is an error
    fn is_error(&self) -> bool {
        self.0.is_error()
    }

    fn __repr__(&self) -> String {
        format!("SyntaxKind.{}", self.0.name())
    }

    fn __str__(&self) -> &str {
        self.0.name()
    }
}

/// A node in the untyped syntax tree.
#[pyclass(name = "SyntaxNode")]
pub struct SyntaxNode(syntax::SyntaxNode);

#[pymethods]
impl SyntaxNode {
    /// The type of the node.
    fn kind(&self) -> PySyntaxKind {
        PySyntaxKind(self.0.kind())
    }

    /// The text of the node if it's a leaf or error node.
    /// Returns the empty string if this is an inner node.
    fn text(&self) -> String {
        self.0.text().to_string()
    }

    /// Return `true` if the length is 0.
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Whether the node or its children contain an error.
    fn erroneous(&self) -> bool {
        self.0.erroneous()
    }

    /// The node's children.
    fn children(&self) -> Vec<SyntaxNode> {
        self.0.children().cloned().map(SyntaxNode).collect()
    }

    /// The span of this node in the source file.
    fn span(&self) -> PySpan {
        PySpan(self.0.span())
    }

    /// The byte length of the node in the source text.
    fn __len__(&self) -> usize {
        self.0.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "SyntaxNode(kind={}, len={})",
            self.0.kind().name(),
            self.0.len()
        )
    }
}

/// Markup content - the root of a Typst document.
#[pyclass(name = "Markup")]
pub struct PyMarkup(syntax::SyntaxNode);

#[pymethods]
impl PyMarkup {
    /// Get all expressions in this markup.
    fn exprs(&self, py: Python<'_>) -> PyResult<Vec<Py<PyAny>>> {
        if let Some(markup) = self.0.cast::<ast::Markup>() {
            markup
                .exprs()
                .map(|expr| PyExpr::from_ast(py, expr))
                .collect()
        } else {
            Ok(vec![])
        }
    }

    /// Get the underlying syntax node.
    fn to_untyped(&self) -> SyntaxNode {
        SyntaxNode(self.0.clone())
    }

    /// The text content of this markup.
    fn text(&self) -> String {
        self.0.text().to_string()
    }

    fn __repr__(&self) -> String {
        format!("Markup(len={})", self.0.len())
    }
}

/// An expression in Typst markup or code.
///
/// This is the base class for all expression types. Use the `variant()` method
/// to check the specific type, or use `isinstance()` to check for subclasses
/// like `Heading`.
#[pyclass(subclass, name = "Expr", module = "typst.syntax")]
#[derive(Clone)]
pub struct PyExpr {
    pub(crate) node: syntax::SyntaxNode,
    variant: &'static str,
}

impl PyExpr {
    /// Create an appropriate Python object for an AST expression.
    /// Returns the specific subclass (e.g., PyHeading) when available,
    /// otherwise returns a generic PyExpr.
    fn from_ast(py: Python<'_>, expr: ast::Expr) -> PyResult<Py<PyAny>> {
        match expr {
            // Markup elements
            ast::Expr::Heading(heading) => PyHeading::create(py, heading),
            ast::Expr::Text(text) => PyText::create(py, text),
            ast::Expr::Strong(strong) => PyStrong::create(py, strong),
            ast::Expr::Emph(emph) => PyEmph::create(py, emph),
            ast::Expr::Space(space) => PySpace::create(py, space),
            ast::Expr::Parbreak(parbreak) => PyParbreak::create(py, parbreak),
            ast::Expr::Raw(raw) => PyRaw::create(py, raw),
            ast::Expr::Equation(equation) => PyEquation::create(py, equation),
            ast::Expr::Link(link) => PyLink::create(py, link),
            ast::Expr::Linebreak(linebreak) => PyLinebreak::create(py, linebreak),
            ast::Expr::Escape(escape) => PyEscape::create(py, escape),
            ast::Expr::Shorthand(shorthand) => PyShorthand::create(py, shorthand),
            ast::Expr::SmartQuote(smartquote) => PySmartQuote::create(py, smartquote),
            ast::Expr::Ref(reference) => PyReference::create(py, reference),
            ast::Expr::Label(label) => PyLabel::create(py, label),
            ast::Expr::ListItem(item) => PyListItem::create(py, item),
            ast::Expr::EnumItem(item) => PyEnumItem::create(py, item),
            ast::Expr::TermItem(item) => PyTermItem::create(py, item),
            // Literals
            ast::Expr::Ident(ident) => PyIdent::create(py, ident),
            ast::Expr::None(none) => PyNone::create(py, none),
            ast::Expr::Auto(auto) => PyAuto::create(py, auto),
            ast::Expr::Bool(boolean) => PyBool::create(py, boolean),
            ast::Expr::Int(int) => PyInt::create(py, int),
            ast::Expr::Float(float) => PyFloat::create(py, float),
            ast::Expr::Str(s) => PyStr::create(py, s),
            ast::Expr::Array(array) => PyArray::create(py, array),
            ast::Expr::Dict(dict) => PyDict::create(py, dict),
            // Code structures
            ast::Expr::CodeBlock(block) => PyCodeBlock::create(py, block),
            ast::Expr::ContentBlock(block) => PyContentBlock::create(py, block),
            ast::Expr::LetBinding(binding) => PyLetBinding::create(py, binding),
            ast::Expr::SetRule(rule) => PySetRule::create(py, rule),
            ast::Expr::ShowRule(rule) => PyShowRule::create(py, rule),
            ast::Expr::Conditional(cond) => PyConditional::create(py, cond),
            ast::Expr::WhileLoop(loop_) => PyWhileLoop::create(py, loop_),
            ast::Expr::ForLoop(loop_) => PyForLoop::create(py, loop_),
            ast::Expr::ModuleImport(import) => PyModuleImport::create(py, import),
            ast::Expr::ModuleInclude(include) => PyModuleInclude::create(py, include),
            ast::Expr::FuncReturn(ret) => PyFuncReturn::create(py, ret),
            ast::Expr::Contextual(ctx) => PyContextual::create(py, ctx),
            // Math expressions
            ast::Expr::Math(math) => PyMath::create(py, math),
            ast::Expr::MathText(text) => PyMathText::create(py, text),
            ast::Expr::MathIdent(ident) => PyMathIdent::create(py, ident),
            ast::Expr::MathShorthand(shorthand) => PyMathShorthand::create(py, shorthand),
            ast::Expr::MathAlignPoint(point) => PyMathAlignPoint::create(py, point),
            ast::Expr::MathDelimited(delim) => PyMathDelimited::create(py, delim),
            ast::Expr::MathAttach(attach) => PyMathAttach::create(py, attach),
            ast::Expr::MathPrimes(primes) => PyMathPrimes::create(py, primes),
            ast::Expr::MathFrac(frac) => PyMathFrac::create(py, frac),
            ast::Expr::MathRoot(root) => PyMathRoot::create(py, root),
            // Additional expressions
            ast::Expr::Numeric(numeric) => PyNumeric::create(py, numeric),
            ast::Expr::Parenthesized(paren) => PyParenthesized::create(py, paren),
            ast::Expr::Unary(unary) => PyUnary::create(py, unary),
            ast::Expr::Binary(binary) => PyBinary::create(py, binary),
            ast::Expr::FieldAccess(access) => PyFieldAccess::create(py, access),
            ast::Expr::FuncCall(call) => PyFuncCall::create(py, call),
            ast::Expr::Closure(closure) => PyClosure::create(py, closure),
            ast::Expr::DestructAssignment(assign) => PyDestructAssign::create(py, assign),
            ast::Expr::LoopBreak(brk) => PyLoopBreak::create(py, brk),
            ast::Expr::LoopContinue(cont) => PyLoopContinue::create(py, cont),
        }
    }
}

#[pymethods]
impl PyExpr {
    /// Get the variant name of this expression.
    fn variant(&self) -> &str {
        &self.variant
    }

    /// Get the underlying syntax node.
    fn to_untyped(&self) -> SyntaxNode {
        SyntaxNode(self.node.clone())
    }

    /// The text content of this expression.
    fn text(&self) -> String {
        self.node.text().to_string()
    }

    /// The span of this expression.
    fn span(&self) -> PySpan {
        PySpan(self.node.span())
    }

    fn __repr__(&self) -> String {
        format!("Expr.{}(...)", self.variant)
    }
}

/// A heading expression: `= Heading`.
#[pyclass(extends = PyExpr, name = "Heading", module = "typst.syntax")]
pub struct PyHeading {
    /// The nesting depth of this heading (number of `=` signs).
    #[pyo3(get)]
    depth: usize,
}

impl PyHeading {
    fn create(py: Python<'_>, heading: ast::Heading) -> PyResult<Py<PyAny>> {
        let node = heading.to_untyped().clone();
        let depth = heading.depth().get();

        let parent = PyExpr {
            node,
            variant: "Heading",
        };
        let child = PyHeading { depth };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyHeading {
    /// Get the body of this heading as a Markup node.
    #[getter]
    fn body(self_: PyRef<'_, Self>) -> Option<PyMarkup> {
        let parent = self_.as_ref();
        if let Some(heading) = parent.node.cast::<ast::Heading>() {
            Some(PyMarkup(heading.body().to_untyped().clone()))
        } else {
            None
        }
    }

    fn __repr__(&self) -> String {
        format!("Heading(depth={})", self.depth)
    }
}

/// A text expression: just plain text.
#[pyclass(extends = PyExpr, name = "Text", module = "typst.syntax")]
pub struct PyText {
    /// The text content.
    #[pyo3(get)]
    content: String,
}

impl PyText {
    fn create(py: Python<'_>, text: ast::Text) -> PyResult<Py<PyAny>> {
        let node = text.to_untyped().clone();
        let content = text.get().to_string();

        let parent = PyExpr {
            node,
            variant: "Text",
        };
        let child = PyText { content };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyText {
    fn __repr__(&self) -> String {
        let display = if self.content.len() > 20 {
            format!("{}...", &self.content[..20])
        } else {
            self.content.clone()
        };
        format!("Text({:?})", display)
    }
}

/// Strong (bold) content: `*strong*`.
#[pyclass(extends = PyExpr, name = "Strong", module = "typst.syntax")]
pub struct PyStrong;

impl PyStrong {
    fn create(py: Python<'_>, strong: ast::Strong) -> PyResult<Py<PyAny>> {
        let node = strong.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Strong",
        };
        let child = PyStrong;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyStrong {
    /// Get the body of this strong element as a Markup node.
    #[getter]
    fn body(self_: PyRef<'_, Self>) -> Option<PyMarkup> {
        let parent = self_.as_ref();
        if let Some(strong) = parent.node.cast::<ast::Strong>() {
            Some(PyMarkup(strong.body().to_untyped().clone()))
        } else {
            None
        }
    }

    fn __repr__(&self) -> String {
        "Strong(...)".to_string()
    }
}

/// Emphasis (italic) content: `_emph_`.
#[pyclass(extends = PyExpr, name = "Emph", module = "typst.syntax")]
pub struct PyEmph;

impl PyEmph {
    fn create(py: Python<'_>, emph: ast::Emph) -> PyResult<Py<PyAny>> {
        let node = emph.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Emph",
        };
        let child = PyEmph;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyEmph {
    /// Get the body of this emphasis element as a Markup node.
    #[getter]
    fn body(self_: PyRef<'_, Self>) -> Option<PyMarkup> {
        let parent = self_.as_ref();
        if let Some(emph) = parent.node.cast::<ast::Emph>() {
            Some(PyMarkup(emph.body().to_untyped().clone()))
        } else {
            None
        }
    }

    fn __repr__(&self) -> String {
        "Emph(...)".to_string()
    }
}

/// A space expression (whitespace between content).
#[pyclass(extends = PyExpr, name = "Space", module = "typst.syntax")]
pub struct PySpace;

impl PySpace {
    fn create(py: Python<'_>, space: ast::Space) -> PyResult<Py<PyAny>> {
        let node = space.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Space",
        };
        let child = PySpace;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PySpace {
    fn __repr__(&self) -> String {
        "Space()".to_string()
    }
}

/// A paragraph break (blank line).
#[pyclass(extends = PyExpr, name = "Parbreak", module = "typst.syntax")]
pub struct PyParbreak;

impl PyParbreak {
    fn create(py: Python<'_>, parbreak: ast::Parbreak) -> PyResult<Py<PyAny>> {
        let node = parbreak.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Parbreak",
        };
        let child = PyParbreak;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyParbreak {
    fn __repr__(&self) -> String {
        "Parbreak()".to_string()
    }
}

/// Raw text (code block): `` `code` `` or ``` ```code``` ```.
#[pyclass(extends = PyExpr, name = "Raw", module = "typst.syntax")]
pub struct PyRaw {
    /// Whether this is a block-level raw element.
    #[pyo3(get)]
    block: bool,
    /// The language tag, if any.
    #[pyo3(get)]
    lang: Option<String>,
}

impl PyRaw {
    fn create(py: Python<'_>, raw: ast::Raw) -> PyResult<Py<PyAny>> {
        let node = raw.to_untyped().clone();
        let block = raw.block();
        let lang = raw.lang().map(|l| l.get().to_string());

        let parent = PyExpr {
            node,
            variant: "Raw",
        };
        let child = PyRaw { block, lang };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyRaw {
    /// Get the text content of all lines in this raw element.
    #[getter]
    fn lines(self_: PyRef<'_, Self>) -> Vec<String> {
        let parent = self_.as_ref();
        if let Some(raw) = parent.node.cast::<ast::Raw>() {
            raw.lines().map(|line| line.get().to_string()).collect()
        } else {
            vec![]
        }
    }

    fn __repr__(&self) -> String {
        if let Some(lang) = &self.lang {
            format!("Raw(lang={:?}, block={})", lang, self.block)
        } else {
            format!("Raw(block={})", self.block)
        }
    }
}

/// A math equation: `$x$` or `$ x $`.
#[pyclass(extends = PyExpr, name = "Equation", module = "typst.syntax")]
pub struct PyEquation {
    /// Whether this is a block-level equation (display math).
    #[pyo3(get)]
    block: bool,
}

impl PyEquation {
    fn create(py: Python<'_>, equation: ast::Equation) -> PyResult<Py<PyAny>> {
        let node = equation.to_untyped().clone();
        let block = equation.block();

        let parent = PyExpr {
            node,
            variant: "Equation",
        };
        let child = PyEquation { block };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyEquation {
    fn __repr__(&self) -> String {
        format!("Equation(block={})", self.block)
    }
}

/// A line break: `\`.
#[pyclass(extends = PyExpr, name = "Linebreak", module = "typst.syntax")]
pub struct PyLinebreak;

impl PyLinebreak {
    fn create(py: Python<'_>, linebreak: ast::Linebreak) -> PyResult<Py<PyAny>> {
        let node = linebreak.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Linebreak",
        };
        let child = PyLinebreak;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyLinebreak {
    fn __repr__(&self) -> String {
        "Linebreak()".to_string()
    }
}

/// An escape sequence: `\#`, `\*`, etc.
#[pyclass(extends = PyExpr, name = "Escape", module = "typst.syntax")]
pub struct PyEscape {
    /// The escaped character.
    #[pyo3(get)]
    character: char,
}

impl PyEscape {
    fn create(py: Python<'_>, escape: ast::Escape) -> PyResult<Py<PyAny>> {
        let node = escape.to_untyped().clone();
        let character = escape.get();

        let parent = PyExpr {
            node,
            variant: "Escape",
        };
        let child = PyEscape { character };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyEscape {
    fn __repr__(&self) -> String {
        format!("Escape({:?})", self.character)
    }
}

/// A shorthand for a unicode codepoint: `~`, `---`, `--`, `...`.
#[pyclass(extends = PyExpr, name = "Shorthand", module = "typst.syntax")]
pub struct PyShorthand {
    /// The resolved unicode character.
    #[pyo3(get)]
    character: char,
}

impl PyShorthand {
    fn create(py: Python<'_>, shorthand: ast::Shorthand) -> PyResult<Py<PyAny>> {
        let node = shorthand.to_untyped().clone();
        let character = shorthand.get();

        let parent = PyExpr {
            node,
            variant: "Shorthand",
        };
        let child = PyShorthand { character };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyShorthand {
    fn __repr__(&self) -> String {
        format!("Shorthand({:?})", self.character)
    }
}

/// A smart quote: `'` or `"`.
#[pyclass(extends = PyExpr, name = "SmartQuote", module = "typst.syntax")]
pub struct PySmartQuote {
    /// Whether this is a double quote.
    #[pyo3(get)]
    double: bool,
}

impl PySmartQuote {
    fn create(py: Python<'_>, smartquote: ast::SmartQuote) -> PyResult<Py<PyAny>> {
        let node = smartquote.to_untyped().clone();
        let double = smartquote.double();

        let parent = PyExpr {
            node,
            variant: "SmartQuote",
        };
        let child = PySmartQuote { double };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PySmartQuote {
    fn __repr__(&self) -> String {
        format!("SmartQuote(double={})", self.double)
    }
}

/// A reference to a label: `@label`.
#[pyclass(extends = PyExpr, name = "Ref", module = "typst.syntax")]
pub struct PyReference {
    /// The target label name.
    #[pyo3(get)]
    target: String,
}

impl PyReference {
    fn create(py: Python<'_>, reference: ast::Ref) -> PyResult<Py<PyAny>> {
        let node = reference.to_untyped().clone();
        let target = reference.target().to_string();

        let parent = PyExpr {
            node,
            variant: "Ref",
        };
        let child = PyReference { target };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyReference {
    fn __repr__(&self) -> String {
        format!("Ref({:?})", self.target)
    }
}

/// A label: `<label>`.
#[pyclass(extends = PyExpr, name = "Label", module = "typst.syntax")]
pub struct PyLabel {
    /// The label name.
    #[pyo3(get)]
    name: String,
}

impl PyLabel {
    fn create(py: Python<'_>, label: ast::Label) -> PyResult<Py<PyAny>> {
        let node = label.to_untyped().clone();
        let name = label.get().to_string();

        let parent = PyExpr {
            node,
            variant: "Label",
        };
        let child = PyLabel { name };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyLabel {
    fn __repr__(&self) -> String {
        format!("Label({:?})", self.name)
    }
}

/// A list item: `- item`.
#[pyclass(extends = PyExpr, name = "ListItem", module = "typst.syntax")]
pub struct PyListItem;

impl PyListItem {
    fn create(py: Python<'_>, item: ast::ListItem) -> PyResult<Py<PyAny>> {
        let node = item.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "ListItem",
        };
        let child = PyListItem;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyListItem {
    /// Get the body of this list item as a Markup node.
    #[getter]
    fn body(self_: PyRef<'_, Self>) -> Option<PyMarkup> {
        let parent = self_.as_ref();
        if let Some(item) = parent.node.cast::<ast::ListItem>() {
            Some(PyMarkup(item.body().to_untyped().clone()))
        } else {
            None
        }
    }

    fn __repr__(&self) -> String {
        "ListItem(...)".to_string()
    }
}

/// An enum item: `+ item` or `1. item`.
#[pyclass(extends = PyExpr, name = "EnumItem", module = "typst.syntax")]
pub struct PyEnumItem {
    /// The explicit number, if any.
    #[pyo3(get)]
    number: Option<u64>,
}

impl PyEnumItem {
    fn create(py: Python<'_>, item: ast::EnumItem) -> PyResult<Py<PyAny>> {
        let node = item.to_untyped().clone();
        let number = item.number();

        let parent = PyExpr {
            node,
            variant: "EnumItem",
        };
        let child = PyEnumItem { number };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyEnumItem {
    /// Get the body of this enum item as a Markup node.
    #[getter]
    fn body(self_: PyRef<'_, Self>) -> Option<PyMarkup> {
        let parent = self_.as_ref();
        if let Some(item) = parent.node.cast::<ast::EnumItem>() {
            Some(PyMarkup(item.body().to_untyped().clone()))
        } else {
            None
        }
    }

    fn __repr__(&self) -> String {
        if let Some(n) = self.number {
            format!("EnumItem(number={})", n)
        } else {
            "EnumItem(...)".to_string()
        }
    }
}

/// A term list item: `/ term: description`.
#[pyclass(extends = PyExpr, name = "TermItem", module = "typst.syntax")]
pub struct PyTermItem;

impl PyTermItem {
    fn create(py: Python<'_>, item: ast::TermItem) -> PyResult<Py<PyAny>> {
        let node = item.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "TermItem",
        };
        let child = PyTermItem;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyTermItem {
    /// Get the term as a Markup node.
    #[getter]
    fn term(self_: PyRef<'_, Self>) -> Option<PyMarkup> {
        let parent = self_.as_ref();
        if let Some(item) = parent.node.cast::<ast::TermItem>() {
            Some(PyMarkup(item.term().to_untyped().clone()))
        } else {
            None
        }
    }

    /// Get the description as a Markup node.
    #[getter]
    fn description(self_: PyRef<'_, Self>) -> Option<PyMarkup> {
        let parent = self_.as_ref();
        if let Some(item) = parent.node.cast::<ast::TermItem>() {
            Some(PyMarkup(item.description().to_untyped().clone()))
        } else {
            None
        }
    }

    fn __repr__(&self) -> String {
        "TermItem(...)".to_string()
    }
}

/// A link to a URL: `https://example.com`.
///
/// The URL can be retrieved via the inherited `text()` method.
#[pyclass(extends = PyExpr, name = "Link", module = "typst.syntax")]
pub struct PyLink;

impl PyLink {
    fn create(py: Python<'_>, link: ast::Link) -> PyResult<Py<PyAny>> {
        let node = link.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Link",
        };
        let child = PyLink;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyLink {
    fn __repr__(self_: PyRef<'_, Self>) -> String {
        let parent: &PyExpr = self_.as_ref();
        format!("Link({})", parent.node.text())
    }
}

/// An identifier: `name`.
#[pyclass(extends = PyExpr, name = "Ident", module = "typst.syntax")]
pub struct PyIdent {
    /// The identifier name.
    #[pyo3(get)]
    name: String,
}

impl PyIdent {
    fn create(py: Python<'_>, ident: ast::Ident) -> PyResult<Py<PyAny>> {
        let node = ident.to_untyped().clone();
        let name = ident.get().to_string();

        let parent = PyExpr {
            node,
            variant: "Ident",
        };
        let child = PyIdent { name };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyIdent {
    fn __repr__(&self) -> String {
        format!("Ident({:?})", self.name)
    }
}

/// A none literal: `none`.
#[pyclass(extends = PyExpr, name = "None_", module = "typst.syntax")]
pub struct PyNone;

impl PyNone {
    fn create(py: Python<'_>, none: ast::None) -> PyResult<Py<PyAny>> {
        let node = none.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "None",
        };
        let child = PyNone;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyNone {
    fn __repr__(&self) -> String {
        "None_()".to_string()
    }
}

/// An auto literal: `auto`.
#[pyclass(extends = PyExpr, name = "Auto", module = "typst.syntax")]
pub struct PyAuto;

impl PyAuto {
    fn create(py: Python<'_>, auto: ast::Auto) -> PyResult<Py<PyAny>> {
        let node = auto.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Auto",
        };
        let child = PyAuto;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyAuto {
    fn __repr__(&self) -> String {
        "Auto()".to_string()
    }
}

/// A boolean literal: `true` or `false`.
#[pyclass(extends = PyExpr, name = "Bool", module = "typst.syntax")]
pub struct PyBool {
    /// The boolean value.
    #[pyo3(get)]
    value: bool,
}

impl PyBool {
    fn create(py: Python<'_>, boolean: ast::Bool) -> PyResult<Py<PyAny>> {
        let node = boolean.to_untyped().clone();
        let value = boolean.get();

        let parent = PyExpr {
            node,
            variant: "Bool",
        };
        let child = PyBool { value };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyBool {
    fn __repr__(&self) -> String {
        format!("Bool({})", self.value)
    }
}

/// An integer literal: `123`.
#[pyclass(extends = PyExpr, name = "Int", module = "typst.syntax")]
pub struct PyInt {
    /// The integer value.
    #[pyo3(get)]
    value: i64,
}

impl PyInt {
    fn create(py: Python<'_>, int: ast::Int) -> PyResult<Py<PyAny>> {
        let node = int.to_untyped().clone();
        let value = int.get();

        let parent = PyExpr {
            node,
            variant: "Int",
        };
        let child = PyInt { value };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyInt {
    fn __repr__(&self) -> String {
        format!("Int({})", self.value)
    }
}

/// A floating-point literal: `1.5`.
#[pyclass(extends = PyExpr, name = "Float", module = "typst.syntax")]
pub struct PyFloat {
    /// The float value.
    #[pyo3(get)]
    value: f64,
}

impl PyFloat {
    fn create(py: Python<'_>, float: ast::Float) -> PyResult<Py<PyAny>> {
        let node = float.to_untyped().clone();
        let value = float.get();

        let parent = PyExpr {
            node,
            variant: "Float",
        };
        let child = PyFloat { value };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyFloat {
    fn __repr__(&self) -> String {
        format!("Float({})", self.value)
    }
}

/// A string literal: `"hello"`.
#[pyclass(extends = PyExpr, name = "Str", module = "typst.syntax")]
pub struct PyStr {
    /// The string value.
    #[pyo3(get)]
    value: String,
}

impl PyStr {
    fn create(py: Python<'_>, s: ast::Str) -> PyResult<Py<PyAny>> {
        let node = s.to_untyped().clone();
        let value = s.get().to_string();

        let parent = PyExpr {
            node,
            variant: "Str",
        };
        let child = PyStr { value };

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyStr {
    fn __repr__(&self) -> String {
        format!("Str({:?})", self.value)
    }
}

/// An array expression: `(1, 2, 3)`.
#[pyclass(extends = PyExpr, name = "Array", module = "typst.syntax")]
pub struct PyArray;

impl PyArray {
    fn create(py: Python<'_>, array: ast::Array) -> PyResult<Py<PyAny>> {
        let node = array.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Array",
        };
        let child = PyArray;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyArray {
    /// Get the items in this array.
    fn items(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Vec<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(array) = parent.node.cast::<ast::Array>() {
            array
                .items()
                .filter_map(|item| match item {
                    ast::ArrayItem::Pos(expr) => Some(PyExpr::from_ast(py, expr)),
                    ast::ArrayItem::Spread(spread) => Some(PyExpr::from_ast(py, spread.expr())),
                })
                .collect()
        } else {
            Ok(vec![])
        }
    }

    fn __repr__(&self) -> String {
        "Array(...)".to_string()
    }
}

/// A dictionary expression: `(a: 1, b: 2)`.
#[pyclass(extends = PyExpr, name = "Dict", module = "typst.syntax")]
pub struct PyDict;

impl PyDict {
    fn create(py: Python<'_>, dict: ast::Dict) -> PyResult<Py<PyAny>> {
        let node = dict.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Dict",
        };
        let child = PyDict;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyDict {
    fn __repr__(&self) -> String {
        "Dict(...)".to_string()
    }
}

/// A code block: `{ let x = 1; x + 1 }`.
#[pyclass(extends = PyExpr, name = "CodeBlock", module = "typst.syntax")]
pub struct PyCodeBlock;

impl PyCodeBlock {
    fn create(py: Python<'_>, block: ast::CodeBlock) -> PyResult<Py<PyAny>> {
        let node = block.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "CodeBlock",
        };
        let child = PyCodeBlock;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyCodeBlock {
    fn __repr__(&self) -> String {
        "CodeBlock(...)".to_string()
    }
}

/// A content block: `[*hello* world]`.
#[pyclass(extends = PyExpr, name = "ContentBlock", module = "typst.syntax")]
pub struct PyContentBlock;

impl PyContentBlock {
    fn create(py: Python<'_>, block: ast::ContentBlock) -> PyResult<Py<PyAny>> {
        let node = block.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "ContentBlock",
        };
        let child = PyContentBlock;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyContentBlock {
    /// Get the body of this content block as a Markup node.
    #[getter]
    fn body(self_: PyRef<'_, Self>) -> Option<PyMarkup> {
        let parent = self_.as_ref();
        if let Some(block) = parent.node.cast::<ast::ContentBlock>() {
            Some(PyMarkup(block.body().to_untyped().clone()))
        } else {
            None
        }
    }

    fn __repr__(&self) -> String {
        "ContentBlock(...)".to_string()
    }
}

/// A let binding: `let x = 1`.
#[pyclass(extends = PyExpr, name = "LetBinding", module = "typst.syntax")]
pub struct PyLetBinding;

impl PyLetBinding {
    fn create(py: Python<'_>, binding: ast::LetBinding) -> PyResult<Py<PyAny>> {
        let node = binding.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "LetBinding",
        };
        let child = PyLetBinding;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyLetBinding {
    /// Get the init expression, if any.
    #[getter]
    fn init(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(binding) = parent.node.cast::<ast::LetBinding>() {
            if let Some(expr) = binding.init() {
                return Ok(Some(PyExpr::from_ast(py, expr)?));
            }
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "LetBinding(...)".to_string()
    }
}

/// A set rule: `set text(red)`.
#[pyclass(extends = PyExpr, name = "SetRule", module = "typst.syntax")]
pub struct PySetRule;

impl PySetRule {
    fn create(py: Python<'_>, rule: ast::SetRule) -> PyResult<Py<PyAny>> {
        let node = rule.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "SetRule",
        };
        let child = PySetRule;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PySetRule {
    fn __repr__(&self) -> String {
        "SetRule(...)".to_string()
    }
}

/// A show rule: `show: it => it`.
#[pyclass(extends = PyExpr, name = "ShowRule", module = "typst.syntax")]
pub struct PyShowRule;

impl PyShowRule {
    fn create(py: Python<'_>, rule: ast::ShowRule) -> PyResult<Py<PyAny>> {
        let node = rule.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "ShowRule",
        };
        let child = PyShowRule;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyShowRule {
    fn __repr__(&self) -> String {
        "ShowRule(...)".to_string()
    }
}

/// A conditional expression: `if cond { ... } else { ... }`.
#[pyclass(extends = PyExpr, name = "Conditional", module = "typst.syntax")]
pub struct PyConditional;

impl PyConditional {
    fn create(py: Python<'_>, cond: ast::Conditional) -> PyResult<Py<PyAny>> {
        let node = cond.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Conditional",
        };
        let child = PyConditional;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyConditional {
    /// Get the condition expression.
    #[getter]
    fn condition(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(cond) = parent.node.cast::<ast::Conditional>() {
            return Ok(Some(PyExpr::from_ast(py, cond.condition())?));
        }
        Ok(None)
    }

    /// Get the if-body expression.
    #[getter]
    fn if_body(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(cond) = parent.node.cast::<ast::Conditional>() {
            return Ok(Some(PyExpr::from_ast(py, cond.if_body())?));
        }
        Ok(None)
    }

    /// Get the else-body expression, if any.
    #[getter]
    fn else_body(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(cond) = parent.node.cast::<ast::Conditional>() {
            if let Some(expr) = cond.else_body() {
                return Ok(Some(PyExpr::from_ast(py, expr)?));
            }
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "Conditional(...)".to_string()
    }
}

/// A while loop: `while cond { ... }`.
#[pyclass(extends = PyExpr, name = "WhileLoop", module = "typst.syntax")]
pub struct PyWhileLoop;

impl PyWhileLoop {
    fn create(py: Python<'_>, loop_: ast::WhileLoop) -> PyResult<Py<PyAny>> {
        let node = loop_.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "WhileLoop",
        };
        let child = PyWhileLoop;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyWhileLoop {
    /// Get the condition expression.
    #[getter]
    fn condition(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(loop_) = parent.node.cast::<ast::WhileLoop>() {
            return Ok(Some(PyExpr::from_ast(py, loop_.condition())?));
        }
        Ok(None)
    }

    /// Get the body expression.
    #[getter]
    fn body(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(loop_) = parent.node.cast::<ast::WhileLoop>() {
            return Ok(Some(PyExpr::from_ast(py, loop_.body())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "WhileLoop(...)".to_string()
    }
}

/// A for loop: `for x in iter { ... }`.
#[pyclass(extends = PyExpr, name = "ForLoop", module = "typst.syntax")]
pub struct PyForLoop;

impl PyForLoop {
    fn create(py: Python<'_>, loop_: ast::ForLoop) -> PyResult<Py<PyAny>> {
        let node = loop_.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "ForLoop",
        };
        let child = PyForLoop;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyForLoop {
    /// Get the iterable expression.
    #[getter]
    fn iterable(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(loop_) = parent.node.cast::<ast::ForLoop>() {
            return Ok(Some(PyExpr::from_ast(py, loop_.iterable())?));
        }
        Ok(None)
    }

    /// Get the body expression.
    #[getter]
    fn body(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(loop_) = parent.node.cast::<ast::ForLoop>() {
            return Ok(Some(PyExpr::from_ast(py, loop_.body())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "ForLoop(...)".to_string()
    }
}

/// A module import: `import "file.typ"`.
#[pyclass(extends = PyExpr, name = "ModuleImport", module = "typst.syntax")]
pub struct PyModuleImport;

impl PyModuleImport {
    fn create(py: Python<'_>, import: ast::ModuleImport) -> PyResult<Py<PyAny>> {
        let node = import.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "ModuleImport",
        };
        let child = PyModuleImport;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyModuleImport {
    /// Get the source expression.
    #[getter]
    fn source(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(import) = parent.node.cast::<ast::ModuleImport>() {
            return Ok(Some(PyExpr::from_ast(py, import.source())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "ModuleImport(...)".to_string()
    }
}

/// A module include: `include "file.typ"`.
#[pyclass(extends = PyExpr, name = "ModuleInclude", module = "typst.syntax")]
pub struct PyModuleInclude;

impl PyModuleInclude {
    fn create(py: Python<'_>, include: ast::ModuleInclude) -> PyResult<Py<PyAny>> {
        let node = include.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "ModuleInclude",
        };
        let child = PyModuleInclude;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyModuleInclude {
    /// Get the source expression.
    #[getter]
    fn source(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(include) = parent.node.cast::<ast::ModuleInclude>() {
            return Ok(Some(PyExpr::from_ast(py, include.source())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "ModuleInclude(...)".to_string()
    }
}

/// A function return: `return value`.
#[pyclass(extends = PyExpr, name = "FuncReturn", module = "typst.syntax")]
pub struct PyFuncReturn;

impl PyFuncReturn {
    fn create(py: Python<'_>, ret: ast::FuncReturn) -> PyResult<Py<PyAny>> {
        let node = ret.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "FuncReturn",
        };
        let child = PyFuncReturn;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyFuncReturn {
    /// Get the return value expression, if any.
    #[getter]
    fn body(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(ret) = parent.node.cast::<ast::FuncReturn>() {
            if let Some(expr) = ret.body() {
                return Ok(Some(PyExpr::from_ast(py, expr)?));
            }
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "FuncReturn(...)".to_string()
    }
}

/// A contextual expression: `context text.lang`.
#[pyclass(extends = PyExpr, name = "Contextual", module = "typst.syntax")]
pub struct PyContextual;

impl PyContextual {
    fn create(py: Python<'_>, ctx: ast::Contextual) -> PyResult<Py<PyAny>> {
        let node = ctx.to_untyped().clone();

        let parent = PyExpr {
            node,
            variant: "Contextual",
        };
        let child = PyContextual;

        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyContextual {
    /// Get the body expression.
    #[getter]
    fn body(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(ctx) = parent.node.cast::<ast::Contextual>() {
            return Ok(Some(PyExpr::from_ast(py, ctx.body())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "Contextual(...)".to_string()
    }
}

// ============================================================================
// Math expressions
// ============================================================================

/// Math content in an equation.
#[pyclass(extends = PyExpr, name = "Math", module = "typst.syntax")]
pub struct PyMath;

impl PyMath {
    fn create(py: Python<'_>, math: ast::Math) -> PyResult<Py<PyAny>> {
        let node = math.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "Math",
        };
        let child = PyMath;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyMath {
    fn __repr__(&self) -> String {
        "Math(...)".to_string()
    }
}

/// Text in math: `x`, `25`, `=`.
#[pyclass(extends = PyExpr, name = "MathText", module = "typst.syntax")]
pub struct PyMathText;

impl PyMathText {
    fn create(py: Python<'_>, text: ast::MathText) -> PyResult<Py<PyAny>> {
        let node = text.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "MathText",
        };
        let child = PyMathText;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyMathText {
    fn __repr__(&self) -> String {
        "MathText(...)".to_string()
    }
}

/// An identifier in math: `pi`, `alpha`.
#[pyclass(extends = PyExpr, name = "MathIdent", module = "typst.syntax")]
pub struct PyMathIdent {
    #[pyo3(get)]
    name: String,
}

impl PyMathIdent {
    fn create(py: Python<'_>, ident: ast::MathIdent) -> PyResult<Py<PyAny>> {
        let node = ident.to_untyped().clone();
        let name = ident.get().to_string();
        let parent = PyExpr {
            node,
            variant: "MathIdent",
        };
        let child = PyMathIdent { name };
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyMathIdent {
    fn __repr__(&self) -> String {
        format!("MathIdent({:?})", self.name)
    }
}

/// A shorthand in math: `<=`, `=>`.
#[pyclass(extends = PyExpr, name = "MathShorthand", module = "typst.syntax")]
pub struct PyMathShorthand;

impl PyMathShorthand {
    fn create(py: Python<'_>, shorthand: ast::MathShorthand) -> PyResult<Py<PyAny>> {
        let node = shorthand.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "MathShorthand",
        };
        let child = PyMathShorthand;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyMathShorthand {
    fn __repr__(&self) -> String {
        "MathShorthand(...)".to_string()
    }
}

/// An alignment point in math: `&`.
#[pyclass(extends = PyExpr, name = "MathAlignPoint", module = "typst.syntax")]
pub struct PyMathAlignPoint;

impl PyMathAlignPoint {
    fn create(py: Python<'_>, point: ast::MathAlignPoint) -> PyResult<Py<PyAny>> {
        let node = point.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "MathAlignPoint",
        };
        let child = PyMathAlignPoint;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyMathAlignPoint {
    fn __repr__(&self) -> String {
        "MathAlignPoint()".to_string()
    }
}

/// Delimited math content: `(x + y)`, `{x}`, `[x]`.
#[pyclass(extends = PyExpr, name = "MathDelimited", module = "typst.syntax")]
pub struct PyMathDelimited;

impl PyMathDelimited {
    fn create(py: Python<'_>, delim: ast::MathDelimited) -> PyResult<Py<PyAny>> {
        let node = delim.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "MathDelimited",
        };
        let child = PyMathDelimited;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyMathDelimited {
    fn __repr__(&self) -> String {
        "MathDelimited(...)".to_string()
    }
}

/// Attached scripts in math: `x^2`, `x_1`, `x_1^2`.
#[pyclass(extends = PyExpr, name = "MathAttach", module = "typst.syntax")]
pub struct PyMathAttach;

impl PyMathAttach {
    fn create(py: Python<'_>, attach: ast::MathAttach) -> PyResult<Py<PyAny>> {
        let node = attach.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "MathAttach",
        };
        let child = PyMathAttach;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyMathAttach {
    fn __repr__(&self) -> String {
        "MathAttach(...)".to_string()
    }
}

/// Primes in math: `x'`, `x''`.
#[pyclass(extends = PyExpr, name = "MathPrimes", module = "typst.syntax")]
pub struct PyMathPrimes {
    #[pyo3(get)]
    count: usize,
}

impl PyMathPrimes {
    fn create(py: Python<'_>, primes: ast::MathPrimes) -> PyResult<Py<PyAny>> {
        let node = primes.to_untyped().clone();
        let count = primes.count();
        let parent = PyExpr {
            node,
            variant: "MathPrimes",
        };
        let child = PyMathPrimes { count };
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyMathPrimes {
    fn __repr__(&self) -> String {
        format!("MathPrimes(count={})", self.count)
    }
}

/// A fraction in math: `x / y`.
#[pyclass(extends = PyExpr, name = "MathFrac", module = "typst.syntax")]
pub struct PyMathFrac;

impl PyMathFrac {
    fn create(py: Python<'_>, frac: ast::MathFrac) -> PyResult<Py<PyAny>> {
        let node = frac.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "MathFrac",
        };
        let child = PyMathFrac;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyMathFrac {
    fn __repr__(&self) -> String {
        "MathFrac(...)".to_string()
    }
}

/// A root in math: `√x`, `∛x`.
#[pyclass(extends = PyExpr, name = "MathRoot", module = "typst.syntax")]
pub struct PyMathRoot;

impl PyMathRoot {
    fn create(py: Python<'_>, root: ast::MathRoot) -> PyResult<Py<PyAny>> {
        let node = root.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "MathRoot",
        };
        let child = PyMathRoot;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyMathRoot {
    fn __repr__(&self) -> String {
        "MathRoot(...)".to_string()
    }
}

// ============================================================================
// Additional expressions
// ============================================================================

/// A numeric literal with a unit: `12pt`, `1em`.
#[pyclass(extends = PyExpr, name = "Numeric", module = "typst.syntax")]
pub struct PyNumeric {
    #[pyo3(get)]
    value: f64,
}

impl PyNumeric {
    fn create(py: Python<'_>, numeric: ast::Numeric) -> PyResult<Py<PyAny>> {
        let node = numeric.to_untyped().clone();
        let value = numeric.get().0;
        let parent = PyExpr {
            node,
            variant: "Numeric",
        };
        let child = PyNumeric { value };
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyNumeric {
    fn __repr__(&self) -> String {
        format!("Numeric({})", self.value)
    }
}

/// A parenthesized expression: `(x + y)`.
#[pyclass(extends = PyExpr, name = "Parenthesized", module = "typst.syntax")]
pub struct PyParenthesized;

impl PyParenthesized {
    fn create(py: Python<'_>, paren: ast::Parenthesized) -> PyResult<Py<PyAny>> {
        let node = paren.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "Parenthesized",
        };
        let child = PyParenthesized;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyParenthesized {
    /// Get the inner expression.
    #[getter]
    fn expr(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(paren) = parent.node.cast::<ast::Parenthesized>() {
            return Ok(Some(PyExpr::from_ast(py, paren.expr())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "Parenthesized(...)".to_string()
    }
}

/// A unary operation: `-x`, `not x`.
#[pyclass(extends = PyExpr, name = "Unary", module = "typst.syntax")]
pub struct PyUnary;

impl PyUnary {
    fn create(py: Python<'_>, unary: ast::Unary) -> PyResult<Py<PyAny>> {
        let node = unary.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "Unary",
        };
        let child = PyUnary;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyUnary {
    /// Get the operand expression.
    #[getter]
    fn expr(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(unary) = parent.node.cast::<ast::Unary>() {
            return Ok(Some(PyExpr::from_ast(py, unary.expr())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "Unary(...)".to_string()
    }
}

/// A binary operation: `x + y`, `x and y`.
#[pyclass(extends = PyExpr, name = "Binary", module = "typst.syntax")]
pub struct PyBinary;

impl PyBinary {
    fn create(py: Python<'_>, binary: ast::Binary) -> PyResult<Py<PyAny>> {
        let node = binary.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "Binary",
        };
        let child = PyBinary;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyBinary {
    /// Get the left-hand side expression.
    #[getter]
    fn lhs(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(binary) = parent.node.cast::<ast::Binary>() {
            return Ok(Some(PyExpr::from_ast(py, binary.lhs())?));
        }
        Ok(None)
    }

    /// Get the right-hand side expression.
    #[getter]
    fn rhs(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(binary) = parent.node.cast::<ast::Binary>() {
            return Ok(Some(PyExpr::from_ast(py, binary.rhs())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "Binary(...)".to_string()
    }
}

/// A field access: `x.y`.
#[pyclass(extends = PyExpr, name = "FieldAccess", module = "typst.syntax")]
pub struct PyFieldAccess;

impl PyFieldAccess {
    fn create(py: Python<'_>, access: ast::FieldAccess) -> PyResult<Py<PyAny>> {
        let node = access.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "FieldAccess",
        };
        let child = PyFieldAccess;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyFieldAccess {
    /// Get the target expression.
    #[getter]
    fn target(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(access) = parent.node.cast::<ast::FieldAccess>() {
            return Ok(Some(PyExpr::from_ast(py, access.target())?));
        }
        Ok(None)
    }

    /// Get the field name.
    #[getter]
    fn field(self_: PyRef<'_, Self>) -> Option<String> {
        let parent = self_.as_ref();
        if let Some(access) = parent.node.cast::<ast::FieldAccess>() {
            return Some(access.field().get().to_string());
        }
        None
    }

    fn __repr__(&self) -> String {
        "FieldAccess(...)".to_string()
    }
}

/// A function call: `f(x)`, `f(x, y)`.
#[pyclass(extends = PyExpr, name = "FuncCall", module = "typst.syntax")]
pub struct PyFuncCall;

impl PyFuncCall {
    fn create(py: Python<'_>, call: ast::FuncCall) -> PyResult<Py<PyAny>> {
        let node = call.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "FuncCall",
        };
        let child = PyFuncCall;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyFuncCall {
    /// Get the callee expression.
    #[getter]
    fn callee(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(call) = parent.node.cast::<ast::FuncCall>() {
            return Ok(Some(PyExpr::from_ast(py, call.callee())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "FuncCall(...)".to_string()
    }
}

/// A closure: `(x) => x + 1`.
#[pyclass(extends = PyExpr, name = "Closure", module = "typst.syntax")]
pub struct PyClosure;

impl PyClosure {
    fn create(py: Python<'_>, closure: ast::Closure) -> PyResult<Py<PyAny>> {
        let node = closure.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "Closure",
        };
        let child = PyClosure;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyClosure {
    /// Get the body expression.
    #[getter]
    fn body(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(closure) = parent.node.cast::<ast::Closure>() {
            return Ok(Some(PyExpr::from_ast(py, closure.body())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "Closure(...)".to_string()
    }
}

/// A destructuring assignment: `(a, b) = (1, 2)`.
#[pyclass(extends = PyExpr, name = "DestructAssign", module = "typst.syntax")]
pub struct PyDestructAssign;

impl PyDestructAssign {
    fn create(py: Python<'_>, assign: ast::DestructAssignment) -> PyResult<Py<PyAny>> {
        let node = assign.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "DestructAssign",
        };
        let child = PyDestructAssign;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyDestructAssign {
    /// Get the value expression.
    #[getter]
    fn value(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let parent = self_.as_ref();
        if let Some(assign) = parent.node.cast::<ast::DestructAssignment>() {
            return Ok(Some(PyExpr::from_ast(py, assign.value())?));
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        "DestructAssign(...)".to_string()
    }
}

/// A loop break: `break`.
#[pyclass(extends = PyExpr, name = "LoopBreak", module = "typst.syntax")]
pub struct PyLoopBreak;

impl PyLoopBreak {
    fn create(py: Python<'_>, brk: ast::LoopBreak) -> PyResult<Py<PyAny>> {
        let node = brk.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "LoopBreak",
        };
        let child = PyLoopBreak;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyLoopBreak {
    fn __repr__(&self) -> String {
        "LoopBreak()".to_string()
    }
}

/// A loop continue: `continue`.
#[pyclass(extends = PyExpr, name = "LoopContinue", module = "typst.syntax")]
pub struct PyLoopContinue;

impl PyLoopContinue {
    fn create(py: Python<'_>, cont: ast::LoopContinue) -> PyResult<Py<PyAny>> {
        let node = cont.to_untyped().clone();
        let parent = PyExpr {
            node,
            variant: "LoopContinue",
        };
        let child = PyLoopContinue;
        let init = PyClassInitializer::from(parent).add_subclass(child);
        Ok(Py::new(py, init)?.into_any())
    }
}

#[pymethods]
impl PyLoopContinue {
    fn __repr__(&self) -> String {
        "LoopContinue()".to_string()
    }
}

/// Parse Typst source code into a markup tree.
///
/// Args:
///     input: The Typst source code to parse.
///
/// Returns:
///     The root Markup node of the parsed document.
#[pyfunction]
pub fn parse(input: &str) -> PyMarkup {
    let node = syntax::parse(input);
    PyMarkup(node)
}
