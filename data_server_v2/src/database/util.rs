pub struct Transaction<'a> {
    queries: Vec<&'a str>,
}

impl<'a> Transaction<'a> {
    const BEGIN_TX: &str = "BEGIN TRANSACTION;";
    const COMMIT_TX: &str = "COMMIT TRANSACTION;";

    pub fn new() -> Self {
        Self {
            queries: vec![Self::BEGIN_TX],
        }
    }

    pub fn add_query(mut self, query: &'a str) -> Self {
        self.queries.push(query);
        self
    }

    pub fn build(mut self) -> String {
        self.queries.push(Self::COMMIT_TX);
        self.queries.join("\n")
    }
}

impl<'a> Default for Transaction<'a> {
    fn default() -> Self {
        Self::new()
    }
}
