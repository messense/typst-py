import pathlib
from typing import List, Optional, TypeVar, overload, Dict, Union, Literal

Input = TypeVar("Input", str, pathlib.Path, bytes)
OutputFormat = Literal["pdf", "svg", "png", "html"]

class Compiler:
    def __init__(
        self,
        input: Input,
        root: Optional[Input] = None,
        font_paths: List[Input] = [],
        ignore_system_fonts: bool = False,
        sys_inputs: Dict[str, str] = {},
        pdf_standards: Optional[Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]] = []
    ) -> None:
        """Initialize a Typst compiler.
        Args:
            input: .typ file bytes or path to project's main .typ file.
            root (Optional[PathLike], optional): Root path for the Typst project.
            font_paths (List[PathLike]): Folders with fonts.
            ignore_system_fonts (bool): Ignore system fonts.
            sys_inputs (Dict[str, str]): string key-value pairs to be passed to the document via sys.inputs
        """

    def compile(
        self,
        output: Optional[Input] = None,
        format: Optional[OutputFormat] = None,
        ppi: Optional[float] = None,
    ) -> Optional[Union[bytes, List[bytes]]]:
        """Compile a Typst project.
        Args:
            output (Optional[PathLike], optional): Path to save the compiled file.
            Allowed extensions are `.pdf`, `.svg` and `.png`
            format (Optional[str]): Output format.
            Allowed values are `pdf`, `svg` and `png`.
            ppi (Optional[float]): Pixels per inch for PNG output, defaults to 144.
        Returns:
            Optional[Union[bytes, List[bytes]]]: Return the compiled file as `bytes` if output is `None`.
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
    font_paths: List[Input] = [],
    ignore_system_fonts: bool = False,
    format: Optional[OutputFormat] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]] = []
) -> None: ...
@overload
def compile(
    input: Input,
    output: None = None,
    root: Optional[Input] = None,
    font_paths: List[Input] = [],
    ignore_system_fonts: bool = False,
    format: Optional[OutputFormat] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]] = []
) -> bytes: ...
def compile(
    input: Input,
    output: Optional[Input] = None,
    root: Optional[Input] = None,
    font_paths: List[Input] = [],
    ignore_system_fonts: bool = False,
    format: Optional[OutputFormat] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[Union[Literal["1.7", "a-2b", "a-3b"], List[Literal["1.7", "a-2b", "a-3b"]]]] = []
) -> Optional[Union[bytes, List[bytes]]]:
    """Compile a Typst project.
    Args:
        input: .typ file bytes or path to project's main .typ file.
        output (Optional[PathLike], optional): Path to save the compiled file.
        Allowed extensions are `.pdf`, `.svg` and `.png`
        root (Optional[PathLike], optional): Root path for the Typst project.
        font_paths (List[PathLike]): Folders with fonts.
        ignore_system_fonts (bool): Ignore system fonts
        format (Optional[str]): Output format.
        Allowed values are `pdf`, `svg` and `png`.
        ppi (Optional[float]): Pixels per inch for PNG output, defaults to 144.
        sys_inputs (Dict[str, str]): string key-value pairs to be passed to the document via sys.inputs
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
    font_paths: List[Input] = [],
    ignore_system_fonts: bool = False,
    sys_inputs: Dict[str, str] = {},
) -> str:
    """Query a Typst document.
    Args:
        input: .typ file bytes or path to project's main .typ file.
        selector (str): Typst selector like `<label>`.
        field (Optional[str], optional): Field to query.
        one (bool, optional): Query only one element.
        format (Optional[str]): Output format, `json` or `yaml`.
        root (Optional[PathLike], optional): Root path for the Typst project.
        font_paths (List[PathLike]): Folders with fonts.
        ignore_system_fonts (bool): Ignore system fonts
        sys_inputs (Dict[str, str]): string key-value pairs to be passed to the document via sys.inputs
    Returns:
        str: Return the query result.
    """
