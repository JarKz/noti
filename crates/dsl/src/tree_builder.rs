trait TreeBuilder {
    const DEFAULT_FIELDS: [&str; 1] = ["key"];
    const TYPE_PATH: &'static str;

    fn validate(&self, /*references: References*/) /* -> Result<(), Error> */;
    fn build(&self) /* -> Option<Widget> */;
}
