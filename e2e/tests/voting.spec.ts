import { test } from "../fixtures";
import { LoginPage } from "../page-objects/loginPage";
import type { CreateVoting } from "../types";
import { createPage, generateRandomString } from "../utils";

test("Full voting flow with 3 voters", async ({ browser, adminLoginPage }) => {
	let adminVotingsPage = await adminLoginPage.login();
	const adminTokensPage = await adminVotingsPage.goToTokens();
	const allTokens = await adminTokensPage.generateBulkTokens();
	adminVotingsPage = await adminTokensPage.goToVotings();

	const voting: CreateVoting = {
		hideVoteCount: false,
		name: generateRandomString(),
		description: "",
		seats: 2,
		candidates: ["a", "b", "c", "d", "e", "f"],
	};
	await adminVotingsPage.createVoting(voting);
	await adminVotingsPage.expectVotingExists(voting);

	const numberOfVoters = 3;

	const tokens = allTokens.slice(0, numberOfVoters);

	await Promise.all(
		tokens.map(async (t) => {
			const page = await createPage(browser);
			const userPage = new LoginPage(page);
			await userPage.goTo();

			const userVotingPage = await userPage.login({
				alias: generateRandomString(10),
				token: t,
			});
		}),
	);
});
