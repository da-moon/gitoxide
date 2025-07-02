#!/bin/bash
# Print the last non-empty line of input
awk 'NF{line=$0} END{print line}'
