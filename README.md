# ConciseMark [(documentation)](https://docs.rs/concisemark/0.3.1/concisemark/index.html)

ConciseMark is a simplified markdown parsing library written in Rust with customization, math supporting
in mind. The `simplified` here means it only supports some common markdown syntax but not full.
For Chinese introduction, please click [ConciseMark 中文介绍](https://zhqli.com/post/1670199332).

## Features

- Basic

    - Emphasis

        Use `*Itaclics*` to write itaclic text.

        Use `**Itaclics**` to write bold text.

    - Paragraph

        A paragraph consists of a sequence lines without empty line between
        them.

    - Heading

        Use `#`, `##`, ... `######` to indicate one to six level heading.

        `#` must be the first character in your heading line.

        As a thumb of rule, you should avoid heading level greating than three.

    - Blockquote

        A blockquote is paragraph starts with `>` symbol, and `>` must be the
        first character of the paragraph, for example

            > a simple blockquote
            with very *long* body
            **really long** body ...

        If you want to show empty line in blockquote, you can do like the
        following

            > a simple line
            >
            > line
            test

    - List

        Use `-` to indicate a list item, nested list is also supported.

        Indentation between two adjacent lists must be 4.

        Numbered list is not supported yet.

    - Link && Image

        Use `[google](https://google.com)` to dispaly a link.

        Use `![image](https://example.com/some.jpg)` to dispaly an image.

    - Code

        Use backtick (`) pair to show inline mode code.

        Indent your code in a new paragraph with indention more than
        four spaces than your current indention, then it will be dispaly mode. 

- Extension

    - Math

        Math in HTML is supported by [katex](https://katex.org/)), use one
        syntax such as `$a^2 + b^2$` to write math equation.

        If `$a^2 + b^2` is in paragraph, then it will be inline mode.

        If `$a^2 + b^2` holds the full paragraph, then it will be display mode.

    - Latex PDF Generation

        ConciseMark supports you to convert your markdown into xelatex source file, then you can compile it
        with xelatex command to generate a pretty PDF document.

        Note that to make the generated xelatex source compilable, you have to install the following
        fonts onto your system

        - [Lora](https://fonts.google.com/specimen/Lora)
        - [Source Code Pro](https://fonts.google.com/specimen/Source+Code+Pro?category=Monospace)
        - [Source Han Serif SC](https://github.com/adobe-fonts/source-han-serif/releases)

    - Markdown Meta

        You can put an optional html comment (whose body is in toml format) in the front of your markdown file

            <!---
            title = "Your title"
            subtitle = "Your subtitle"
            date = "2021-10-13 00:00:00"
            authors = ["name <example@gmail.com>"]
            tags = ["demo", "example"]
            -->

        This content will be parsed as your page meta, you can use it when rendering latex or html page.
