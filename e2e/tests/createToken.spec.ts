import { test } from "../fixtures";

test("Can create tokens", async ({ adminLoginPage }) => {
	const votingsPage = await adminLoginPage.login();
	await votingsPage.expectIsVisible();
	const tokensPage = await votingsPage.goToTokens();
	await tokensPage.expectIsVisible();
	await tokensPage.generateBulkTokens();
	await tokensPage.expectCanSeeTokens();
});
