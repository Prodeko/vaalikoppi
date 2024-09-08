import { test } from "../fixtures";

test("has title", async ({ loginPage, adminLoginPage }) => {
	const adminHomePage = await adminLoginPage.login();
	await adminHomePage.expectIsVisible();
});
