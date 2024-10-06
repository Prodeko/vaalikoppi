import { test } from "../fixtures";
import { AdminLoginPage } from "../page-objects/adminLoginPage";

test("Can log in as admin", async ({ adminLoginPage }) => {
	const adminHomePage = await adminLoginPage.login();
	await adminHomePage.expectIsVisible();
});

test("Can create tokens", async ({ adminLoginPage }) => {
	const homePage = await adminLoginPage.login();
	await homePage.expectIsVisible();
	const tokensPage = await homePage.goToTokens();
	await tokensPage.expectIsVisible();
	await tokensPage.generateBulkTokens();
	await tokensPage.expectCanSeeTokens();
});
