from ._typst import *


class TypstError(RuntimeError):
    """A structured error raised during Typst compilation or querying.
    
    This exception provides structured access to Typst diagnostics including
    error messages, hints, and stack traces.
    
    Attributes:
        message (str): The main error message
        hints (list[str]): List of helpful hints for resolving the error
        trace (list[str]): Stack trace information showing error location context
    """
    
    def __init__(self, message, hints=None, trace=None):
        super().__init__(message)
        self.message = message
        self.hints = hints or []
        self.trace = trace or []
    
    def __str__(self):
        # Maintain backward compatibility by returning the formatted message
        return self.message


__doc__ = _typst.__doc__
__all__ = _typst.__all__ + ["TypstError"]
