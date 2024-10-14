import { test } from "../fixtures";

test("User can log in", async ({ adminLoginPage }) => {
	const adminHomePage = await adminLoginPage.login();
	const adminTokensPage = await adminHomePage.goToTokens();
	const tokens = await adminTokensPage.generateBulkTokens();
});
