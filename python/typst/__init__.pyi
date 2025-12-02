import pathlib
from typing import List, Optional, TypeVar, overload, Dict, Union, Literal, Tuple

Input = TypeVar("Input", str, pathlib.Path, bytes)
OutputFormat = Literal["pdf", "svg", "png", "html"]
PathLike = TypeVar("PathLike", str, pathlib.Path)

class TypstError(RuntimeError):
    """A structured error raised during Typst compilation or querying.

    This exception provides structured access to Typst diagnostics including
    error messages, hints, and stack traces.

    Attributes:
        message (str): The main error message
        hints (list[str]): List of helpful hints for resolving the error
        trace (list[str]): Stack trace information showing error location context
    """

    message: str
    diagnostic: str
    hints: List[str]
    trace: List[str]

    def __init__(
        self,
        message: str,
        diagnostic: str,
        hints: Optional[List[str]] = None,
        trace: Optional[List[str]] = None,
    ) -> None: ...

class TypstWarning(UserWarning):
    """A structured warning raised during Typst compilation.

    This warning provides structured access to Typst warning diagnostics including
    warning messages, hints, and stack traces.

    Attributes:
        message (str): The main warning message
        hints (list[str]): List of helpful hints related to the warning
        trace (list[str]): Stack trace information showing warning location context
    """

    message: str
    diagnostic: str
    hints: List[str]
    trace: List[str]

    def __init__(
        self,
        message: str,
        diagnostic: str,
        hints: Optional[List[str]] = None,
        trace: Optional[List[str]] = None,
    ) -> None: ...

class Fonts:
    def __init__(
        self,
        include_system_fonts: bool = True,
        include_embedded_fonts: bool = True,
        font_paths: List[Input] = [],
    ) -> None: ...

class Compiler:
    def __init__(
        self,
        input: Optional[Input] = None,
        root: Optional[PathLike] = None,
        font_paths: Union[Fonts, List[Input]] = [],
        ignore_system_fonts: bool = False,
        sys_inputs: Dict[str, str] = {},
        pdf_standards: Optional[
            Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]
        ] = [],
        package_path: Optional[PathLike] = None,
    ) -> None:
        """Initialize a Typst compiler.
        Args:
            input: Optional .typ file bytes or path to project's main .typ file. Defaults to an empty in-memory document when omitted.
            root (Optional[PathLike], optional): Root path for the Typst project.
            font_paths (Union[Fonts, List[Input]]): Folders with fonts.
            ignore_system_fonts (bool): Ignore system fonts.
            sys_inputs (Dict[str, str]): string key-value pairs to be passed to the document via sys.inputs
            pdf_standards (Optional[Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]]):
            One or more PDF standard profiles to apply when exporting. Allowed values are `1.7`, `a-2b`, `a-3b`.
            package_path (Optional[PathLike]): Path to load local packages from.
        """

    def compile(
        self,
        input: Optional[Input] = None,
        output: Optional[Input] = None,
        format: Optional[OutputFormat] = None,
        ppi: Optional[float] = None,
    ) -> Optional[Union[bytes, List[bytes]]]:
        """Compile a Typst project.
        Args:
            input: Optional .typ file bytes or path to compile for this invocation.
            output (Optional[PathLike], optional): Path to save the compiled file.
            Allowed extensions are `.pdf`, `.svg` and `.png`
            format (Optional[str]): Output format.
            Allowed values are `pdf`, `svg` and `png`.
            ppi (Optional[float]): Pixels per inch for PNG output, defaults to 144.
        Returns:
            Optional[Union[bytes, List[bytes]]]: Return the compiled file as `bytes` if output is `None`.
        """

    def compile_with_warnings(
        self,
        input: Optional[Input] = None,
        output: Optional[Input] = None,
        format: Optional[OutputFormat] = None,
        ppi: Optional[float] = None,
    ) -> Tuple[Optional[Union[bytes, List[bytes]]], List[TypstWarning]]:
        """Compile a Typst project and return both result and warnings.
        Args:
            input: Optional .typ file bytes or path to compile for this invocation.
            output (Optional[PathLike], optional): Path to save the compiled file.
            Allowed extensions are `.pdf`, `.svg` and `.png`
            format (Optional[str]): Output format.
            Allowed values are `pdf`, `svg` and `png`.
            ppi (Optional[float]): Pixels per inch for PNG output, defaults to 144.
        Returns:
            Tuple[Optional[Union[bytes, List[bytes]]], List[TypstWarning]]: Return a tuple of (compiled_data, warnings).
            The first element is the compiled file as `bytes` if output is `None`, otherwise `None`.
            The second element is a list of structured warnings that occurred during compilation.
        """

    def query(
        self,
        selector: str,
        field: Optional[str] = None,
        one: bool = False,
        format: Optional[Literal["json", "yaml"]] = None,
    ) -> str:
        """Query a Typst document.
        Args:
            selector (str): Typst selector like `<label>`.
            field (Optional[str], optional): Field to query.
            one (bool, optional): Query only one element.
            format (Optional[str]): Output format, `json` or `yaml`.
        Returns:
            str: Return the query result.
        """

@overload
def compile(
    input: Input,
    output: Input,
    root: Optional[Input] = None,
    font_paths: Union[Fonts, List[Input]] = [],
    ignore_system_fonts: bool = False,
    format: Optional[OutputFormat] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[
        Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]
    ] = [],
    package_path: Optional[PathLike] = None,
) -> None: ...
@overload
def compile(
    input: Input,
    output: None = None,
    root: Optional[Input] = None,
    font_paths: Union[Fonts, List[Input]] = [],
    ignore_system_fonts: bool = False,
    format: Optional[OutputFormat] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[
        Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]
    ] = [],
    package_path: Optional[PathLike] = None,
) -> bytes: ...
def compile(
    input: Input,
    output: Optional[Input] = None,
    root: Optional[Input] = None,
    font_paths: Union[Fonts, List[Input]] = [],
    ignore_system_fonts: bool = False,
    format: Optional[OutputFormat] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[
        Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]
    ] = [],
    package_path: Optional[PathLike] = None,
) -> Optional[Union[bytes, List[bytes]]]:
    """Compile a Typst project.
    Args:
        input: .typ file bytes or path to project's main .typ file.
        output (Optional[PathLike], optional): Path to save the compiled file.
        Allowed extensions are `.pdf`, `.svg` and `.png`
        root (Optional[PathLike], optional): Root path for the Typst project.
        font_paths (Union[Fonts, List[Input]]): Folders with fonts.
        ignore_system_fonts (bool): Ignore system fonts
        format (Optional[str]): Output format.
        Allowed values are `pdf`, `svg` and `png`.
        ppi (Optional[float]): Pixels per inch for PNG output, defaults to 144.
        sys_inputs (Dict[str, str]): string key-value pairs to be passed to the document via sys.inputs
        pdf_standards (Optional[Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]]):
        One or more PDF standard profiles to apply when exporting. Allowed values are `1.7`, `a-2b`, `a-3b`.
        package_path (Optional[PathLike]): Path to load local packages from.
    Returns:
        Optional[Union[bytes, List[bytes]]]: Return the compiled file as `bytes` if output is `None`.
    """

@overload
def compile_with_warnings(
    input: Input,
    output: Input,
    root: Optional[Input] = None,
    font_paths: Union[Fonts, List[Input]] = [],
    ignore_system_fonts: bool = False,
    format: Optional[OutputFormat] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[
        Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]
    ] = [],
    package_path: Optional[PathLike] = None,
) -> Tuple[None, List[TypstWarning]]: ...
@overload
def compile_with_warnings(
    input: Input,
    output: None = None,
    root: Optional[Input] = None,
    font_paths: Union[Fonts, List[Input]] = [],
    ignore_system_fonts: bool = False,
    format: Optional[OutputFormat] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[
        Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]
    ] = [],
    package_path: Optional[PathLike] = None,
) -> Tuple[bytes, List[TypstWarning]]: ...
def compile_with_warnings(
    input: Input,
    output: Optional[Input] = None,
    root: Optional[Input] = None,
    font_paths: Union[Fonts, List[Input]] = [],
    ignore_system_fonts: bool = False,
    format: Optional[OutputFormat] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[
        Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]
    ] = [],
    package_path: Optional[PathLike] = None,
) -> Tuple[Optional[Union[bytes, List[bytes]]], List[TypstWarning]]:
    """Compile a Typst project and return warnings.
    Args:
        input: .typ file bytes or path to project's main .typ file.
        output (Optional[PathLike], optional): Path to save the compiled file.
        Allowed extensions are `.pdf`, `.svg` and `.png`
        root (Optional[PathLike], optional): Root path for the Typst project.
        font_paths (Union[Fonts, List[Input]]): Folders with fonts.
        ignore_system_fonts (bool): Ignore system fonts
        format (Optional[str]): Output format.
        Allowed values are `pdf`, `svg` and `png`.
        ppi (Optional[float]): Pixels per inch for PNG output, defaults to 144.
        sys_inputs (Dict[str, str]): string key-value pairs to be passed to the document via sys.inputs
        pdf_standards (Optional[Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]]):
        One or more PDF standard profiles to apply when exporting. Allowed values are `1.7`, `a-2b`, `a-3b`.
        package_path (Optional[PathLike]): Path to load local packages from.
    Returns:
        Optional[Union[bytes, List[bytes]]]: Return the compiled file as `bytes` if output is `None`.
    """

def query(
    input: Input,
    selector: str,
    field: Optional[str] = None,
    one: bool = False,
    format: Optional[Literal["json", "yaml"]] = None,
    root: Optional[Input] = None,
    font_paths: Union[Fonts, List[Input]] = [],
    ignore_system_fonts: bool = False,
    sys_inputs: Dict[str, str] = {},
    package_path: Optional[PathLike] = None,
) -> str:
    """Query a Typst document.
    Args:
        input: .typ file bytes or path to project's main .typ file.
        selector (str): Typst selector like `<label>`.
        field (Optional[str], optional): Field to query.
        one (bool, optional): Query only one element.
        format (Optional[str]): Output format, `json` or `yaml`.
        root (Optional[PathLike], optional): Root path for the Typst project.
        font_paths (Union[Fonts, List[Input]]): Folders with fonts.
        ignore_system_fonts (bool): Ignore system fonts
        sys_inputs (Dict[str, str]): string key-value pairs to be passed to the document via sys.inputs
        package_path (Optional[PathLike]): Path to load local packages from.
    Returns:
        str: Return the query result.
    """
