-- dated      | metric       | val
-- ---------------------------------
-- 2022-01-01 | unemployment | 1234
CREATE TABLE IF NOT EXISTS econ.us_std (
	dated   DATE NOT NULL,
	metric	VARCHAR NOT NULL,
	val     FLOAT NOT NULL,
	PRIMARY KEY (dated, metric, val)
);

CREATE TABLE IF NOT EXISTS econ.us_lobbying (
	dated 			DATE NOT NULL,
	filer_type   		VARCHAR,
	filing_state 		VARCHAR,
	filing_country 		VARCHAR,
	registrant_name 	VARCHAR,
	registrant_desc 	VARCHAR,
	registrant_state 	VARCHAR,
	registrant_country 	VARCHAR,
	registrant_contact 	VARCHAR,
	lobbyist_first_name 	VARCHAR,
	lobbyist_middle_name 	VARCHAR,
	lobbyist_last_name 	VARCHAR,
	pacs 			VARCHAR[],
	contr_type 		VARCHAR,
	contr_type_disp 	VARCHAR,
	contr_name 		VARCHAR,
	payee_name 		VARCHAR,
	honoree_name 		VARCHAR,
	amount 			FLOAT,
	PRIMARY KEY (dated, filer_type, contr_name, payee_name, honoree_name, amount)
);
