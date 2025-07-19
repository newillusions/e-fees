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
DEFINE FIELD name ON company TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name_short ON company TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD reg_no ON company TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD tax_no ON company TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD time ON company TYPE object PERMISSIONS FULL;
DEFINE FIELD time.created_at ON company TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON company TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;



-- ------------------------------
-- TABLE: contacts
-- ------------------------------

DEFINE TABLE contacts TYPE NORMAL SCHEMAFULL PERMISSIONS FULL;

DEFINE FIELD company ON contacts TYPE record<company> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD email ON contacts TYPE string ASSERT string::is::email($value) PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD first_name ON contacts TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD full_name ON contacts TYPE string VALUE string::concat(first_name, ' ', last_name) PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD last_name ON contacts TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD phone ON contacts TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD position ON contacts TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD time ON contacts TYPE object PERMISSIONS FULL;
DEFINE FIELD time.created_at ON contacts TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON contacts TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;



-- ------------------------------
-- TABLE: country
-- ------------------------------

DEFINE TABLE country TYPE NORMAL SCHEMAFULL PERMISSIONS FULL;

DEFINE FIELD code ON country TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD code_alt ON country TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD currency_code ON country FLEXIBLE TYPE record<currency> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD dial_code ON country TYPE number PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name ON country TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name_formal ON country TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name_official ON country TYPE string PERMISSIONS FOR select, create, update WHERE FULL;



-- ------------------------------
-- TABLE: currency
-- ------------------------------

DEFINE TABLE currency TYPE NORMAL SCHEMAFULL PERMISSIONS FULL;

DEFINE FIELD code ON currency TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name ON currency TYPE string PERMISSIONS FOR select, create, update WHERE FULL;



-- ------------------------------
-- TABLE: projects
-- ------------------------------

DEFINE TABLE projects TYPE NORMAL SCHEMAFULL PERMISSIONS FULL;

DEFINE FIELD activity ON projects TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD area ON projects TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD city ON projects TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD country ON projects TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD folder ON projects TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name ON projects TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD number ON projects TYPE object PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD number.country ON projects TYPE number PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD number.id ON projects TYPE string PERMISSIONS FULL;
DEFINE FIELD number.seq ON projects TYPE number PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD number.year ON projects TYPE number PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD package ON projects TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD status ON projects TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD time ON projects TYPE object PERMISSIONS FULL;
DEFINE FIELD time.created_at ON projects TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON projects TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;



-- ------------------------------
-- TABLE: rfp
-- ------------------------------

DEFINE TABLE rfp TYPE NORMAL SCHEMAFULL PERMISSIONS FULL;

DEFINE FIELD company_id ON rfp TYPE record<company> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD contact_id ON rfp TYPE record<contacts> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD issue_date ON rfp TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name ON rfp TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD number ON rfp TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD project_id ON rfp TYPE record<projects> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD rev ON rfp TYPE int DEFAULT 1 PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD strap_line ON rfp TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD time ON rfp TYPE object PERMISSIONS FULL;
DEFINE FIELD time.created_at ON rfp TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON rfp TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

DEFINE INDEX id ON rfp FIELDS contact_id UNIQUE;


