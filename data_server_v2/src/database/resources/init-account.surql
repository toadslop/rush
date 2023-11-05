# Defines the account table. An account represents a person or organization
# who owns Rush instances.
DEFINE TABLE account SCHEMAFULL
	PERMISSIONS FOR select, update
        # Only the owner of the account may modify the account. They may not delete it.
        WHERE id = $auth.id;

# The account name is a human-readable identifier for the account.
# This will typically be a person's name or a company's name.
DEFINE FIELD name ON account TYPE string
    # Prevent a user from submitting too much text and wasting memory
	ASSERT string::len($value) <= 50
	PERMISSIONS FOR select, create, update
		WHERE id = $auth.id;

DEFINE FIELD email ON account TYPE string
	ASSERT string::is::email($value) AND string::len($value) <= 320
    # an email may not be modified after creation
	PERMISSIONS FOR select, create
		WHERE id = $auth.id;

# emails must be unique
DEFINE INDEX email ON account FIELDS email UNIQUE;

DEFINE FIELD password ON account TYPE string
    # TODO: consider removing update/create permissions and make a dedicated update password function
	VALUE crypto::argon2::generate($value)
	PERMISSIONS FOR select, create, update
		WHERE id = $auth.id;

# An account must have have a confirmed email before it can create instances.
DEFINE FIELD confirmed ON account TYPE bool
    # This field may only be updated by the system
	PERMISSIONS NONE
	DEFAULT false;

# Keep track of the instances associated with this account.
# TODO: consider grouping instances so an account could have multiple solutions -- each with dev, uat, and prod 
DEFINE FIELD instances ON account TYPE array
	PERMISSIONS FOR select, create, update
		WHERE id = $auth.id
	DEFAULT [];

DEFINE FIELD instances.* ON TABLE account TYPE record(instance);

DEFINE FIELD created_at ON TABLE account TYPE datetime
	PERMISSIONS NONE
	DEFAULT time::now();

DEFINE FIELD created_by ON TABLE account TYPE record(account)
    PERMISSIONS NONE
    # account:system is how we determine actions performed by the system
    DEFAULT $auth.id OR account:system;

DEFINE FIELD updated_at ON TABLE account TYPE datetime
	PERMISSIONS NONE
	DEFAULT time::now();

DEFINE FIELD updated_by ON TABLE account TYPE record(account)
	PERMISSIONS NONE
	DEFAULT $auth.id OR account:system;

# Track changes on table
DEFINE EVENT on_update ON TABLE account
    WHEN $event = "UPDATE" THEN {
	    $after.updated_at = time::now();
	    $after.updated_by = $auth.id OR account:system;
    };

# Issue a confirmation token when the account is created
DEFINE EVENT issue_token ON TABLE account WHEN $event = "CREATE" THEN {
	let $token_ = CREATE confirmation_token;
	RELATE ONLY $after->has->$token_;
};

# The system account
CREATE account:system CONTENT {
	email: "system@rush.io", # todo: system email account needs to be configurable
 	password: "testtest",
 	name: "system",
 	confirmed: true,
 	instances: [],
};

# Hnadles confirming an account and related error states
DEFINE FUNCTION fn::confirm_account($conf_token: uuid) {
    # Lookup the account based on the provided token
	LET $account_token = (SELECT <-has<-account.*, id, created_at FROM confirmation_token WHERE token = $conf_token FETCH account)[0];
	
    # Means an invalid token was submitted
	IF $account_token IS NONE THEN {
		THROW "Requested token not found";
	} END;

	LET $account = $account_token["<-has"]["<-account"];

    # Means the token is valid but no account for the token exist.
    # This means there is a bug -- there should always be an account for the token.
	IF $account IS NONE THEN {
		DELETE $account_token;
		THROW "No account found for token";
	} END;

	# Check to see if the token is expired or not
	IF $account_token.created_at + $TOKEN_EXPIRATION < time::now() THEN {
		DELETE $account_token;
		THROW "Token is expired. Please reissue."
	} END;

	$account = UPDATE $account MERGE {
		confirmed: true
	};
	
	RETURN $account;
};

-- BEGIN TRANSACTION;
--             LET $saved = CREATE ONLY account CONTENT $account;
--             LET $conf_token = (SELECT ->has->confirmation_token.token FROM $saved)[0]["->has"]["->confirmation_token"].token[0];
--             RETURN {
--                 account: $saved,
--                 token: $conf_token
--             };
--             COMMIT TRANSACTION;

# Scope for account login
DEFINE SCOPE account SESSION 15m
	SIGNIN (
		SELECT * FROM account WHERE email = $email AND crypto::argon2::compare(password, $password)
	)
	SIGNUP (
        CREATE ONLY type::thing("account", $email) CONTENT {
				name: $name,
				email: $email,
				password: crypto::argon2::generate($password)
		}		
	)
;