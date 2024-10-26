import type { Browser } from "@playwright/test";

export function generateRandomString(length = 10) {
	return Math.random().toString(20).substring(2, length);
}

export async function createPage(browser: Browser) {
	const context = await browser.newContext();
	const page = await context.newPage();
	return page;
}
