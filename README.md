# File Suite
Collection of personal utilites created as needed, intentions exists for everything to
be available in a single binary to avoid cluttering up PATH.

Todo:
- [ ] Everything should compile, and function.
- [ ] Make everything available from `file-suite` executable.
- [ ] Tests

## file-suite-proc-impl
Procedural macros for use in other crates, mostly me playing around.

Todo:
- [ ] String format function of some kind.
- [ ] Cheap to copy tokenstream, using rc and including groups.
- [ ] Quote-like token format function, in addition to the existing array expression paste function.
- [ ] Some method of passing parameters to aliases, perhaps as locals somehow.
- [x] Tests for at least all chain functions.
- [x] Allow variables to be used in most places expecting arguments.
- [ ] Punctuation type.
- [x] DefaultArgs instead of default for arguments with default values.
- [x] Better error messages for situations similar to `LookaheadParse::optional_parse`.
- [ ] Change LookaheadParse signature to take `Lookahead1` as mut ref and drive it forward on success.
- [x] Move function tests to respective modules.
- [ ] Split up into multiple crates.
- [ ] Separate array expressions from `syn`, `quote` and `proc_macro2`

Function Todo:
- [x] Take (negative value should result in back elements being taken)
- [x] Chain
- [x] Skip (negative value should result in back elements being skipped)
- [x] Intersperse
- [x] Get (variable, not by index)
- [x] Nth (with error on failure, and negative indexing)
- [ ] ~~Every (could be emulated by .chunks(N, .take(1)))~~
- [x] Block (Chain with same locals as parent)
