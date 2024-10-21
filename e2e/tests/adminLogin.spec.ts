import { test } from "../fixtures";

test("Can log in as admin", async ({ adminLoginPage }) => {
	const adminVotingsPage = await adminLoginPage.login();
	await adminVotingsPage.expectIsVisible();
});
