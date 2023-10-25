# Edge table for the relationship between an account and a confirmation token
DEFINE TABLE has SCHEMAFULL;

DEFINE FIELD in ON TABLE has TYPE record<account>;
DEFINE FIELD out ON TABLE has TYPE record<confirmation_token>;

# Enforces that a user may have only one token.
DEFINE INDEX unique_relationships 
    ON TABLE has 
    COLUMNS in, out UNIQUE;