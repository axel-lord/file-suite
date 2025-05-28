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
7. [ ] Punctuation type.
8. [x] DefaultArgs instead of default for arguments with default values.
9. [x] Better error messages for situations similar to `LookaheadParse::optional_parse`.
10. [ ] Change LookaheadParse signature to take `Lookahead1` as mut ref and drive it forward on success.

Function Todo:
1. [ ] Take (negative value should result in back elements being taken)
2. [ ] Chain
3. [ ] Skip (negative value should result in back elements being skipped)
3. [ ] Intersperse
4. [ ] Get (variable, not by index)
5. [ ] Nth (with error on failure, and negative indexing)
6. [ ] Every (could be emulated by .chunks(N, .take(1)))
7. [ ] Block (Chain with same locals as parent)
