BEGIN TRANSACTION;

ALTER TABLE candidate_result_data
	DROP CONSTRAINT candidate_result_data_voting_id_name_fkey;

ALTER TABLE candidate_result_data
    ADD CONSTRAINT candidate_result_data_voting_id_name_fkey
	FOREIGN KEY (voting_id, name)
	REFERENCES candidate(voting_id, name);


ALTER TABLE candidate_result_data
	DROP CONSTRAINT candidate_result_data_voting_id_round_fkey;

ALTER TABLE candidate_result_data
    ADD CONSTRAINT candidate_result_data_voting_id_round_fkey
	FOREIGN KEY (voting_id, round)
	REFERENCES voting_round_result(voting_id, round);


ALTER TABLE voting_round_result
	DROP CONSTRAINT fk_ensure_dropped_candidate_has_data;

ALTER TABLE voting_round_result
    ADD CONSTRAINT fk_ensure_dropped_candidate_has_data
	FOREIGN KEY (voting_id, dropped_candidate_name, round)
	REFERENCES candidate_result_data(voting_id, name, round);


ALTER TABLE token
	DROP CONSTRAINT fk_link_token_to_election;

ALTER TABLE token
    ADD CONSTRAINT fk_link_token_to_election
	FOREIGN KEY (election_id)
	REFERENCES election(id);


ALTER TABLE voting
	DROP CONSTRAINT fk_link_voting_to_election;

ALTER TABLE voting
    ADD CONSTRAINT fk_link_voting_to_election
	FOREIGN KEY (election_id)
	REFERENCES election(id);


ALTER TABLE ONLY has_voted
	DROP CONSTRAINT has_voted_token_token_fkey;

ALTER TABLE ONLY has_voted
    ADD CONSTRAINT has_voted_token_token_fkey
	FOREIGN KEY (token_token)
	REFERENCES token(token);


ALTER TABLE passing_candidate_result
	DROP CONSTRAINT passing_candidate_result_voting_id_name_round_fkey;

ALTER TABLE passing_candidate_result
	ADD CONSTRAINT passing_candidate_result_voting_id_name_round_fkey
	FOREIGN KEY (voting_id, name, round)
	REFERENCES candidate_result_data(voting_id, name, round);


ALTER TABLE voting_round_result
	DROP CONSTRAINT voting_round_result_voting_id_fkey;

ALTER TABLE voting_round_result
    ADD CONSTRAINT voting_round_result_voting_id_fkey
	FOREIGN KEY (voting_id)
	REFERENCES voting(id);

COMMIT;
