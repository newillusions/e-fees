-- 005-country-normalization.surql
-- Adds iso2 field (ISO 3166-1 alpha-2) to country table and defines fn::resolve_country
-- Run on: emittiv/projects DB (both prod 10.0.23.11 and dev 10.0.23.12)

-- 1. Add iso2 field and index
DEFINE FIELD iso2 ON country TYPE option<string>;
DEFINE INDEX idx_country_iso2 ON country FIELDS iso2;

-- 2. Populate iso2 for all 235 countries
UPDATE country:AFG SET iso2 = "AF";
UPDATE country:AIA SET iso2 = "AI";
UPDATE country:ALB SET iso2 = "AL";
UPDATE country:ALG SET iso2 = "DZ";
UPDATE country:AND SET iso2 = "AD";
UPDATE country:ANG SET iso2 = "AO";
UPDATE country:ANT SET iso2 = "AG";
UPDATE country:ARG SET iso2 = "AR";
UPDATE country:ARM SET iso2 = "AM";
UPDATE country:ARU SET iso2 = "AW";
UPDATE country:ASA SET iso2 = "AS";
UPDATE country:ATG SET iso2 = "AG";
UPDATE country:AUS SET iso2 = "AU";
UPDATE country:AUT SET iso2 = "AT";
UPDATE country:AZE SET iso2 = "AZ";
UPDATE country:BAH SET iso2 = "BS";
UPDATE country:BAN SET iso2 = "BD";
UPDATE country:BDI SET iso2 = "BI";
UPDATE country:BEL SET iso2 = "BE";
UPDATE country:BEN SET iso2 = "BJ";
UPDATE country:BER SET iso2 = "BM";
UPDATE country:BFA SET iso2 = "BF";
UPDATE country:BHR SET iso2 = "BH";
UPDATE country:BHU SET iso2 = "BT";
UPDATE country:BIH SET iso2 = "BA";
UPDATE country:BLR SET iso2 = "BY";
UPDATE country:BLZ SET iso2 = "BZ";
UPDATE country:BOL SET iso2 = "BO";
UPDATE country:BOT SET iso2 = "BW";
UPDATE country:BRA SET iso2 = "BR";
UPDATE country:BRB SET iso2 = "BB";
UPDATE country:BRU SET iso2 = "BN";
UPDATE country:BUL SET iso2 = "BG";
UPDATE country:CAM SET iso2 = "KH";
UPDATE country:CAN SET iso2 = "CA";
UPDATE country:CAY SET iso2 = "KY";
UPDATE country:CCK SET iso2 = "CC";
UPDATE country:CGO SET iso2 = "CG";
UPDATE country:CHA SET iso2 = "TD";
UPDATE country:CHI SET iso2 = "CL";
UPDATE country:CHN SET iso2 = "CN";
UPDATE country:CIV SET iso2 = "CI";
UPDATE country:CMR SET iso2 = "CM";
UPDATE country:COD SET iso2 = "CD";
UPDATE country:COK SET iso2 = "CK";
UPDATE country:COL SET iso2 = "CO";
UPDATE country:COM SET iso2 = "KM";
UPDATE country:CPV SET iso2 = "CV";
UPDATE country:CRC SET iso2 = "CR";
UPDATE country:CRO SET iso2 = "HR";
UPDATE country:CTA SET iso2 = "CF";
UPDATE country:CUB SET iso2 = "CU";
UPDATE country:CUW SET iso2 = "CW";
UPDATE country:CXR SET iso2 = "CX";
UPDATE country:CYP SET iso2 = "CY";
UPDATE country:CZE SET iso2 = "CZ";
UPDATE country:DEN SET iso2 = "DK";
UPDATE country:DJI SET iso2 = "DJ";
UPDATE country:DMA SET iso2 = "DM";
UPDATE country:DOM SET iso2 = "DO";
UPDATE country:ECU SET iso2 = "EC";
UPDATE country:EGY SET iso2 = "EG";
UPDATE country:EQG SET iso2 = "GQ";
UPDATE country:ERI SET iso2 = "ER";
UPDATE country:ESP SET iso2 = "ES";
UPDATE country:EST SET iso2 = "EE";
UPDATE country:ETH SET iso2 = "ET";
UPDATE country:FIJ SET iso2 = "FJ";
UPDATE country:FIN SET iso2 = "FI";
UPDATE country:FLK SET iso2 = "FK";
UPDATE country:FRA SET iso2 = "FR";
UPDATE country:FRO SET iso2 = "FO";
UPDATE country:FSM SET iso2 = "FM";
UPDATE country:GAB SET iso2 = "GA";
UPDATE country:GAM SET iso2 = "GM";
UPDATE country:GBG SET iso2 = "GG";
UPDATE country:GBJ SET iso2 = "JE";
UPDATE country:GBM SET iso2 = "IM";
UPDATE country:GBR SET iso2 = "GB";
UPDATE country:GBZ SET iso2 = "GI";
UPDATE country:GEO SET iso2 = "GE";
UPDATE country:GER SET iso2 = "DE";
UPDATE country:GHA SET iso2 = "GH";
UPDATE country:GLP SET iso2 = "GP";
UPDATE country:GNB SET iso2 = "GW";
UPDATE country:GRE SET iso2 = "GR";
UPDATE country:GRL SET iso2 = "GL";
UPDATE country:GRN SET iso2 = "GD";
UPDATE country:GUA SET iso2 = "GT";
UPDATE country:GUF SET iso2 = "GF";
UPDATE country:GUI SET iso2 = "GN";
UPDATE country:GUM SET iso2 = "GU";
UPDATE country:GUY SET iso2 = "GY";
UPDATE country:HAI SET iso2 = "HT";
UPDATE country:HKG SET iso2 = "HK";
UPDATE country:HON SET iso2 = "HN";
UPDATE country:HUN SET iso2 = "HU";
UPDATE country:IDN SET iso2 = "ID";
UPDATE country:IND SET iso2 = "IN";
UPDATE country:IRL SET iso2 = "IE";
UPDATE country:IRN SET iso2 = "IR";
UPDATE country:IRQ SET iso2 = "IQ";
UPDATE country:ISL SET iso2 = "IS";
UPDATE country:ISR SET iso2 = "IL";
UPDATE country:ITA SET iso2 = "IT";
UPDATE country:JAM SET iso2 = "JM";
UPDATE country:JOR SET iso2 = "JO";
UPDATE country:JPN SET iso2 = "JP";
UPDATE country:KAZ SET iso2 = "KZ";
UPDATE country:KEN SET iso2 = "KE";
UPDATE country:KGZ SET iso2 = "KG";
UPDATE country:KIR SET iso2 = "KI";
UPDATE country:KOR SET iso2 = "KR";
UPDATE country:KSA SET iso2 = "SA";
UPDATE country:KUW SET iso2 = "KW";
UPDATE country:LAO SET iso2 = "LA";
UPDATE country:LBR SET iso2 = "LR";
UPDATE country:LBY SET iso2 = "LY";
UPDATE country:LCA SET iso2 = "LC";
UPDATE country:LES SET iso2 = "LS";
UPDATE country:LIB SET iso2 = "LB";
UPDATE country:LIE SET iso2 = "LI";
UPDATE country:LTU SET iso2 = "LT";
UPDATE country:LUX SET iso2 = "LU";
UPDATE country:LVA SET iso2 = "LV";
UPDATE country:MAC SET iso2 = "MO";
UPDATE country:MAD SET iso2 = "MG";
UPDATE country:MAR SET iso2 = "MA";
UPDATE country:MAS SET iso2 = "MY";
UPDATE country:MDA SET iso2 = "MD";
UPDATE country:MDV SET iso2 = "MV";
UPDATE country:MEX SET iso2 = "MX";
UPDATE country:MHL SET iso2 = "MH";
UPDATE country:MKD SET iso2 = "MK";
UPDATE country:MLI SET iso2 = "ML";
UPDATE country:MLT SET iso2 = "MT";
UPDATE country:MNE SET iso2 = "ME";
UPDATE country:MNG SET iso2 = "MN";
UPDATE country:MON SET iso2 = "MC";
UPDATE country:MOZ SET iso2 = "MZ";
UPDATE country:MRI SET iso2 = "MU";
UPDATE country:MSR SET iso2 = "MS";
UPDATE country:MTN SET iso2 = "MR";
UPDATE country:MTQ SET iso2 = "MQ";
UPDATE country:MWI SET iso2 = "MW";
UPDATE country:MYA SET iso2 = "MM";
UPDATE country:MYT SET iso2 = "YT";
UPDATE country:NAM SET iso2 = "NA";
UPDATE country:NCA SET iso2 = "NI";
UPDATE country:NCL SET iso2 = "NC";
UPDATE country:NED SET iso2 = "NL";
UPDATE country:NEP SET iso2 = "NP";
UPDATE country:NFK SET iso2 = "NF";
UPDATE country:NGA SET iso2 = "NG";
UPDATE country:NIG SET iso2 = "NE";
UPDATE country:NIU SET iso2 = "NU";
UPDATE country:NMI SET iso2 = "MP";
UPDATE country:NOR SET iso2 = "NO";
UPDATE country:NRU SET iso2 = "NR";
UPDATE country:NZL SET iso2 = "NZ";
UPDATE country:OMA SET iso2 = "OM";
UPDATE country:PAK SET iso2 = "PK";
UPDATE country:PAN SET iso2 = "PA";
UPDATE country:PAR SET iso2 = "PY";
UPDATE country:PCN SET iso2 = "PN";
UPDATE country:PER SET iso2 = "PE";
UPDATE country:PHI SET iso2 = "PH";
UPDATE country:PLW SET iso2 = "PW";
UPDATE country:PNG SET iso2 = "PG";
UPDATE country:POL SET iso2 = "PL";
UPDATE country:POR SET iso2 = "PT";
UPDATE country:PRK SET iso2 = "KP";
UPDATE country:PUR SET iso2 = "PR";
UPDATE country:QAT SET iso2 = "QA";
UPDATE country:REU SET iso2 = "RE";
UPDATE country:ROU SET iso2 = "RO";
UPDATE country:RSA SET iso2 = "ZA";
UPDATE country:RUS SET iso2 = "RU";
UPDATE country:RWA SET iso2 = "RW";
UPDATE country:SAH SET iso2 = "EH";
UPDATE country:SAM SET iso2 = "WS";
UPDATE country:SEN SET iso2 = "SN";
UPDATE country:SEY SET iso2 = "SC";
UPDATE country:SHN SET iso2 = "SH";
UPDATE country:SIN SET iso2 = "SG";
UPDATE country:SKN SET iso2 = "KN";
UPDATE country:SLE SET iso2 = "SL";
UPDATE country:SLV SET iso2 = "SV";
UPDATE country:SMR SET iso2 = "SM";
UPDATE country:SOL SET iso2 = "SB";
UPDATE country:SOM SET iso2 = "SO";
UPDATE country:SPM SET iso2 = "PM";
UPDATE country:SRB SET iso2 = "RS";
UPDATE country:SRI SET iso2 = "LK";
UPDATE country:STP SET iso2 = "ST";
UPDATE country:SUD SET iso2 = "SD";
UPDATE country:SUI SET iso2 = "CH";
UPDATE country:SUR SET iso2 = "SR";
UPDATE country:SVK SET iso2 = "SK";
UPDATE country:SVN SET iso2 = "SI";
UPDATE country:SWE SET iso2 = "SE";
UPDATE country:SWZ SET iso2 = "SZ";
UPDATE country:SXM SET iso2 = "SX";
UPDATE country:SYR SET iso2 = "SY";
UPDATE country:TAH SET iso2 = "PF";
UPDATE country:TAN SET iso2 = "TZ";
UPDATE country:TCA SET iso2 = "TC";
UPDATE country:TGA SET iso2 = "TO";
UPDATE country:THA SET iso2 = "TH";
UPDATE country:TJK SET iso2 = "TJ";
UPDATE country:TKL SET iso2 = "TK";
UPDATE country:TKM SET iso2 = "TM";
UPDATE country:TLS SET iso2 = "TL";
UPDATE country:TOG SET iso2 = "TG";
UPDATE country:TPE SET iso2 = "TW";
UPDATE country:TRI SET iso2 = "TT";
UPDATE country:TUN SET iso2 = "TN";
UPDATE country:TUR SET iso2 = "TR";
UPDATE country:TUV SET iso2 = "TV";
UPDATE country:UAE SET iso2 = "AE";
UPDATE country:UGA SET iso2 = "UG";
UPDATE country:UKR SET iso2 = "UA";
UPDATE country:URU SET iso2 = "UY";
UPDATE country:USA SET iso2 = "US";
UPDATE country:UZB SET iso2 = "UZ";
UPDATE country:VAN SET iso2 = "VU";
UPDATE country:VAT SET iso2 = "VA";
UPDATE country:VEN SET iso2 = "VE";
UPDATE country:VGB SET iso2 = "VG";
UPDATE country:VIE SET iso2 = "VN";
UPDATE country:VIN SET iso2 = "VC";
UPDATE country:WLF SET iso2 = "WF";
UPDATE country:YEM SET iso2 = "YE";
UPDATE country:ZAM SET iso2 = "ZM";
UPDATE country:ZIM SET iso2 = "ZW";

-- 3. Define resolve function
-- Accepts any reasonable country input and returns canonical { name, code, iso2, dial_code }
-- Match cascade: code → iso2 → code_alt → name exact → name dot-stripped → official/formal
DEFINE FUNCTION fn::resolve_country($input: string) {
    LET $clean = string::trim($input);
    LET $lower = string::lowercase($clean);
    LET $nodots = string::replace($lower, ".", "");

    -- 1. code exact (3-letter: "UAE", "KSA")
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE code = string::uppercase($clean) LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 2. iso2 exact (2-letter: "AE", "SA")
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE iso2 = string::uppercase($clean) LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 3. code_alt exact (alt 3-letter: "ARE", "SAU")
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE code_alt = string::uppercase($clean) LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 4. name exact ("U.A.E.", "K.S.A.", "India")
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE name = $clean LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 5. name dot-stripped ("UAE" matches "U.A.E." after removing dots)
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE string::lowercase(string::replace(name, ".", "")) = $nodots LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 6. name_official / name_formal exact (case-insensitive)
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE string::lowercase(name_official) = $lower
           OR string::lowercase(name_formal) = $lower LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 7. name_official / name_formal contains (case-insensitive)
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE string::lowercase(name_official) CONTAINS $lower
           OR string::lowercase(name_formal) CONTAINS $lower LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    RETURN NONE;
};
