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
- [ ] Python
- [x] Rust
- [ ] Zig
- [ ] C#
- [ ] Go
- [ ] Java
- [ ] TypeScript
- [x] Shell (sh / bash)
