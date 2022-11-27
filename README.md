# ConciseMark - Concise Markdown

ConciseMark is a simplified markdown parsing library.

## Usage

    let content = "# Title";
    let parser = Parser::new(content);
    let page = parser.parse();
    let ast = page.ast;
    let html = page.to_html();

The `ast` of the markdown is showed as below

    Node {
        data: RefCell {
            value: NodeData {
                tag: "div",
                range: 0..8,
                parent: (Weak),
                children: [
                    RefCell {
                        value: NodeData {
                            tag: "h1",
                            range: 0..8,
                            parent: (Weak),
                            children: [
                                RefCell {
                                    value: NodeData {
                                        tag: "text",
                                        range: 1..8,
                                        parent: (Weak),
                                        children: [],
                                    },
                                },
                            ],
                        },
                    },
                ],
            },
        },
    }

The `html` renderd using `ast` will be

    <div><h1> Title </h1></div>
