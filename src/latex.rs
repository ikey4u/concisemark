#[derive(Debug)]
pub struct Env {
    pub name: String,
    pub posargs: Vec<String>,
    pub optargs: Vec<String>,
    pub body: String,
}

impl Env {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            posargs: vec![],
            optargs: vec![],
            body: String::new(),
        }
    }

    pub fn with_posarg<S: AsRef<str>>(mut self, posarg: S) -> Self {
        self.posargs.push(posarg.as_ref().to_owned());
        self
    }

    pub fn with_optarg<S: AsRef<str>>(mut self, optarg: S) -> Self {
        self.optargs.push(optarg.as_ref().to_owned());
        self
    }

    pub fn with_body<S: AsRef<str>>(mut self, body: S) -> Self {
        self.body = body.as_ref().to_owned();
        self
    }

    pub fn append<S: AsRef<str>>(&mut self, content: S) {
        self.body.push_str(content.as_ref())
    }

    pub fn append_cmd(&mut self, cmd: &Cmd) {
        self.body.push_str(&cmd.to_string())
    }

    pub fn to_string(&self) -> String {
        let mut content = format!(r#"\begin{{{}}}"#, self.name);
        content.push('\n');
        for optarg in self.optargs.iter() {
            content.push_str(&format!("[{}]", optarg));
        }
        for posarg in self.posargs.iter() {
            content.push_str(&format!("{{{}}}", posarg));
        }
        content.push_str(&self.body);
        content.push_str(&format!(r#"\end{{{}}}"#, self.name));
        content.push('\n');
        content
    }
}

#[derive(Debug)]
pub struct Cmd {
    pub name: String,
    pub posargs: Vec<String>,
    pub optargs:Vec<String>,
}

impl Cmd {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            posargs: vec![],
            optargs: vec![],
        }
    }

    pub fn with_posarg<S: AsRef<str>>(mut self, posarg: S) -> Self {
        self.posargs.push(posarg.as_ref().to_owned());
        self
    }

    pub fn with_optarg<S: AsRef<str>>(mut self, optarg: S) -> Self {
        self.optargs.push(optarg.as_ref().to_owned());
        self
    }

    pub fn to_string(&self) -> String {
        let mut content = format!(r#"\{}"#, self.name);
        for optarg in self.optargs.iter() {
            content.push_str(&format!("[{}]", optarg));
        }
        for posarg in self.posargs.iter() {
            content.push_str(&format!("{{{}}}", posarg));
        }
        content.push('\n');
        content
    }
}
