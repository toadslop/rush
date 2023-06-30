use std::fmt::Display;

use surrealdb::{engine::remote::ws::Client, opt::auth::Root, sql::Op, Surreal};

pub trait Initialize {
    fn initialize(&self) -> String;
}

pub struct DbInitializer {
    namespace: String,
    database: String,
    tables: Vec<TableInitializer>,
}

impl DbInitializer {
    const BEGIN_TRANSACTION: &str = "BEGIN TRANSATION;\n";
    const COMMIT_TRANSACTION: &str = "COMMIT TRANSATION;\n";

    pub fn new(namespace: String, database: String) -> Self {
        Self {
            namespace,
            database,
            tables: Vec::new(),
        }
    }

    pub fn table(mut self, table: TableInitializer) -> Self {
        self.tables.push(table);
        self
    }

    fn define_ns(&self) -> String {
        let ns = &self.namespace;
        format!("DEFINE NAMESPACE {ns};\n")
    }

    fn use_ns(&self) -> String {
        let ns = &self.namespace;
        format!("USE NAMESPACE {ns};\n")
    }

    fn define_db(&self) -> String {
        let db = &self.database;
        format!("DEFINE DATABASE {db};\n")
    }

    fn use_db(&self) -> String {
        let db = &self.database;
        format!("USE DATABASE {db};\n")
    }
}

impl Initialize for DbInitializer {
    fn initialize(&self) -> String {
        let mut query = String::from(Self::BEGIN_TRANSACTION);
        query.push_str(&self.define_ns());
        query.push_str(&self.use_ns());
        query.push_str(&self.define_db());
        query.push_str(&self.use_db());

        query.push_str(Self::COMMIT_TRANSACTION);
        query
    }
}

pub enum Schema {
    Schemafull,
    Schemaless,
}

impl Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Schema::Schemafull => write!(f, "SCHEMAFULL"),
            Schema::Schemaless => write!(f, "SCHEMALESS"),
        }
    }
}

pub struct TableInitializer {
    table_name: String,
    drop: bool,
    schema: Schema,
    fields: Vec<FieldInitializer>,
}

impl TableInitializer {
    pub fn new(table_name: String, drop: bool, schema: Schema) -> Self {
        Self {
            table_name,
            drop,
            schema,
            fields: Vec::new(),
        }
    }

    pub fn field(&mut self, mut field: FieldInitializer) {
        field.table_name = Some(self.table_name.clone());
        self.fields.push(field);
    }

    fn define_table(&self) -> String {
        let table = &self.table_name;
        let drop = self.drop_txt();
        let schema = self.schema_txt();
        format!("DEFINE TABLE {table}{drop}{schema};\n")
    }

    fn drop_txt(&self) -> String {
        if self.drop {
            " DROP".into()
        } else {
            "".into()
        }
    }

    fn schema_txt(&self) -> String {
        self.schema.to_string()
    }
}

impl Initialize for TableInitializer {
    fn initialize(&self) -> String {
        self.define_table()
    }
}

pub struct FieldInitializer {
    pub name: String,
    pub data_type: SurrealType,
    pub value: Option<String>,
    pub assert: Option<String>,
    pub table_name: Option<String>,
}

impl FieldInitializer {
    pub fn new(
        name: String,
        data_type: SurrealType,
        value: Option<String>,
        assert: Option<String>,
    ) -> Self {
        Self {
            name,
            data_type,
            value,
            assert,
            table_name: None,
        }
    }

    pub fn on_table_txt(&self) -> String {
        format!(" ON TABLE {}", self.table_name.clone().unwrap_or_default())
    }

    pub fn type_txt(&self) -> String {
        format!(" TYPE {}", self.data_type)
    }

    pub fn value_txt(&self) -> String {
        format!(" VALUE {}", self.data_type)
    }

    pub fn define_field(&self) -> String {
        let field = &self.name;
        let on_table = self.on_table_txt();
        let type_text = self.type_txt();
        let value_text = self.value_txt();
        format!("DEFINE FIELD{on_table}{type_text}{value_text};\n")
    }
}

impl Initialize for FieldInitializer {
    fn initialize(&self) -> String {
        todo!()
    }
}

pub enum SurrealType {
    String,
    Datetime,
    Object,
    Array,
    Bool,
    Record(TableInitializer),
}

impl Display for SurrealType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SurrealType::String => write!(f, "string"),
            SurrealType::Datetime => write!(f, "datetime"),
            SurrealType::Object => write!(f, "object"),
            SurrealType::Array => write!(f, "array"),
            SurrealType::Bool => write!(f, "bool"),
            SurrealType::Record(table) => write!(f, "record ({})", table.table_name),
        }
    }
}
