import { test } from "../fixtures";
import { generateRandomString } from "../utils";

test("User can log in", async ({ adminLoginPage, loginPage }) => {
	const adminHomePage = await adminLoginPage.login();
	const adminTokensPage = await adminHomePage.goToTokens();
	const tokens = await adminTokensPage.generateBulkTokens();
	const selectedToken = tokens[0];
	await adminTokensPage.activateToken(selectedToken);

	const homePage = await loginPage.login({
		alias: generateRandomString(),
		token: selectedToken,
	});

	await homePage.expectIsVisible();
});
