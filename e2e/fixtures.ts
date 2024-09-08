import { test as base } from "@playwright/test";
import { LoginPage } from "./page-objects/loginPage";
import { AdminLoginPage } from "./page-objects/adminLoginPage";

export interface LoginTest {
	adminLoginPage: AdminLoginPage;
	loginPage: LoginPage;
}

export const test = base.extend<LoginTest>({
	adminLoginPage: async ({ browser }, use) => {
		const context = await browser.newContext();
		const page = await context.newPage();
		const adminLoginPage = new AdminLoginPage(page);
		await adminLoginPage.goTo();
		await use(adminLoginPage);
	},
	loginPage: async ({ browser }, use) => {
		const context = await browser.newContext();
		const page = await context.newPage();
		const loginPage = new LoginPage(page);
		await loginPage.goTo();
		await use(loginPage);
	},
});
