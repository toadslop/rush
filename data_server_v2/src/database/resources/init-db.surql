BEGIN TRANSACTION;

# Initialize root namespace and database
# This database and namespace store account and instance metadata
DEFINE NAMESPACE root;
DEFINE DATABASE root;

# Accounts are for those who own purchased instances
DEFINE TABLE account SCHEMAFULL
	PERMISSIONS
		FOR select, update WHERE id = $auth.id;

DEFINE FIELD name ON account TYPE string ASSERT string::len($value) <= 50 PERMISSIONS FOR select, create, update WHERE id = $auth.id;
# email may not be changed
DEFINE FIELD email ON account TYPE string ASSERT string::is::email($value) AND string::len($value) <= 320 PERMISSIONS FOR select, create WHERE id = $auth.id;
DEFINE FIELD password ON account TYPE string PERMISSIONS FOR select, create, update WHERE id = $auth.id ASSERT string::len($value) >= 8 AND string::len($value) <= 16;
DEFINE FIELD confirmed ON account TYPE bool PERMISSIONS NONE DEFAULT false;
DEFINE FIELD instances ON account TYPE array PERMISSIONS FOR select, create, update WHERE id = $auth.id DEFAULT [];
DEFINE FIELD instances.* ON TABLE account TYPE record(instance);
DEFINE INDEX email ON account FIELDS email UNIQUE;

# account scope
DEFINE SCOPE account SESSION 15m
	SIGNIN (
		SELECT * FROM account WHERE email = $email AND crypto::argon2::compare(password, $password)
	)
	SIGNUP (
		CREATE account CONTENT {
			name: $name,
			email: $email,
			password: crypto::argon2::generate($password)
		}
	);

# Instance table
DEFINE PARAM $INSTANCE_TYPE VALUE ["trial", "dev", "uat", "prod"];
DEFINE PARAM $DB_URL VALUE "localhost:8000"; # TODO: handle database URLs dynamically
DEFINE TABLE instance SCHEMAFULL;

DEFINE FIELD name ON instance TYPE string ASSERT string::len($value) >= 8;
DEFINE FIELD account ON instance TYPE record(account);
DEFINE FIELD db_url ON instance TYPE option<string> ASSERT string::is::url($value) VALUE type::string($DB_URL) PERMISSIONS NONE;
DEFINE FIELD instance_type ON instance TYPE string ASSERT $value INSIDE $INSTANCE_TYPE DEFAULT "trial" PERMISSIONS NONE;;
DEFINE FIELD namespace ON instance TYPE option<string> VALUE function() {
	let query = new surrealdb.Query("SELECT VALUE name FROM $id", { id: this.account });
	return (await surrealdb.query(query))[0];
} PERMISSIONS NONE;
DEFINE FIELD database ON instance TYPE option<string> VALUE function() {
  	let query = new surrealdb.Query("SELECT VALUE name FROM $id", { id: this.account });
	let account_name = (await surrealdb.query(query))[0];
	return account_name + "_" + this.instance_type;
} PERMISSIONS NONE;

DEFINE INDEX nameIndex ON TABLE instance COLUMNS name UNIQUE;
DEFINE EVENT relate_account ON TABLE instance WHEN $event = "CREATE" THEN {
	UPDATE $after.account SET instances += $after.id;
};
DEFINE EVENT set_database ON TABLE instance WHEN $event = "CREATE" THEN {
	
};

COMMIT TRANSACTION;