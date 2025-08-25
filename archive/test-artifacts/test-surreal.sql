-- Test database connectivity and baseline data
SELECT "Connection test successful" as status;

-- Get production baseline (should not contain any DELETE ME records)
SELECT COUNT() as production_projects FROM projects WHERE name NOT CONTAINS "DELETE ME";
SELECT COUNT() as production_companies FROM company WHERE name NOT CONTAINS "DELETE ME"; 
SELECT COUNT() as production_contacts FROM contacts WHERE first_name NOT CONTAINS "DELETE ME";
SELECT COUNT() as production_fees FROM rfp WHERE name NOT CONTAINS "DELETE ME";

-- Create test company with DELETE ME identifier
CREATE company CONTENT {
  name: "DELETE ME - Test Company [2025-08-22T15:30:45] TEST-validation01",
  name_short: "TESTCO", 
  city: "Test City",
  country: "United Arab Emirates",
  email: "test@example.com",
  phone: "+971-50-123-4567"
};

-- Read the test company back
SELECT * FROM company WHERE name CONTAINS "DELETE ME";

-- Create test contact linked to company
CREATE contacts CONTENT {
  first_name: "DELETE ME - John [2025-08-22T15:30:45] TEST-validation01",
  last_name: "Doe",
  full_name: "DELETE ME - John [2025-08-22T15:30:45] TEST-validation01 Doe",
  email: "john.doe@example.com", 
  phone: "+971-50-987-6543",
  position: "Test Manager",
  company: (SELECT id FROM company WHERE name CONTAINS "DELETE ME" LIMIT 1)[0].id
};

-- Read the test contact back
SELECT * FROM contacts WHERE first_name CONTAINS "DELETE ME";

-- Create test project 
CREATE projects CONTENT {
  name: "DELETE ME - Test Project [2025-08-22T15:30:45] TEST-validation01",
  status: "Draft",
  city: "Test City", 
  country: "United Arab Emirates"
};

-- Read the test project back
SELECT * FROM projects WHERE name CONTAINS "DELETE ME";

-- Count all test data created
SELECT COUNT() as test_records_created FROM (
  SELECT * FROM projects WHERE name CONTAINS "DELETE ME"
  UNION
  SELECT * FROM company WHERE name CONTAINS "DELETE ME"
  UNION 
  SELECT * FROM contacts WHERE first_name CONTAINS "DELETE ME"
);

-- Clean up all test data
DELETE FROM contacts WHERE first_name CONTAINS "DELETE ME";
DELETE FROM company WHERE name CONTAINS "DELETE ME";
DELETE FROM projects WHERE name CONTAINS "DELETE ME";

-- Verify cleanup - should return 0
SELECT COUNT() as remaining_test_data FROM (
  SELECT * FROM projects WHERE name CONTAINS "DELETE ME"
  UNION
  SELECT * FROM company WHERE name CONTAINS "DELETE ME"
  UNION
  SELECT * FROM contacts WHERE first_name CONTAINS "DELETE ME"
);

-- Final validation - production data should be unchanged
SELECT COUNT() as final_production_projects FROM projects WHERE name NOT CONTAINS "DELETE ME";
SELECT COUNT() as final_production_companies FROM company WHERE name NOT CONTAINS "DELETE ME";
SELECT COUNT() as final_production_contacts FROM contacts WHERE first_name NOT CONTAINS "DELETE ME";
SELECT COUNT() as final_production_fees FROM rfp WHERE name NOT CONTAINS "DELETE ME";

SELECT "CRUD validation test completed successfully" as final_status;