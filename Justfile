# Just a task runner
# https://github.com/casey/just

# shows this help message
help:
    @just -l

# runs the binary
run *args:
    cargo run -- {{args}}

[private]
alias r := run

# runs tests
test *args:
    cargo test {{args}}

[private]
alias t := test

# generates documentation
doc *args:
    cargo doc {{args}}

[private]
alias d := doc

[private]
do *args:
    cargo doc --open {{args}}

# formats code
format *args:
    cargo fmt {{args}}

[private]
alias f := format
