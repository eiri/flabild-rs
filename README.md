# flabild

Generator of fake pronounceable words. Port of https://github.com/eiri/flabild

## Summary

This is [Markov chain](https://en.wikipedia.org/wiki/Markov_chain) based generator of fake words. During generation each following letter choosen by a frequency based on two previous letters. This leads to generaton of (semi-) pronounceable words. Frequency module is generated from provided dictionary, allowing for generation of fake words in a given language.

### Name

**flabild** is a generated fake pronounceable word meaning generator of fake pronounceable words.

## Usage

```bash
$ flabild -n 12
an
stlentanes
pose
ser
mer
ble
in
aallingioldwidly
dianta
obbly
mirt
cometal
```

## Dev tools

Trying to use [mise-en-place](https://mise.jdx.dev) and [hk](https://hk.jdx.dev) for this project.

```bash
$ mise task ls
Name     Description
build    Build the CLI
default  Run `run` task
flabild  Run the built binary
lint     Lint the code
run      Run the CLI
test     Test the code
```

## License

[MIT](https://github.com/eiri/flabild-rs/blob/main/LICENSE)
