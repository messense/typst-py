import pathlib
from typing import List, Optional, TypeVar

PathLike = TypeVar("PathLike", str, pathlib.Path)

class Compiler:
    def __init__(self, root: PathLike, font_paths: List[PathLike] = []) -> None: ...
    def compile(self, input: PathLike, output: Optional[PathLike] = None) -> Optional[bytes]: ...

def compile(
    input: PathLike,
    output: Optional[PathLike] = None,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
) -> Optional[bytes]: ...
