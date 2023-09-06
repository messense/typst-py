import pathlib
from typing import List, Optional, TypeVar, overload

PathLike = TypeVar("PathLike", str, pathlib.Path)

@overload
def compile(
    input: PathLike,
    output: PathLike,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
) -> None: ...
@overload
def compile(
    input: PathLike,
    output: None = None,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
) -> bytes: ...
def compile(
    input: PathLike,
    output: Optional[PathLike] = None,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
) -> Optional[bytes]:
    """Compile a Typst project.

    Args:
        input (PathLike): Projet's main .typ file.
        output (Optional[PathLike], optional): Path to save the compiled file.
        Allowed extensions are `.pdf`, `.svg` and `.png`
        root (Optional[PathLike], optional): Root path for the Typst project.
        font_paths (List[PathLike]): Folders with fonts.

    Returns:
        Optional[bytes]: Return the compiled file as `bytes` if output is `None`.
    """
