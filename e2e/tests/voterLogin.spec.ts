import { test } from "../fixtures";

test("User can log in", async ({ adminLoginPage, loginPage }) => {
	const adminHomePage = await adminLoginPage.login();
	const adminTokensPage = await adminHomePage.goToTokens();
	const tokens = await adminTokensPage.generateBulkTokens();
	const selectedToken = tokens[0];
	await adminTokensPage.activateToken(selectedToken);
	await loginPage.login({ alias: "ASDFSDF", token: selectedToken });
});
