import pathlib
from typing import List, Optional, TypeVar

PathLike = TypeVar("PathLike", str, pathlib.Path)

def compile(
    input: PathLike,
    output: Optional[PathLike] = None,
    root: Optional[PathLike] = None,
    font_paths: List[PathLike] = [],
) -> Optional[bytes]: ...
