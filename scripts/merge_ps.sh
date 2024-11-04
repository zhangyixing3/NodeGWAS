#!/bin/bash

# Checks if input argument is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <input_pattern>"
    exit 1
fi

# The input pattern is provided through command line argument
input_pattern=$1

# Execute AWG command, where we find all matching files sorted by chromosome number
awk '
BEGIN { print "SNP\tCHR\tBP\tP" }
{
  chr = "unknown";  # Default chromosome number
  split(FILENAME, parts, "/");  # Splits the path
  if (length(parts) > 1 && parts[2] ~ /^[0-9]+$/) chr = parts[2];  # Checks if the directory name is a number
  print $1, chr, FNR*15, $4
}' OFS="\t" $(find . -type f -path "$input_pattern" | sort -t '/' -k2,2n)
