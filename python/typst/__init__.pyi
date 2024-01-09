import pathlib
from typing import List, Optional, TypeVar, overload

PathLike = TypeVar("PathLike", str, pathlib.Path)

class Compiler:
    def __init__(
        self,
        input: PathLike,
        root: Optional[PathLike] = None,
        font_paths: List[PathLike] = [],
    ) -> None:
        """Initialize a Typst compiler.
        Args:
            input (PathLike): Project's main .typ file.
            root (Optional[PathLike], optional): Root path for the Typst project.
            font_paths (List[PathLike]): Folders with fonts.
        """
    def compile(
        self,
        output: Optional[PathLike] = None,
        format: Optional[str] = None,
        ppi: Optional[float] = None,
    ) -> Optional[bytes]:
        """Compile a Typst project.
        Args:
            output (Optional[PathLike], optional): Path to save the compiled file.
            Allowed extensions are `.pdf`, `.svg` and `.png`
            format (Optional[str]): Output format.
            Allowed values are `pdf`, `svg` and `png`.
            ppi (Optional[float]): Pixels per inch for PNG output, defaults to 144.
        Returns:
            Optional[bytes]: Return the compiled file as `bytes` if output is `None`.
        """

@overload
def compile(
    input: PathLike,
    output: PathLike,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
    format: Optional[str] = None,
    ppi: Optional[float] = None,
) -> None: ...
@overload
def compile(
    input: PathLike,
    output: None = None,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
    format: Optional[str] = None,
    ppi: Optional[float] = None,
) -> bytes: ...
def compile(
    input: PathLike,
    output: Optional[PathLike] = None,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
    format: Optional[str] = None,
    ppi: Optional[float] = None,
) -> Optional[bytes]:
    """Compile a Typst project.
    Args:
        input (PathLike): Project's main .typ file.
        output (Optional[PathLike], optional): Path to save the compiled file.
        Allowed extensions are `.pdf`, `.svg` and `.png`
        root (Optional[PathLike], optional): Root path for the Typst project.
        font_paths (List[PathLike]): Folders with fonts.
        format (Optional[str]): Output format.
        Allowed values are `pdf`, `svg` and `png`.
        ppi (Optional[float]): Pixels per inch for PNG output, defaults to 144.
    Returns:
        Optional[bytes]: Return the compiled file as `bytes` if output is `None`.
    """
