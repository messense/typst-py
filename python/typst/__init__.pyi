import pathlib
from typing import List, Optional, TypeVar, overload, Dict, Union, Literal

PathLike = TypeVar("PathLike", str, pathlib.Path)

class Compiler:
    def __init__(
        self,
        input: PathLike,
        root: Optional[PathLike] = None,
        font_paths: List[PathLike] = [],
        ignore_system_fonts: bool = False,
        sys_inputs: Dict[str, str] = {},
        pdf_standards: Optional[Union[Literal["1.7", "a-2b"], List[Literal["1.7", "a-2b"]]]] = []
    ) -> None:
        """Initialize a Typst compiler.
        Args:
            input (PathLike): Project's main .typ file.
            root (Optional[PathLike], optional): Root path for the Typst project.
            font_paths (List[PathLike]): Folders with fonts.
            ignore_system_fonts (bool): Ignore system fonts.
            sys_inputs (Dict[str, str]): string key-value pairs to be passed to the document via sys.inputs
        """

    def compile(
        self,
        output: Optional[PathLike] = None,
        format: Optional[Literal["pdf", "svg", "png"]] = None,
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
    input: PathLike,
    output: PathLike,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
    ignore_system_fonts: bool = False,
    format: Optional[Literal["pdf", "svg", "png"]] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[Union[Literal["1.7", "a-2b"], List[Literal["1.7", "a-2b"]]]] = []
) -> None: ...
@overload
def compile(
    input: PathLike,
    output: None = None,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
    ignore_system_fonts: bool = False,
    format: Optional[Literal["pdf", "svg", "png"]] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[Union[Literal["1.7", "a-2b"], List[Literal["1.7", "a-2b"]]]] = []
) -> bytes: ...
def compile(
    input: PathLike,
    output: Optional[PathLike] = None,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
    ignore_system_fonts: bool = False,
    format: Optional[Literal["pdf", "svg", "png"]] = None,
    ppi: Optional[float] = None,
    sys_inputs: Dict[str, str] = {},
    pdf_standards: Optional[Union[Literal["1.7", "a-2b"], List[Literal["1.7", "a-2b"]]]] = []
) -> Optional[Union[bytes, List[bytes]]]:
    """Compile a Typst project.
    Args:
        input (PathLike): Project's main .typ file.
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
    input: PathLike,
    selector: str,
    field: Optional[str] = None,
    one: bool = False,
    format: Optional[Literal["json", "yaml"]] = None,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
    ignore_system_fonts: bool = False,
    sys_inputs: Dict[str, str] = {},
) -> str:
    """Query a Typst document.
    Args:
        input (PathLike): Project's main .typ file.
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
