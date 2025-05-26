# File Suite
Collection of personal utilites created as needed, intentions exists for everything to
be available in a single binary to avoid cluttering up PATH.

Todo:
1. [ ] Everything should compile, and function.
2. [ ] Make everything available from `file-suite` executable.
3. [ ] Tests

## file-suite-proc-impl
Procedural macros for use in other crates, mostly me playing around.

Todo:
1. [ ] String format function of some kind.
2. [ ] Cheap to copy tokenstream, using rc and including groups.
3. [ ] Quote-like token format function, in addition to the existing array expression paste function.
4. [ ] Some method of passing parameters to aliases, perhaps as locals somehow.
5. [ ] Tests for at least all chain functions.
6. [ ] Allow variables to be used in most places expecting arguments.
