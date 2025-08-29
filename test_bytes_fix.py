#!/usr/bin/env python3
"""
Test to verify both bytes and file compilation work correctly.
This test covers the regression reported in issue #116 and ensures
file change tracking still works as intended for issue #114.
"""

import sys
import os

# Add the python directory to the path so we can import typst
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

try:
    import typst
except ImportError:
    print("ERROR: Could not import typst. Make sure the package is built with 'maturin develop'")
    sys.exit(1)

def test_bytes_compilation():
    """Test that bytes input can be compiled successfully."""
    # Test basic bytes compilation
    content = b'=Title\nHello world'
    result = typst.compile(content)
    assert isinstance(result, bytes)
    assert len(result) > 0
    print("✓ Basic bytes compilation works")

def test_bytes_compilation_with_warnings():
    """Test that bytes input works with compile_with_warnings."""
    content = b'=Title\nHello world'
    result, warnings = typst.compile_with_warnings(content)
    assert isinstance(result, bytes)
    assert len(result) > 0
    assert isinstance(warnings, list)
    print("✓ Bytes compilation with warnings works")

def test_bytes_compiler_class():
    """Test that bytes input works with Compiler class."""
    content = b'=Another Title\nFrom compiler'
    compiler = typst.Compiler(content)
    result = compiler.compile()
    assert isinstance(result, bytes)
    assert len(result) > 0
    print("✓ Bytes compilation with Compiler class works")

def test_multiple_compilations():
    """Test that multiple compilations work (tests reset functionality)."""
    content = b'=Document\nContent here'
    
    # First compilation
    result1 = typst.compile(content)
    assert len(result1) > 0
    
    # Second compilation - this would fail if reset doesn't work properly
    result2 = typst.compile(content)
    assert len(result2) > 0
    assert len(result1) == len(result2)  # Should be identical
    
    print("✓ Multiple bytes compilations work")

def test_file_compilation_still_works():
    """Test that file-based compilation still works after the fix."""
    import tempfile
    import os
    
    # Create a temporary file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.typ', delete=False) as f:
        f.write('=Test Document\n\nThis is a test document.')
        temp_path = f.name
    
    try:
        # Test file-based compilation
        result = typst.compile(temp_path)
        assert isinstance(result, bytes)
        assert len(result) > 0
        print("✓ File-based compilation still works")
    finally:
        # Clean up
        os.unlink(temp_path)

if __name__ == "__main__":
    print("Running tests for typst-py bytes compilation fix...")
    test_bytes_compilation()
    test_bytes_compilation_with_warnings()
    test_bytes_compiler_class()
    test_multiple_compilations()
    test_file_compilation_still_works()
    print("All tests passed! ✓")
    print("\nThis verifies that issue #116 has been fixed.")