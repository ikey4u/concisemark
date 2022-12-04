# ConciseMark

ConciseMark is a simplified markdown parsing library written in Rust with customization, math supporting
in mind. The `simplified` here means it only supports some common markdown syntax but not full.
Here is a list it supports for now

- codeblock and inline codeblock
- heading
- list
- link
- image
- math (supported by [katex](https://katex.org/))

    Use `$a^2 + b^2$` to write math equation: $a^2 + b^2$

- extension

    This is a ConciseMark extension, the syntax is

        @KEY{VALUE}

    It supports the following `KEY`

    - emoji

        `@emoji{smile}` will render to 😄.

    - math

        `@math{a^2 + b^2}` is the same as `$a^2 + b^2$`. 

    - kbd

        `@kbd{cmd + f}` will render to `⌘+f`.

    This syntax can be easily extended by library user.

With the continuous development of ConciseMark, this list will grow, see its
[documentation](https://docs.rs/concisemark/0.2.0/concisemark/index.html) for usage.
