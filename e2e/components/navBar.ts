import type { Locator, Page } from "@playwright/test";

export class NavBar {
	private readonly auditLink: Locator;
	private readonly homeLink: Locator;
	private readonly logoutButton: Locator;

	constructor(private readonly locator: Locator) {
		this.auditLink = this.locator.getByRole("link", {
			name: "app_registration",
		});
		this.homeLink = this.locator.getByRole("link").nth(1); // TODO: Get this with some more robust method
		this.auditLink = this.locator.getByText("fingerprint");
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
