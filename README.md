# ConciseMark [(documentation)](https://docs.rs/concisemark/0.3.1/concisemark/index.html)

ConciseMark is a simplified markdown parsing library written in Rust with customization, math supporting
in mind. The `simplified` here means it only supports some common markdown syntax but not full.
For Chinese introduction, please click [ConciseMark 中文介绍](https://zhqli.com/post/1670199332).

## Features

- Latex PDF Generation

    ConciseMark supports you to convert your markdown into xelatex source file, then you can compile it
    with xelatex command to generate a pretty PDF document.

    Note that to make the generated xelatex source compilable, you have to install the following
    fonts onto your system

    - [Lora](https://fonts.google.com/specimen/Lora)
    - [Source Code Pro](https://fonts.google.com/specimen/Source+Code+Pro?category=Monospace)
    - [Source Han Serif SC](https://github.com/adobe-fonts/source-han-serif/releases)

- markdown meta

    You can put an optional html comment (whose body is in toml format) in the front of your markdown file

        <!---
        title = "Your title"
        subtitle = "Your subtitle"
        date = "2021-10-13 00:00:00"
        authors = ["name <example@gmail.com>"]
        tags = ["demo", "example"]
        -->

    This content will be parsed as your page meta, you can use it when rendering latex or html page.

- codeblock and inline codeblock
- heading
- list
- link
- image
- math (Math in HTML is supported by [katex](https://katex.org/))

    Use one syntax such as `$a^2 + b^2$` to write inline or display mode math equation.

- extension

    This is a ConciseMark extension, the format is

        @KEY{VALUE}

    It supports the following `KEY`

    - emoji

        `@emoji{smile}` will render to 😄.

    - kbd

        `@kbd{cmd + f}` will render to `⌘+f`.

    This feature will be exposed to library user in future.
