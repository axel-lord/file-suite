# Description of ast

## Arg
    - String/Bytes
    - FString
    - Parenthesized "(..)" `Expr`

## Cmd
    - `Arg`...

## Chain
    - `Src`, `Cmd`..., `Sink`
    - `Cmd`..., `Sink`
    - `Src`, `Sink`

    Sink requires some input, is required, and is implied to be Default if missing.

## Expr
    - `Chain`
    - ...

## Sink
    - Default
    - File
    - Stdout
    - Stderr
    - Split

## Src
    - File
    - Stdin
    - Join
