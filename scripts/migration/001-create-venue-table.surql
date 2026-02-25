-- 001-create-venue-table.surql
-- Domain Model Restructure: Phase 1 — Create venue table

DEFINE TABLE venue SCHEMAFULL;

DEFINE FIELD name ON venue TYPE string ASSERT string::len($value) > 0;
DEFINE FIELD name_short ON venue TYPE string DEFAULT '';
DEFINE FIELD location ON venue TYPE object DEFAULT {};
DEFINE FIELD location.city ON venue TYPE string DEFAULT '';
DEFINE FIELD location.country ON venue TYPE string DEFAULT '';
DEFINE FIELD location.area ON venue TYPE string DEFAULT '';
DEFINE FIELD tags ON venue TYPE array<string> DEFAULT [];
DEFINE FIELD notes ON venue TYPE string DEFAULT '';
DEFINE FIELD time ON venue TYPE object VALUE { created_at: time::now(), updated_at: time::now() };
DEFINE FIELD time.created_at ON venue TYPE datetime VALUE $before OR time::now();
DEFINE FIELD time.updated_at ON venue TYPE datetime VALUE time::now();

DEFINE INDEX venue_name ON venue FIELDS name;
DEFINE INDEX venue_country ON venue FIELDS location.country;
