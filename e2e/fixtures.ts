import { test as base } from "@playwright/test";
import { LoginPage } from "./page-objects/loginPage";
import { AdminLoginPage } from "./page-objects/adminLoginPage";
import { createPage } from "./utils";

export interface LoginTest {
	adminLoginPage: AdminLoginPage;
	loginPage: LoginPage;
}

export const test = base.extend<LoginTest>({
	adminLoginPage: async ({ browser }, use) => {
		const page = await createPage(browser);
		const adminLoginPage = new AdminLoginPage(page);
		await adminLoginPage.goTo();
		await use(adminLoginPage);
		await page.close();
	},
	loginPage: async ({ browser }, use) => {
		const page = await createPage(browser);
		const loginPage = new LoginPage(page);
		await loginPage.goTo();
		await use(loginPage);
		await page.close();
	},
});
