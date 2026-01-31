#!/usr/bin/env python3
"""
Example demonstrating different approaches for compiling multi-file Typst projects.

This is particularly useful when your Typst files are bundled as Python package resources.
"""

import typst
import tempfile
from pathlib import Path


def example1_dict_based():
    """
    Example 1: Dictionary-based approach (recommended for bundled packages)
    
    This is the simplest and most efficient method when files are already in memory
    as bytes (e.g., from importlib.resources.read_binary).
    """
    print("=" * 70)
    print("Example 1: Dictionary-based approach")
    print("=" * 70)
    
    # Define your Typst files as bytes
    main_content = b'''
#import "lib.typ": greet, farewell
#import "utils.typ": format_name

= Multi-file Project Example

#greet(format_name("Alice"))

#farewell(format_name("Bob"))
'''
    
    lib_content = b'''
#let greet(name) = [Hello, #name! Welcome to our document.]

#let farewell(name) = [Goodbye, #name! See you next time.]
'''
    
    utils_content = b'''
#let format_name(name) = text(weight: "bold", fill: blue)[#name]
'''
    
    # Create a dictionary mapping filenames to content
    files = {
        "main": main_content,      # The main file (can also use "main.typ")
        "lib.typ": lib_content,    # Imported library
        "utils.typ": utils_content, # Another imported file
    }
    
    # Compile the multi-file project
    pdf = typst.compile(files, format="pdf")
    
    print(f"✅ Successfully compiled! PDF size: {len(pdf)} bytes")
    print(f"   Files included: {', '.join(files.keys())}")
    
    # You can also save to a file
    with open("/tmp/multifile_example1.pdf", "wb") as f:
        f.write(pdf)
    print(f"   Saved to: /tmp/multifile_example1.pdf")
    print()


def example2_temp_directory():
    """
    Example 2: Temporary directory approach
    
    This is useful when you want to work with actual files, such as when using
    importlib.resources.as_file or when you need to write files to disk first.
    """
    print("=" * 70)
    print("Example 2: Temporary directory approach")
    print("=" * 70)
    
    # Same content as example 1
    files_content = {
        "main.typ": b'#import "lib.typ": greet\n= Document\n#greet("World")',
        "lib.typ": b'#let greet(name) = [Hello, #name!]',
    }
    
    with tempfile.TemporaryDirectory() as tmpdir:
        print(f"Creating temporary directory: {tmpdir}")
        
        # Write all files to the same temporary directory
        for filename, content in files_content.items():
            filepath = Path(tmpdir) / filename
            filepath.write_bytes(content)
            print(f"  Written: {filename}")
        
        # Compile using the main file path
        main_path = Path(tmpdir) / "main.typ"
        pdf = typst.compile(str(main_path), format="pdf")
        
        print(f"✅ Successfully compiled! PDF size: {len(pdf)} bytes")
        
    print("   Temporary directory cleaned up")
    print()


def example3_simulating_package_resources():
    """
    Example 3: Simulating importlib.resources usage
    
    This shows how you would use this with actual package resources.
    """
    print("=" * 70)
    print("Example 3: Simulating package resources")
    print("=" * 70)
    
    # In a real scenario, you would use:
    # import importlib.resources
    # content = importlib.resources.read_binary("mypackage.typst_files", "main.typ")
    
    # For this example, we'll simulate having package resources
    def read_resource(filename):
        """Simulate importlib.resources.read_binary"""
        resources = {
            "main.typ": b'#import "templates.typ": header\n#header()\n= My Report\nContent here.',
            "templates.typ": b'#let header() = [= Company Header\n_Generated Report_\n]',
        }
        return resources[filename]
    
    # Approach 3a: Using dictionary (recommended)
    print("Approach 3a: Dictionary-based (recommended)")
    files = {}
    for filename in ["main.typ", "templates.typ"]:
        files[filename] = read_resource(filename)
    
    pdf = typst.compile(files, format="pdf")
    print(f"✅ Compiled using dict: {len(pdf)} bytes")
    
    # Approach 3b: Using temporary directory
    print("\nApproach 3b: Temporary directory")
    with tempfile.TemporaryDirectory() as tmpdir:
        for filename in ["main.typ", "templates.typ"]:
            content = read_resource(filename)
            (Path(tmpdir) / filename).write_bytes(content)
        
        pdf = typst.compile(str(Path(tmpdir) / "main.typ"), format="pdf")
        print(f"✅ Compiled using temp dir: {len(pdf)} bytes")
    
    print()


def example4_compiler_reuse():
    """
    Example 4: Reusing Compiler instance with multiple file sets
    
    This is useful when you need to compile multiple different projects
    or the same project with different content.
    """
    print("=" * 70)
    print("Example 4: Reusing Compiler instance")
    print("=" * 70)
    
    # Create a compiler instance
    compiler = typst.Compiler()
    
    # First project
    files1 = {
        "main": b'#import "lib.typ": f1\n= Project 1\n#f1()',
        "lib.typ": b'#let f1() = [Function from project 1]',
    }
    
    result1 = compiler.compile(input=files1, format="pdf")
    print(f"✅ Project 1 compiled: {len(result1)} bytes")
    
    # Second project with different files
    files2 = {
        "main": b'#import "lib.typ": f2\n= Project 2\n#f2()',
        "lib.typ": b'#let f2() = [Function from project 2]',
    }
    
    result2 = compiler.compile(input=files2, format="pdf")
    print(f"✅ Project 2 compiled: {len(result2)} bytes")
    
    print()


if __name__ == "__main__":
    print("\n")
    print("╔" + "═" * 68 + "╗")
    print("║" + " " * 15 + "Multi-file Typst Projects in Python" + " " * 18 + "║")
    print("╚" + "═" * 68 + "╝")
    print()
    
    example1_dict_based()
    example2_temp_directory()
    example3_simulating_package_resources()
    example4_compiler_reuse()
    
    print("=" * 70)
    print("All examples completed successfully!")
    print("=" * 70)
    print("\nRecommendation:")
    print("  - For bundled packages: Use the dictionary-based approach (Example 1)")
    print("  - For file-based workflows: Use regular file paths")
    print("  - Only use temp directories when absolutely necessary")
    print()
