#!/usr/bin/env fish

# Check for correct number of arguments
if test (count $argv) -ne 1
    echo "Usage: $argv[0] <C++ file>"
    exit 1
end

set SOURCE_FILE $argv[1]

# Ensure the file exists
if not test -f $SOURCE_FILE
    echo "Error: File $SOURCE_FILE does not exist"
    exit 1
end

# Determine the operating system
set OS (uname)
if test "$OS" = "Darwin"
    # macOS: Use mktemp to create a temporary file
    set EXECUTABLE (mktemp -t tmp_cpp_exec)
else
    # Linux: Use /dev/shm to store the executable
    set EXECUTABLE "/dev/shm/tmp_cpp_exec"
end

# Use clang++ to compile and run (in-memory compilation, no disk access)
clang++ -x c++ $SOURCE_FILE -o $EXECUTABLE -pipe -std=c++20 && $EXECUTABLE

# Clean up the executable in memory (ensure no garbage remains)
rm -f $EXECUTABLE
