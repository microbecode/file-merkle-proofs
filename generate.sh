#!/bin/bash

# Use this script to generate dummy files for testing

# Number of files to generate
num_files=51

# Directory to store the files
dir="client_storage"

# Create the directory if it doesn't exist
mkdir -p "$dir"

# Generate files
for i in $(seq 1 $num_files)
do
    filename="$dir/file$i.txt"
    echo "This is file $i" > "$filename"
    echo "Created $filename"
done

echo "Generated $num_files files in the $dir directory."