pub struct CliArgument {
    name: String,
    description: Option<String>,
    optional: bool,
    variadic: bool,
}

impl CliArgument {
    pub fn new(arg_name: String) -> Self {
        Self {
            name: arg_name,
            description: None,
            optional: false,
            variadic: false,
        }
    }
    pub fn optional(&mut self) -> &mut Self {
        self.optional = true;
        self
    }
    pub fn variadic(&mut self) -> &mut Self {
        self.variadic = true;
        self
    }
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }
}
