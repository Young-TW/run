# Run

Compile/Interpret and Execute any language code snippets quickly without creating any files on disk.

## Usage

```sh
run <source-file>
```

The language is detected from the file extension, then the snippet is
compiled (into `/dev/shm` on Linux, no persistent disk access) and executed,
or interpreted directly for scripting languages. The program's exit code is
propagated.

## Supported Languages

- [x] C++
- [x] C
- [x] Python
- [x] Rust
- [ ] Zig
- [ ] C# (needs the `dotnet` runtime)
- [x] Go
- [x] Java
- [x] JavaScript
- [x] Ruby
- [x] TypeScript
- [x] Shell (sh / bash)

The runtime is selected from the file extension, so `.sh` runs with `sh`,
`.bash` with `bash`, `.ts` with the first available of `bun` / `tsx` /
`ts-node`, and so on. Interpreted languages fall back through a list of
candidate runtimes when the preferred one is not installed.

## Exit Codes

When the program itself runs, `run` propagates the program's own exit code.
Otherwise it exits consistently and safely:

| Code | Meaning |
|------|---------|
| `0`  | Program ran successfully |
| _program's code_ | Program ran and exited with that code |
| `1`  | A compiled language failed to compile |
| `2`  | The language / file extension is not supported |
| `127`| No execution environment found (the required compiler or interpreter is not installed) |
