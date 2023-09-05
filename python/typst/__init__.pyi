import pathlib
from typing import List, Optional, TypeVar, overload

PathLike = TypeVar("PathLike", str, pathlib.Path)

@overload
def compile(
    input: PathLike,
    output: PathLike,
    root: Optional[PathLike] = None,
    font_paths: Optional[List[PathLike]] = None,
) -> None: ...

@overload
def compile(
    input: PathLike,
    output: None = None,
    root: Optional[PathLike] = None,
    font_paths: Optional[List[PathLike]] = None,
) -> bytes: ...


def compile(
    input: PathLike,
    output: Optional[PathLike] = None,
    root: Optional[PathLike] = None,
    font_paths: Optional[List[PathLike]] = None,
) -> Optional[bytes]: ...
