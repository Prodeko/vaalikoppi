import { test } from "../fixtures";
import { generateRandomString } from "../utils";

test("User can log in", async ({ adminLoginPage, loginPage }) => {
	const adminVotingsPage = await adminLoginPage.login();
	const adminTokensPage = await adminVotingsPage.goToTokens();
	const tokens = await adminTokensPage.generateBulkTokens();
	const selectedToken = tokens[0];
	await adminTokensPage.activateToken(selectedToken);

	const votingsPage = await loginPage.login({
		alias: generateRandomString(),
		token: selectedToken,
	});

	await votingsPage.expectIsVisible();
});
