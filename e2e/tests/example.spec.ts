import { test } from "../fixtures";
import { AdminLoginPage } from "../page-objects/adminLoginPage";
import { AdminVotingsPage } from "../page-objects/adminVotingsPage";

test("Can log in as admin", async ({ adminLoginPage }) => {
	const adminVotingsPage = await adminLoginPage.login();
	await adminVotingsPage.expectIsVisible();
});

test("Can create tokens", async ({ adminLoginPage }) => {
	const homePage = await adminLoginPage.login();
	await homePage.expectIsVisible();
	const tokensPage = await homePage.goToTokens();
	await tokensPage.expectIsVisible();
	await tokensPage.generateBulkTokens();
	await tokensPage.expectCanSeeTokens();
});
