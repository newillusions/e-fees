-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- TABLE: company
-- ------------------------------

DEFINE TABLE company TYPE NORMAL SCHEMAFULL PERMISSIONS FULL;

DEFINE FIELD abbreviation ON company TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD city ON company TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD country ON company TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name ON company TYPE string ASSERT string::len($value) > 0 PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name_short ON company TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD reg_no ON company TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD tax_no ON company TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD time ON company TYPE object PERMISSIONS FULL;
DEFINE FIELD time.created_at ON company TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON company TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

DEFINE INDEX company_abbreviation ON company FIELDS abbreviation;
DEFINE INDEX company_country ON company FIELDS country;
DEFINE INDEX company_country_city ON company FIELDS country, city;
DEFINE INDEX company_created ON company FIELDS time.created_at;
DEFINE INDEX company_name ON company FIELDS name;


-- ------------------------------
-- TABLE DATA: company
-- ------------------------------

