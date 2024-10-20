import { test } from "../fixtures";
import { generateRandomString } from "../utils";

test("Can create voting as admin", async ({ adminLoginPage }) => {
	const adminVotingsPage = await adminLoginPage.login();
	const voting = {
		hideVoteCount: false,
		name: generateRandomString(),
		description: "",
		seats: 3,
	};
	await adminVotingsPage.createVoting(voting);
	await adminVotingsPage.expectVotingExists(voting);
});
