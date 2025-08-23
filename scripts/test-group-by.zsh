#!/usr/bin/env zsh
emulate -L zsh

print -- cmdline: "$@"
print -- inputs:

while IFS=$'\0' read -rd $'\0' LINE; do
	print -- '  -' $LINE
done
