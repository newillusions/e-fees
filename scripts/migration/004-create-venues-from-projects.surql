-- 004-create-venues-from-projects.surql
-- Domain Model Restructure: Phase 3 — Create venue records from existing project data
--
-- INSTRUCTIONS:
--   1. Run the venue CREATEs in STEP 1
--   2. Run the project UPDATEs in STEP 2 to link projects to venues
--   3. Run the diagnostic query in STEP 3 to find unlinked projects
--   4. Manually assign remaining projects to venues
--
-- NOTE: This script covers known multi-engagement venues.
-- Projects with unique venues need individual assignment (see STEP 3).

-- ============================================================================
-- STEP 1: Create venue records for known locations
-- ============================================================================

-- MAF MiCC — Majid Al Futtaim kids entertainment centre (multiple engagements)
CREATE venue:maf_micc CONTENT {
    name: "MAF MiCC",
    name_short: "MiCC",
    location: { city: "Dubai", country: "UAE", area: "" },
    tags: ["entertainment", "retail"],
    notes: "Majid Al Futtaim kids entertainment centre — multiple design phases"
};

-- Shanghai Tang — Retail space
CREATE venue:shanghai_tang CONTENT {
    name: "Shanghai Tang",
    name_short: "Shanghai Tang",
    location: { city: "Dubai", country: "UAE", area: "" },
    tags: ["retail"],
    notes: ""
};

-- Lvl 63 — Restaurant (formerly Reserve Cut)
CREATE venue:lvl_63 CONTENT {
    name: "Lvl 63",
    name_short: "Lvl 63",
    location: { city: "Dubai", country: "UAE", area: "" },
    tags: ["restaurant", "hospitality"],
    notes: "Formerly known as Reserve Cut"
};

-- RAK Beach District
CREATE venue:rak_beach CONTENT {
    name: "RAK Beach District",
    name_short: "RAK Beach",
    location: { city: "Ras Al Khaimah", country: "UAE", area: "" },
    tags: ["hospitality", "mixed-use"],
    notes: ""
};

-- ============================================================================
-- STEP 2: Link projects to venues
-- Adjust project IDs to match your actual data
-- ============================================================================

-- MAF MiCC engagements
-- UPDATE projects SET venue_id = venue:maf_micc WHERE name CONTAINS 'MiCC' OR name CONTAINS 'MAF';

-- Shanghai Tang engagements
-- UPDATE projects SET venue_id = venue:shanghai_tang WHERE name CONTAINS 'Shanghai Tang';

-- Lvl 63 engagements
-- UPDATE projects SET venue_id = venue:lvl_63 WHERE name CONTAINS 'Lvl 63' OR name CONTAINS 'Reserve Cut';

-- RAK Beach engagements
-- UPDATE projects SET venue_id = venue:rak_beach WHERE name CONTAINS 'RAK Beach';

-- ============================================================================
-- STEP 3: Diagnostic — Find projects without venue assignment
-- ============================================================================

-- Run this to see which projects still need venue assignment:
-- SELECT number.id, name, status, city, country FROM projects WHERE venue_id IS NONE;

-- To create a quick venue for each unlinked project:
-- FOR $p IN (SELECT * FROM projects WHERE venue_id IS NONE) {
--     LET $vid = string::slug($p.name);
--     CREATE type::thing('venue', $vid) CONTENT {
--         name: $p.name,
--         name_short: $p.name_short,
--         location: { city: $p.city, country: $p.country, area: $p.area },
--         tags: [],
--         notes: "Auto-created from project data"
--     };
--     UPDATE $p.id SET venue_id = type::thing('venue', $vid);
-- };
