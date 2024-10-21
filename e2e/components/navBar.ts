import type { Locator, Page } from "@playwright/test";

export class NavBar {
	private readonly page: Page;
	private readonly locator: Locator;
	private readonly auditLink: Locator;
	private readonly homeLink: Locator;
	private readonly logoutButton: Locator;

	constructor(locator: Locator) {
		this.page = locator.page();
		const auditLink = this.page.getByRole("link", {
			name: "app_registration",
		});
		const homeLink = this.page.getByRole("link").nth(1); // TODO: Get this with some more robust method
		const logoutButton = this.page.getByText("fingerprint");

		this.locator = locator
			.locator("div")
			.filter({ has: auditLink })
			.filter({ has: homeLink })
			.filter({ has: logoutButton });

		this.auditLink = this.locator.locator(auditLink);
		this.homeLink = this.locator.locator(homeLink);
		this.logoutButton = this.locator.locator(logoutButton);
	}

	public goToAudit() {
		return this.auditLink.click();
	}

	public goToHome() {
		return this.homeLink.click();
	}

	public logout() {
		return this.logoutButton.click();
	}
}
