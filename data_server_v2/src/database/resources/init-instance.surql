# The Rush instances that belong to an account;
DEFINE TABLE instance SCHEMAFULL
    PERMISSIONS FOR create, update, select, delete
        WHERE account = $auth.id;

# Name of the instance
# Todo: consider what a valid name should be
DEFINE FIELD name ON instance
    TYPE string
    ASSERT string::len($value) >= 0;

# Reference to the account that owns the instance
DEFINE FIELD account ON instance
    TYPE record(account);

# Need to know what database this instance exists on.
# Needed because dev and UAT instances need to be hosted on a separate instance
# of SurrealDB than production instances so that load testing can be carried out
# without impacting the performance of production systems
DEFINE FIELD db_url ON instance
    TYPE string
    ASSERT string::is::url($value)
    DEFAULT type::string($DB_URL)
    # This may not be updated
    PERMISSIONS NONE;

# Valid instance types
DEFINE PARAM $INSTANCE_TYPE VALUE ["trial", "dev", "uat", "prod"];

# Track what kind of instance this is - trial, dev, uat, prod
DEFINE FIELD instance_type ON instance
    TYPE string
    ASSERT $value INSIDE $INSTANCE_TYPE
    DEFAULT "trial"
    PERMISSIONS NONE;

# Need to determine the namespace in the surreal instance
# TODO: this should actually be a field on the account as an account will have a dedicated namespace
# TODO: consider automatically generating this field from the account's account name
DEFINE FIELD namespace ON instance TYPE option<string> DEFAULT function() {
	let query = new surrealdb.Query("SELECT VALUE name FROM $id", { id: this.account });
	return (await surrealdb.query(query))[0];
} PERMISSIONS NONE;

# The name of the database where the instance lives
DEFINE FIELD database ON instance
    TYPE string
    # We automatically generate the instace
    # TODO: consider simply having the instance name be the instance type
    DEFAULT function() {
  	    let query = new surrealdb.Query("SELECT VALUE name FROM $id", { id: this.account });
	    let account_name = (await surrealdb.query(query))[0];
	    return account_name + "_" + this.instance_type;
    }
    PERMISSIONS NONE;
    
DEFINE FIELD created_at ON TABLE instance
    TYPE datetime
    PERMISSIONS NONE
    DEFAULT time::now();

DEFINE FIELD created_by ON TABLE instance
    TYPE record(account)
    DEFAULT $auth.id OR account:system
    PERMISSIONS NONE;

DEFINE FIELD updated_at ON TABLE instance
    TYPE datetime
    PERMISSIONS NONE
    DEFAULT time::now();

DEFINE FIELD updated_by ON TABLE instance
    TYPE record(account)
    PERMISSIONS NONE
    DEFAULT $auth.id OR account:system;

DEFINE INDEX unique_name ON TABLE instance
    COLUMNS name
    UNIQUE;

# Relate the account to the instance when created
DEFINE EVENT relate_account ON TABLE instance
    WHEN $event = "CREATE" THEN {
	    UPDATE $after.account SET instances += $after.id;
    };

# Track changes on table
DEFINE EVENT on_update ON TABLE account
    WHEN $event = "UPDATE" THEN {
	    $after.updated_at = time::now();
	    $after.updated_by = $auth.id OR account:system;
    };