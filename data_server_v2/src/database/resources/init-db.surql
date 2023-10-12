BEGIN TRANSACTION;

# Initialize root namespace and database
# This database and namespace store account and instance metadata
DEFINE NAMESPACE root;
DEFINE DATABASE root;

# Accounts are for those who own purchased instances
DEFINE TABLE account SCHEMAFULL
	PERMISSIONS
		FOR select, update WHERE id = $auth.id;

DEFINE FIELD name ON account TYPE string PERMISSIONS FOR select, create, update WHERE id = $auth.id;
# email may not be changed
DEFINE FIELD email ON account TYPE string ASSERT string::is::email($value) PERMISSIONS NONE;
DEFINE FIELD password ON account TYPE string PERMISSIONS FOR select, create, update WHERE id = $auth.id;
DEFINE FIELD confirmed ON account TYPE bool PERMISSIONS NONE DEFAULT false;
DEFINE FIELD instances ON account TYPE array PERMISSIONS FOR select, create, update WHERE id = $auth.id;
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
DEFINE TABLE instance SCHEMAFULL;

DEFINE FIELD name ON instance TYPE string;
DEFINE INDEX nameIndex ON TABLE instance COLUMNS name UNIQUE;

COMMIT TRANSACTION;