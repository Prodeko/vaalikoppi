import { test } from "../fixtures";
import { AdminLoginPage } from "../page-objects/adminLoginPage";

test("Can log in as admin", async ({ adminLoginPage }) => {
	const adminHomePage = await adminLoginPage.login();
	await adminHomePage.expectIsVisible();
});

test("Can create tokens", async ({ adminLoginPage }) => {
	const adminHomePage = await adminLoginPage.login();
	await adminHomePage.expectIsVisible();
});
