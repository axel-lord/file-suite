# Description of ast

## Arg
    - String/Bytes
    - FString
    - Parenthesized "(..)" `Expr`

## Cmd
    - `Arg`...

## Chain
    - `Src`, `Cmd`..., `Sink`
    - `Src`, `Cmd`...
    - `Cmd`..., `Sink`
    - `Cmd`...,
    - `Src`, `Sink`
    - `Src`

    Sink requires some input.

## Expr
    - `Chain`
    - ...

## Sink
    - File
    - Stdout
    - Stderr
    - Split

## Src
    - File
    - Stdin
    - Join
