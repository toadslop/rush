# Defines the length of time that a confirmation token is valid
DEFINE PARAM $TOKEN_EXPIRATION VALUE 1d;

# Confirmation tokens for accounts
DEFINE TABLE confirmation_token SCHEMAFULL
    PERMISSIONS NONE;

# Used to determine if the token is still valid
DEFINE FIELD created_at ON TABLE confirmation_token
    TYPE datetime
    # a token may not be modified
    PERMISSIONS NONE
    DEFAULT time::now();


DEFINE FIELD token ON TABLE confirmation_token
    TYPE uuid
    PERMISSIONS NONE
    # randomly generate the token when it is created
    DEFAULT rand::uuid::v4();